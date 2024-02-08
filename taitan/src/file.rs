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
use futures::{Stream, TryStreamExt};
use serde_json::error::Category;
use std::io;
use std::io::SeekFrom;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::{fs::File, fs::OpenOptions, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::info;

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

async fn save_to_file(dir: impl AsRef<str>, mut multipart: Multipart) -> Result<String> {
    let final_file = "".to_string();
    let mut file = create_file(dir, &final_file).await?;
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        stream_to_file(&mut file, field).await?;
    }

    Ok(final_file)
}

async fn create_file(dir: impl AsRef<str>, file_name: impl AsRef<str>) -> Result<File> {
    let path = std::path::Path::new(dir.as_ref()).join(file_name.as_ref());
    if !path_is_valid(path.to_str().unwrap()) {
        return Err(Error::logic_error("invalid path"));
    }
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path)
        .await
        .map_err(|err| Error::FileError(err))?;
    Ok(file)
}

async fn stream_to_file<S, E>(file: &mut File, stream: S) -> Result<()>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    E: Into<BoxError>,
{
    /*
    file.seek(SeekFrom::Start(offset))
        .await
        .map_err(Error::FileError)?;
    */

    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    let mut file = BufWriter::new(file);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file)
        .await
        .map_err(Error::FileError)?;
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
