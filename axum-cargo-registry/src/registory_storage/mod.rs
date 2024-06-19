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
    #[cfg(feature = "api")]
    fn get_index_data(
        &self,
        crate_name: &str,
    ) -> impl futures_util::TryStream<Ok = IndexData, Error = RegistryStorageError> + Send
    where
        Self: Sized;
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
}

#[cfg(feature = "api")]
pub trait WritableRegistryStorage: RegistryStorage {
    /// Put the index file
    fn put_index(
        &self,
        index_path: &str,
        data: IndexData,
        prev_data: Vec<IndexData>,
    ) -> impl Future<Output = Result<(), axum::Error>> + Send;

    /// Put a crate file
    fn put_crate(
        &self,
        crate_name: &str,
        version: &str,
        data: &[u8],
    ) -> impl Future<Output = Result<(), axum::Error>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryStorageError {
    #[error("Not Found")]
    NotFound,
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl RegistryStorageError {
    pub fn new<E: Into<Box<dyn std::error::Error + Send + Sync>>>(e: E) -> Self {
        RegistryStorageError::Other(e.into())
    }
}

impl IntoResponse for RegistryStorageError {
    fn into_response(self) -> axum::response::Response {
        match self {
            RegistryStorageError::NotFound => StatusCode::NOT_FOUND.into_response(),
            RegistryStorageError::Other(e) => {
                tracing::error!("{:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
