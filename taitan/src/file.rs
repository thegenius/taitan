use crate::error::Error;
use crate::result::Result;
use axum::{
    body::Bytes,
    extract::{Multipart, Path, Request},
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
    BoxError, Router,
};
use futures::{FutureExt, Stream, TryStreamExt};
use serde_json::error::Category;
use std::io;
use std::io::SeekFrom;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::{fs::File, fs::OpenOptions, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::{debug, info};
use uuid::Uuid;

pub trait FileManager {
    // return the list of owner's file of specified category
    async fn list_files<T>(
        owner: impl AsRef<str>,
        category: impl AsRef<[T]>,
    ) -> Result<Vec<String>>
    where
        T: AsRef<T>;

    // write file, return etag
    async fn write_file(file_name: impl AsRef<str>, data: Bytes) -> Result<String>;

    // write file seg, true if write, false if already exists
    async fn write_seg(file_name: impl AsRef<str>, seg_no: u32, file_seg: Bytes) -> Result<bool>;

    // return true if the file's etag equals to the input etag
    async fn check_etag(file_name: impl AsRef<str>, etag: impl AsRef<str>) -> Result<bool>;
}

// file may be endup with .001.xseg
fn get_origin_file_name(file_name: impl AsRef<str>) -> String {
    let file_name = file_name.as_ref();
    if !file_name.ends_with(".xseg") {
        return file_name.to_owned();
    }

    let mut splits: Vec<&str> = file_name.split('.').collect();
    let splits_len = splits.len();
    if splits_len <= 2 {
        return file_name.to_owned();
    }
    splits.truncate(splits_len - 2);
    splits.join(".")
}

fn check_etag(file_name: &str, etag: &str) -> Result<bool> {
    Ok(true)
}

fn save_etag(file_name: &str, etag: &str) -> Result<()> {
    Ok(())
}
/* ********************************************
   Content-Disposition: form-data; name="files"
   Content-Type: multipart/mixed; boundary=BbC04y

   --BbC04y
   Content-Disposition: file; filename="file1.txt"
   Content-Type: text/plain

   ... contents of file1.txt ...
   --BbC04y
   Content-Disposition: file; filename="file2.gif"
   Content-Type: image/gif
   Content-Transfer-Encoding: binary

   ...contents of file2.gif...
   --BbC04y--
   --AaB03x--
*/
// multipart可以上传多个文件，但是整体的multipart的axum默认是2MB，在taitan中默认改为了10MB
pub async fn save_to_file(dir: impl AsRef<str>, mut multipart: Multipart, uuid_name: Option<String>) -> Result<Vec<String>> {
    // request_uuid must place on the heading of multipart
    info!("save_to_file({:?}, {:?})", dir.as_ref(), uuid_name);
    if let Some(uuid_name) = uuid_name {
        if let Ok(Some(field)) = multipart.next_field().await {
            if let Some(field_name) = field.name() {
                if field_name == uuid_name {
                    let data = field.bytes().await?;
                    let data_string = String::from_utf8(data.to_vec())?;
                    let uuid_prefix = uuid::Uuid::parse_str(&data_string)?;
                    let prefix = uuid_prefix.as_simple().to_string();
                    return Ok(save_to_file_with_prefix(dir, &prefix, multipart).await?);
                }
            }
        }
    } else {
        return Ok(save_to_file_with_prefix(dir, "", multipart).await?);
    }
    return Ok(Vec::new());
}


pub async fn save_to_file_with_prefix(dir: impl AsRef<str>, prefix: impl AsRef<str>, mut multipart: Multipart) -> Result<Vec<String>> {
    info!("save_to_file_with_prefix({:?}, {:?})", dir.as_ref(), prefix.as_ref());
    let mut files: Vec<String> = Vec::new();
    let prefix_string = prefix.as_ref();
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(file_name) = field.file_name() {
            let final_file_name = format!("{}.{}", prefix_string.to_string(), file_name.to_owned());
            let mut file = create_file(&dir, &final_file_name).await?;
            debug!("save_to_file_with_prefix - final_file_name: {:?}", final_file_name);
            let (_, upper_bound)= field.size_hint();
            if upper_bound.is_none() {
                return Err(Error::logic_error("file size not known"));
            }
            const SINGLE_FILE_MAX_SIZE: usize = 5 * 1024 * 1024; // single file limit
            let upper_bound = upper_bound.unwrap();
            if upper_bound > SINGLE_FILE_MAX_SIZE {
                debug!("");
                return Err(Error::logic_error("file size larger than 5MB"));
            }

            files.push(final_file_name);
            stream_to_file(&mut file, field).await?;
        } else {
            continue;
        };
    }
    return Ok(files);
}



async fn create_file(dir: impl AsRef<str>, file_name: impl AsRef<str>) -> Result<File> {
    info!("create_file: {:?}, {:?}", dir.as_ref(), file_name.as_ref());
    let path = std::path::Path::new(dir.as_ref()).join(file_name.as_ref());
    if !path_is_valid(path.to_str().unwrap()) {
        return Err(Error::logic_error("invalid path"));
    }
    info!("create_file path valid: {:?}, {:?}", dir.as_ref(), file_name.as_ref());
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path)
        .await
        .map_err(|err| Error::FileError(err))?;
    info!("create_file success: {:?}, {:?}", dir.as_ref(), file_name.as_ref());
    Ok(file)
}

pub async fn stream_to_file<S, E>(file: &mut File, stream: S) -> Result<()>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    E: Into<BoxError>,
{
    /*
    file.seek(SeekFrom::Start(offset))
        .await
        .map_err(Error::FileError)?;
    */
    info!("stream_to_file begin ...");
    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    let mut file = BufWriter::new(file);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file)
        .await
        .map_err(Error::FileError)?;
    info!("stream_to_file success");
    Ok(())
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
pub fn path_is_valid(path: impl AsRef<str>) -> bool {
    let path = std::path::Path::new(path.as_ref());
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
