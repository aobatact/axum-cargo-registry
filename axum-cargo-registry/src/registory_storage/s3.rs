//! S3 storage backend for the registry.

use super::RegistryStorage;
use crate::header_util::get_if_none_match;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{error::SdkError, presigning::PresigningConfig};
use axum::{
    http::{HeaderMap, StatusCode},
    response::{AppendHeaders, IntoResponse, Redirect, Response},
};
use std::time::Duration;

/// S3 storage backend
#[derive(Debug)]
pub struct S3RegistoryStorage {
    client: aws_sdk_s3::Client,
    config: StorageConfig,
}

/// Configuration for the S3 storage backend

#[derive(Debug)]
pub struct StorageConfig {
    index_bucket: String,
    index_prefix: String,
    crate_bucket: String,
    crate_prefix: String,
    presigned_expire: Duration,
}

impl StorageConfig {
    /// Create a new [`StorageConfig`]
    pub fn new(
        index_bucket: String,
        mut index_prefix: String,
        crate_bucket: String,
        mut crate_prefix: String,
        presigned_expire: Duration,
    ) -> Self {
        if !index_prefix.ends_with('/') {
            if !index_prefix.is_empty() {
                index_prefix.push('/')
            }
        }
        if !crate_prefix.ends_with('/') {
            if !crate_prefix.is_empty() {
                crate_prefix.push('/')
            }
        }

        Self {
            index_bucket,
            index_prefix,
            crate_bucket,
            crate_prefix,
            presigned_expire,
        }
    }

    /// Create a new [`StorageConfig`] from environment variables.
    /// Environment variables `INDEX_BUCKET`, `CRATE_BUCKET`, `PRESIGNED_EXPIRE` are required.
    /// `INDEX_PREFIX` and `CRATE_PREFIX` are optional.
    pub fn from_env() -> Result<Self, Error> {
        const INDEX_BUCKET: &str = "INDEX_BUCKET";
        const INDEX_PREFIX: &str = "INDEX_PREFIX";
        const CRATE_BUCKET: &str = "CRATE_BUCKET";
        const CRATE_PREFIX: &str = "CRATE_PREFIX";
        const PRESIGNED: &str = "PRESIGNED_EXPIRE";

        let index_bucket = std::env::var(INDEX_BUCKET).map_err(|_| Error::Env(INDEX_BUCKET))?;
        let index_prefix = std::env::var(INDEX_PREFIX).unwrap_or_default();
        let crate_bucket = std::env::var(CRATE_BUCKET).map_err(|_| Error::Env(CRATE_PREFIX))?;
        let crate_prefix = std::env::var(CRATE_PREFIX).unwrap_or_default();
        let presigned_expire = std::env::var(PRESIGNED)
            .ok()
            .and_then(|x| Some(Duration::from_secs(x.parse().ok()?)))
            .ok_or(Error::Env(PRESIGNED))?;

        Ok(Self::new(
            index_bucket,
            index_prefix,
            crate_bucket,
            crate_prefix,
            presigned_expire,
        ))
    }
}

/// Error type for S3RegistoryStorage
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Environment value {0} invalid or not found.")]
    Env(&'static str),
    #[error("Presigned duration too long")]
    PresignedExpireDurationError,
}

impl S3RegistoryStorage {
    /// Create a new [`S3RegistoryStorage`]
    pub fn new(client: aws_sdk_s3::Client, config: StorageConfig) -> Self {
        Self { client, config }
    }

    /// Create a new [`S3RegistoryStorage`] from environment variables.
    pub async fn from_env() -> Result<Self, Error> {
        let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
            .load()
            .await;
        let client = aws_sdk_s3::Client::new(&config);

        Ok(Self::new(client, StorageConfig::from_env()?))
    }

    fn presigned_config(&self) -> PresigningConfig {
        PresigningConfig::builder()
            .expires_in(self.config.presigned_expire)
            .build()
            .unwrap()
    }

    /// Create a presigned request for S3
    pub async fn create_get_presigned_request(
        &self,
        headers: &HeaderMap,
        bucket_name: String,
        object_key: String,
    ) -> Response {
        tracing::debug!(bucket_name, object_key);
        let result = self
            .client
            .get_object()
            .bucket(bucket_name)
            .key(object_key)
            .set_if_none_match(get_if_none_match(&headers))
            .presigned(self.presigned_config())
            .await;
        match result {
            Ok(out) => {
                let uri = out.uri();
                tracing::trace!(uri, "presigned");
                ((AppendHeaders(out.headers()), Redirect::temporary(uri))).into_response()
            }
            Err(e) => match e {
                SdkError::ResponseError(e) => {
                    let raw = e.into_raw();
                    let status = raw.status();
                    if status.is_server_error() {
                        tracing::error!("ResponseError ({}): {:?}", status, raw);
                    } else {
                        tracing::debug!("ResponseError ({}): {:?}", status, raw);
                    }
                    (
                        StatusCode::from_u16(status.as_u16())
                            .expect("Unknown status code from S3."),
                        format!("{raw:?}"),
                    )
                        .into_response()
                }
                other => match other.into_service_error() {
                    aws_sdk_s3::operation::get_object::GetObjectError::NoSuchKey(_) => {
                        tracing::debug!("Key not found");
                        StatusCode::NOT_FOUND.into_response()
                    }
                    e => {
                        tracing::debug!("{e:?}");
                        (StatusCode::BAD_REQUEST, e.to_string()).into_response()
                    }
                },
            },
        }
    }
}

impl RegistryStorage for S3RegistoryStorage {
    fn get_index(
        &self,
        headers: &axum::http::HeaderMap,
        index_path: &str,
    ) -> impl std::future::Future<Output = impl axum::response::IntoResponse> + Send {
        self.create_get_presigned_request(
            headers,
            self.config.index_bucket.clone(),
            format!("{}{index_path}", self.config.index_prefix),
        )
    }

    fn get_crate(
        &self,
        headers: &axum::http::HeaderMap,
        crate_name: &str,
        version: &str,
    ) -> impl std::future::Future<Output = impl axum::response::IntoResponse> + Send {
        self.create_get_presigned_request(
            headers,
            self.config.crate_bucket.clone(),
            format!("{}{crate_name}/{version}", self.config.crate_prefix),
        )
    }
}
