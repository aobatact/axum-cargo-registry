use crate::{
    registory_storage::{RegistryError, RegistryStorage},
    App,
};
use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponseInner {
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(skip)]
    pub status_code: StatusCode,
    pub errors: Vec<ErrorResponseInner>,
}

impl From<(StatusCode, RegistryError)> for ErrorResponse {
    fn from(err: (StatusCode, RegistryError)) -> Self {
        if err.0.is_server_error() {
            tracing::error!(error = %err.1, "Internal server error");
        } else {
            tracing::trace!(error = %err.1, "Client error");
        }
        ErrorResponse {
            status_code: err.0,
            errors: vec![ErrorResponseInner {
                detail: err.1.to_string(),
            }],
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, axum::Json(self)).into_response()
    }
}

pub mod publish;

impl<RS: RegistryStorage> App<RS> {
    pub fn api_nest() -> axum::Router<Arc<Self>> {
        axum::Router::new().route("/v1/crates/new", axum::routing::put(Self::publish))
    }
}
