use crate::{App, RegistryStorage};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

impl<RS> App<RS>
where
    RS: RegistryStorage,
{
    /// Function that is used to register to the router for `/index/config.json`
    pub async fn get_crate(
        State(state): State<Arc<Self>>,
        Path((crate_name, version)): Path<(String, String)>,
        headers: HeaderMap,
    ) -> Response {
        tracing::trace!(crate_name = %crate_name, version = %version, "Getting crate");
        state
            .registory_storage()
            .get_crate_file(&headers, &crate_name, &version)
            .await
            .into_response()
    }
}
