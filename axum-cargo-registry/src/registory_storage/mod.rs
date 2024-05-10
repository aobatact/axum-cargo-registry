use axum::{http::HeaderMap, response::IntoResponse};
use std::future::Future;

#[cfg(feature = "storage-local")]
pub mod local;
#[cfg(feature = "storage-s3")]
pub mod s3;

/// Trait that defines the interface for the storage backend
pub trait RegistryStorage: Send + Sync + 'static {
    /// Get the index file
    fn get_index(
        &self,
        headers: &HeaderMap,
        index_path: &str,
    ) -> impl Future<Output = impl IntoResponse> + Send;
    /// Get a crate file
    fn get_crate(
        &self,
        headers: &HeaderMap,
        crate_name: &str,
        version: &str,
    ) -> impl Future<Output = impl IntoResponse> + Send;
}
