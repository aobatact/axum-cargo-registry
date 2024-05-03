use axum::{http::HeaderMap, response::IntoResponse};
use std::future::Future;

#[cfg(feature = "storage-local")]
pub mod local;
#[cfg(feature = "storage-s3")]
pub mod s3;

pub trait RegistryStorage: Send + Sync + 'static {
    fn get_index(
        &self,
        headers: &HeaderMap,
        index_path: &str,
    ) -> impl Future<Output = impl IntoResponse> + Send;
    fn get_crate(
        &self,
        headers: &HeaderMap,
        crate_name: &str,
        version: &str,
    ) -> impl Future<Output = impl IntoResponse> + Send;
}
