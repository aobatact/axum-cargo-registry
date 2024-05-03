use crate::{App, RegistryStorage};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

impl<RS> App<RS>
where
    RS: RegistryStorage,
{
    pub async fn get_index_config_json(state: State<Arc<Self>>) -> Json<Value> {
        Json(json!({ "dl": state.dl_name() }))
    }

    pub async fn get_crate_index(
        State(state): State<Arc<Self>>,
        Path(crate_index_path): Path<String>,
        headers: HeaderMap,
    ) -> Response {
        state
            .registory_storage()
            .get_index(&headers, &crate_index_path)
            .await
            .into_response()
    }
}
