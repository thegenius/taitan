use crate::error::Error;
use crate::result::Result;
use axum::{
    body::Bytes,
    extract::{multipart::Field, Multipart, Request},
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
use std::path::Path;
use tempfile::tempfile;

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
pub async fn save_to_file(dir: &Path, mut multipart: Multipart, uuid_name: Option<String>) -> Result<Vec<String>> {
    // request_uuid must place on the heading of multipart
    info!("save_to_file({:?}, {:?})", dir, uuid_name);
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

// std::fs::File 还不支持异步操作
// pub async fn save_to_temp_file(mut multipart: Multipart, uuid_name: Option<String>) -> Result<Vec<String>> {
//     // request_uuid must place on the heading of multipart
//     info!("save_to_temp_file( uuid_name: {:?})", uuid_name);
//     if let Some(uuid_name) = uuid_name {
//         if let Ok(Some(field)) = multipart.next_field().await {
//             if let Some(field_name) = field.name() {
//                 if field_name == uuid_name {
//                     let data = field.bytes().await?;
//                     let data_string = String::from_utf8(data.to_vec())?;
//                     let uuid_prefix = uuid::Uuid::parse_str(&data_string)?;
//                     let prefix = uuid_prefix.as_simple().to_string();
//                     return Ok(save_to_file_with_prefix(dir, &prefix, multipart).await?);
//                 }
//             }
//         }
//     } else {
//         return Ok(save_to_file_with_prefix(dir, "", multipart).await?);
//     }
//     return Ok(Vec::new());
// }


fn get_validate_file_name(dir: &Path, prefix_string: &str, file_name: &str, field: &Field) -> Option<String> {
    let final_file_name: String;
    if prefix_string.is_empty() {
        final_file_name = file_name.to_owned();
        if !validate_path(dir.as_ref(), &final_file_name, 1) {
            return None;
        }
    } else {
        final_file_name = format!("{}.{}", prefix_string.to_string(), file_name.to_owned());
        if !validate_path(dir.as_ref(), &final_file_name, 2) {
            return None;
        }
    }

    // let (_, upper_bound)= field.size_hint();
    // if upper_bound.is_none() {
    //     debug!("file size not known");
    //     return None;
    // }
    
    // const SINGLE_FILE_MAX_SIZE: usize = 5 * 1024 * 1024; // single file limit
    // let upper_bound = upper_bound.unwrap();
    // if upper_bound > SINGLE_FILE_MAX_SIZE {
    //     debug!("file size larger than 5MB");
    //     return None;
    // }

    return Some(final_file_name);
}

pub async fn save_to_file_with_prefix(dir: &Path, prefix: impl AsRef<str>, mut multipart: Multipart) -> Result<Vec<String>> {
    info!("save_to_file_with_prefix({:?}, {:?})", dir, prefix.as_ref());
    let mut files: Vec<String> = Vec::new();
    let prefix_string = prefix.as_ref();
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(file_name) = field.file_name() {
            let final_file_name = get_validate_file_name(dir, prefix_string.as_ref(), file_name, &field);
            if let Some(final_file_name) = final_file_name {
                debug!("save_to_file_with_prefix - final_file_name: {:?}", final_file_name);
                let mut file = create_file(&dir,  &final_file_name).await?;
                stream_to_file(&mut file, field).await?;
                files.push(final_file_name);
            }
        } else {
            continue;
        };
    }
    return Ok(files);
}

// 因为std::fs::File还不支持异步操作，所以这个函数还无法实现
// pub async fn save_temp_file(mut multipart: Multipart) -> Result<Vec<String>> {
//     info!("save_temp_file");
//     let mut files: Vec<String> = Vec::new();
//     while let Ok(Some(field)) = multipart.next_field().await {
//         if let Some(file_name) = field.file_name() {
            
//                 let mut file = tempfile()?;
//                 // let mut file = create_file(&dir,  &final_file_name).await?;
//                 stream_to_temp_file(&mut file, field).await?;
//                 files.push(file_name);
            
//         } else {
//             continue;
//         };
//     }
//     return Ok(files);
// }

// 因为std::fs::File还不支持异步操作，所以这个函数还无法实现
// pub async fn stream_to_temp_file<S, E>(file: &mut std::fs::File, stream: S) -> Result<()>
// where
//     S: Stream<Item = std::result::Result<Bytes, E>>,
//     E: Into<BoxError>,
// {
//     debug!("stream_to_file begin ...");
//     // Convert the stream into an `AsyncRead`.
//     let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
//     let body_reader = StreamReader::new(body_with_io_error);
//     futures::pin_mut!(body_reader);

//     let mut file = BufWriter::new(file);

//     // Copy the body into the file.
//     tokio::io::copy(&mut body_reader, &mut file)
//         .await
//         .map_err(Error::FileError)?;
//     debug!("stream_to_file success");
//     Ok(())
// }


async fn create_file(dir: &Path, file_name: impl AsRef<str>) -> Result<File> {
    debug!("create_file: {:?}, {:?}", dir, file_name.as_ref());
    let path = dir.join(file_name.as_ref());
    let path_clone = path.clone();
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path)
        .await
        .map_err(|err| Error::FileError(err))?;
    debug!("create_file success: {:?}, {:?}", path_clone, file_name.as_ref());
    Ok(file)
}

pub async fn stream_to_file<S, E>(file: &mut File, stream: S) -> Result<()>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    E: Into<BoxError>,
{
    debug!("stream_to_file begin ...");
    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    let mut file = BufWriter::new(file);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file)
        .await
        .map_err(Error::FileError)?;
    debug!("stream_to_file success");
    Ok(())
}

pub fn validate_path(dir: &Path, file_name: &str, valid_component: usize) -> bool {
    let path = dir.join(file_name);
    return is_path_valid(path.as_path(), valid_component);
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
pub fn is_path_valid(path: &Path, valid_component: usize) -> bool {
    // let path = std::path::Path::new(path);
    let mut components = path.components().peekable();
    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }
    let count = components.count();
    return count == valid_component;
}
