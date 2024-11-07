use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

pub use crate::crates::CrateInfo;
use crate::{
    registory_storage::{RegistryError, RegistryStorage},
    App,
};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub per_page: Option<usize>,
    pub page: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct MetaInfo {
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub crates: Vec<CrateInfo>,
    pub meta: MetaInfo,
}

pub async fn search_crates<RS: RegistryStorage>(
    State(state): State<Arc<App<RS>>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, RegistryError> {
    state
        .registory_storage()
        .list_crate(
            &params.q,
            params.per_page.unwrap_or(100),
            params.page.unwrap_or(1),
        )
        .await
        .map(Json)
        .map_err(Into::into)
}
