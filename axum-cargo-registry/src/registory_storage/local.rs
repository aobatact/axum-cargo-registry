use crate::header_util::{self, get_modified_since};

use super::RegistryStorage;
use axum::{
    body::Bytes,
    http::StatusCode,
    response::{AppendHeaders, IntoResponse, Response},
};
use std::{io::Read, path::PathBuf, time::SystemTime};

pub struct LocalStorage {
    index_path: PathBuf,
    crate_path: PathBuf,
}

impl LocalStorage {
    async fn get_local_file(path: PathBuf, last_modified: Option<SystemTime>) -> Response {
        fn inner(
            path: PathBuf,
            last_modified: Option<SystemTime>,
        ) -> Result<Option<(Vec<u8>, SystemTime)>, std::io::Error> {
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
        let (vec, time) = match inner(path, last_modified) {
            Ok(Some(f)) => f,
            Ok(None) => {
                todo!()
            }
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
    fn get_index(
        &self,
        headers: &axum::http::HeaderMap,
        index_path: &str,
    ) -> impl std::future::Future<Output = impl axum::response::IntoResponse> + Send {
        let last = get_modified_since(headers);
        Self::get_local_file(self.index_path.join(index_path), last)
    }

    fn get_crate(
        &self,
        headers: &axum::http::HeaderMap,
        crate_name: &str,
        version: &str,
    ) -> impl std::future::Future<Output = impl axum::response::IntoResponse> + Send {
        let last = get_modified_since(headers);
        let mut path = self.crate_path.clone();
        path.push(format!("{crate_name}/{version}.crate"));
        Self::get_local_file(self.crate_path.join(path), last)
    }
}
