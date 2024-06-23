use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::future::Future;

#[cfg(feature = "api")]
use crate::index::IndexData;

#[cfg(feature = "storage-local")]
pub mod local;
#[cfg(feature = "storage-s3")]
pub mod s3;

/// Trait that defines the interface for the storage backend
pub trait RegistryStorage: Send + Sync + 'static {
    /// Get the index file
    fn get_index_file(
        &self,
        headers: &HeaderMap,
        index_path: &str,
    ) -> impl Future<Output = impl IntoResponse> + Send;
    /// Get a crate file
    fn get_crate_file(
        &self,
        headers: &HeaderMap,
        crate_name: &str,
        version: &str,
    ) -> impl Future<Output = impl IntoResponse> + Send;

    #[cfg(feature = "api")]
    /// Get the index data for the given crate.
    fn get_index_data(
        &self,
        crate_name: &str,
    ) -> impl Future<Output = Result<Option<Vec<IndexData>>, RegistryError>> + Send
    where
        Self: Sized;
    #[cfg(feature = "api")]
    /// Put the index file
    fn put_index(
        &self,
        index_path: &str,
        data: IndexData,
        prev_data: Vec<IndexData>,
    ) -> impl Future<Output = Result<(), RegistryError>> + Send;

    #[cfg(feature = "api")]
    /// Put a crate file
    fn put_crate(
        &self,
        crate_name: &str,
        version: &str,
        data: &[u8],
    ) -> impl Future<Output = Result<(), RegistryError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Not Found")]
    NotFound,
    #[error("Duplicate")]
    Duplicate,
    #[error("Reqwest Deserialize error {0}")]
    ReqwestDe(serde_json::Error),
    #[error("Serde error {0}")]
    SerDeOther(serde_json::Error),
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl RegistryError {
    pub fn new<E: Into<Box<dyn std::error::Error + Send + Sync>>>(e: E) -> Self {
        RegistryError::Other(e.into())
    }
}

impl IntoResponse for RegistryError {
    fn into_response(self) -> axum::response::Response {
        match self {
            RegistryError::NotFound => StatusCode::NOT_FOUND.into_response(),
            RegistryError::ReqwestDe(_) => StatusCode::BAD_REQUEST.into_response(),
            RegistryError::Duplicate => StatusCode::CONFLICT.into_response(),
            e => {
                tracing::debug!("{e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}
