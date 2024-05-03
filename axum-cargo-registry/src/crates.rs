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
    pub async fn get_crate(
        State(state): State<Arc<Self>>,
        Path((crate_name, version)): Path<(String, String)>,
        headers: HeaderMap,
    ) -> Response {
        state
            .registory_storage()
            .get_crate(&headers, &crate_name, &version)
            .await
            .into_response()
    }
}
