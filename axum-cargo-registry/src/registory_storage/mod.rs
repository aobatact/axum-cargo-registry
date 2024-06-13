use axum::{http::HeaderMap, response::IntoResponse, Error};
use std::future::Future;

use crate::index::Index;

#[cfg(feature = "storage-local")]
pub mod local;
#[cfg(feature = "storage-s3")]
pub mod s3;

/// Trait that defines the interface for the storage backend
pub trait RegistryStorage: Send + Sync + 'static {
    fn get_index_data(
        &self,
        crate_name: &str,
    ) -> impl futures_util::TryStream<Ok = Index, Error = Error> + Send
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

pub trait WritableRegistryStorage: RegistryStorage {
    /// Put the index file
    fn put_index(
        &self,
        headers: &HeaderMap,
        index_path: &str,
        data: Vec<u8>,
    ) -> impl Future<Output = impl IntoResponse> + Send;

    /// Put a crate file
    fn put_crate(
        &self,
        headers: &HeaderMap,
        crate_name: &str,
        version: &str,
        data: Vec<u8>,
    ) -> impl Future<Output = impl IntoResponse> + Send;
}
