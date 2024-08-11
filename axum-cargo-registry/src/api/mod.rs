use crate::{
    registory_storage::{RegistryError, RegistryStorage},
    App,
};
use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod publish;
pub mod search;
pub mod yank;

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

impl ErrorResponse {
    pub fn crate_not_found() -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            errors: vec![ErrorResponseInner {
                detail: "Crate not found".into(),
            }],
        }
    }
    pub fn version_not_found() -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            errors: vec![ErrorResponseInner {
                detail: "Version not found".into(),
            }],
        }
    }
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
impl From<RegistryError> for ErrorResponse {
    fn from(err: RegistryError) -> Self {
        let code = match &err {
            RegistryError::NotFound => StatusCode::NOT_FOUND,
            RegistryError::Duplicate => StatusCode::CONFLICT,
            RegistryError::ReqwestDe(_) => StatusCode::BAD_REQUEST,
            RegistryError::SerDeOther(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RegistryError::NotSupported => StatusCode::NOT_IMPLEMENTED,
            RegistryError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, err).into()
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, axum::Json(self)).into_response()
    }
}

impl<RS: RegistryStorage> App<RS> {
    pub fn api_nest() -> axum::Router<Arc<Self>> {
        axum::Router::new().route("/v1/crates/new", axum::routing::put(Self::publish))
    }
}
