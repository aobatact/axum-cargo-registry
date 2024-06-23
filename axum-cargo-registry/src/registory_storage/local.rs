//! Local storage backend for the registry.

use super::{RegistryError, RegistryStorage};
use crate::{
    crate_utils::crate_name_to_index,
    header_util::{self, get_modified_since},
};
use axum::{
    body::Bytes,
    http::StatusCode,
    response::{AppendHeaders, IntoResponse, Response},
};
use std::{
    io::{BufRead, Read, Write},
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

    #[cfg(feature = "api")]
    async fn get_index_data(
        &self,
        crate_name: &str,
    ) -> Result<Option<Vec<crate::index::IndexData>>, RegistryError>
    where
        Self: Sized,
    {
        let path = crate_name_to_index(crate_name);
        tracing::trace!(crate_name, path, "Getting index data");
        match std::fs::read(self.index_path.join(path)) {
            Ok(data) => {
                let items = data
                    .lines()
                    .map(|line| {
                        tracing::trace!(line = ?line, "Parsing line");
                        line.map_err(RegistryError::new)
                            .map(|s| serde_json::from_str(&s).unwrap())
                            // .and_then(|s| serde_json::from_str(&s).map_err(RegistryError::new))
                            .and_then(|s| {
                                tracing::trace!(s = ?s, "Parsed line");
                                serde_json::from_value(s).map_err(RegistryError::new)
                            })
                    })
                    .collect();
                Some(items).transpose()
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(RegistryError::new(e)),
        }
    }

    #[cfg(feature = "api")]
    async fn put_index(
        &self,
        index_path: &str,
        data: crate::index::IndexData,
        mut prev_data: Vec<crate::index::IndexData>,
    ) -> Result<(), RegistryError> {
        let path: PathBuf = self.index_path.join(index_path);
        let dir = path
            .parent()
            .ok_or_else(|| RegistryError::new("Invalid path"))?;
        tracing::trace!(dir = ?std::path::absolute(dir), "Creating directory");
        std::fs::create_dir_all(dir).map_err(RegistryError::new)?;
        let mut file = match std::fs::File::options()
            .write(true)
            .append(true)
            .create(true)
            .open(&path)
        {
            Ok(f) => f,
            Err(e) => {
                tracing::error!(?e, path = %path.display(), "Failed to open file");
                return Err(RegistryError::new(e));
            }
        };
        // prev_data.push(data);
        // for prev in prev_data {
        file.write_all(&serde_json::to_vec(&data).unwrap())
            .map_err(RegistryError::new)?;
        file.write_all(b"\n").map_err(RegistryError::new)?;

        Ok(())
    }

    #[cfg(feature = "api")]
    async fn put_crate(
        &self,
        crate_name: &str,
        version: &str,
        data: &[u8],
    ) -> Result<(), RegistryError> {
        let path = self
            .crate_path
            .join(format!("{crate_name}/{version}.crate"));
        let dir = path
            .parent()
            .ok_or_else(|| RegistryError::new("Invalid path"))?;
        std::fs::create_dir_all(dir).map_err(RegistryError::new)?;
        let mut file = match std::fs::File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(f) => f,
            Err(e) => {
                tracing::error!(?e, path = %path.display(), "Failed to open file");
                return Err(RegistryError::new(e));
            }
        };
        if let Err(e) = file.write_all(data) {
            tracing::error!(?e, path = %path.display(), "Failed to write to file");
            return Err(RegistryError::new(e));
        }
        Ok(())
    }
}
