//! Local storage backend for the registry.

use super::{RegistryStorage, WritableRegistryStorage};
use crate::{
    crate_utils::crate_name_to_index,
    header_util::{self, get_modified_since},
};
use axum::{
    body::Bytes,
    http::StatusCode,
    response::{AppendHeaders, IntoResponse, Response},
    Error,
};
use futures_util::StreamExt;
use std::{
    io::{BufRead, Cursor, Read, Write},
    path::PathBuf,
    time::SystemTime,
};

/// Local storage backend
pub struct LocalStorage {
    index_path: PathBuf,
    crate_path: PathBuf,
}

impl LocalStorage {
    /// Create a new [`LocalStorage`]
    pub fn new(index_path: PathBuf, crate_path: PathBuf) -> Self {
        Self {
            index_path,
            crate_path,
        }
    }

    fn inner(
        path: PathBuf,
        last_modified: Option<SystemTime>,
    ) -> Result<Option<(Vec<u8>, SystemTime)>, std::io::Error> {
        tracing::trace!(path = ?path, "Getting local file");
        let mut vec = vec![];
        let file = &mut std::fs::File::open(path)?;
        let time = file.metadata()?.modified()?;
        if let Some(last_modified) = last_modified {
            if time <= last_modified {
                return Ok(None);
            }
        }
        file.read_to_end(&mut vec)?;
        Ok(Some((vec, time)))
    }

    /// Get a local file for the path.
    ///
    /// If the file is not modified since `last_modified`, it will return a 304.
    async fn get_local_file(path: PathBuf, last_modified: Option<SystemTime>) -> Response {
        let (vec, time) = match Self::inner(path, last_modified) {
            Ok(Some(f)) => f,
            Ok(None) => return (StatusCode::NOT_MODIFIED).into_response(),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
                    return StatusCode::NOT_FOUND.into_response()
                }
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
        };
        (
            AppendHeaders(header_util::to_modified(time)),
            Bytes::from(vec),
        )
            .into_response()
    }

    async fn create_new_local_file(path: PathBuf, data: Vec<u8>) -> Response {
        let mut file = match std::fs::File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(f) => f,
            Err(e) => {
                let kind = e.kind();
                tracing::error!(?e, path = %path.display(), "Failed to open file");
                return match kind {
                    std::io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN.into_response(),
                    std::io::ErrorKind::AlreadyExists => {
                        (StatusCode::CONFLICT, "Already Exists").into_response()
                    }
                    _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                };
            }
        };
        if let Err(e) = file.write_all(&data) {
            let kind = e.kind();
            tracing::error!(?e, path = %path.display(), "Failed to write to file");
            match kind {
                std::io::ErrorKind::PermissionDenied => {
                    return StatusCode::FORBIDDEN.into_response()
                }
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        StatusCode::OK.into_response()
    }
}

impl RegistryStorage for LocalStorage {
    fn get_index_file(
        &self,
        headers: &axum::http::HeaderMap,
        index_path: &str,
    ) -> impl std::future::Future<Output = impl axum::response::IntoResponse> + Send {
        let last = get_modified_since(headers);
        Self::get_local_file(self.index_path.join(index_path), last)
    }

    fn get_crate_file(
        &self,
        headers: &axum::http::HeaderMap,
        crate_name: &str,
        version: &str,
    ) -> impl std::future::Future<Output = impl axum::response::IntoResponse> + Send {
        let last = get_modified_since(headers);
        let mut path: PathBuf = self.crate_path.clone();
        path.push(format!("{crate_name}/{version}.crate"));
        Self::get_local_file(path, last)
    }

    fn get_index_data(
        &self,
        crate_name: &str,
    ) -> impl futures_util::TryStream<Ok = crate::index::Index, Error = axum::Error> + Send
    where
        Self: Sized,
    {
        match std::fs::read(crate_name_to_index(crate_name)) {
            Ok(data) => {
                let items = Cursor::new(data).lines().map(|line| {
                    line.map_err(Error::new)
                        .and_then(|s| serde_json::from_str(&s).map_err(Error::new))
                });
                futures_util::stream::iter(items).left_stream()
            }
            Err(e) => futures_util::stream::once(async { Err(Error::new(e)) }).right_stream(),
        }
    }
}

impl WritableRegistryStorage for LocalStorage {
    async fn put_index(
        &self,
        headers: &axum::http::HeaderMap,
        index_path: &str,
        data: Vec<u8>,
    ) -> impl IntoResponse {
    }

    async fn put_crate(
        &self,
        headers: &axum::http::HeaderMap,
        crate_name: &str,
        version: &str,
        data: Vec<u8>,
    ) -> impl IntoResponse {
        Self::create_new_local_file(
            self.crate_path
                .join(format!("{crate_name}/{version}.crate")),
            data,
        )
        .await
    }
}
