use super::ErrorResponse;
use crate::{registory_storage::RegistryStorage, App};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct OkResponse {
    ok: bool,
}

impl OkResponse {
    fn success<E>() -> Result<Json<Self>, E> {
        Ok(Json(Self { ok: true }))
    }
}

impl<RS: RegistryStorage> App<RS> {
    pub(crate) async fn yank_base(
        State(state): State<Arc<Self>>,
        Path((crate_name, version)): Path<(String, String)>,
        yank: bool,
    ) -> Result<Json<OkResponse>, ErrorResponse> {
        let Some(mut data) = state
            .registory_storage()
            .get_index_data(&crate_name)
            .await?
        else {
            return Err(ErrorResponse::crate_not_found());
        };
        if let Some(index) = data.iter_mut().find(|index| index.vers == version) {
            if index.yanked ^ yank {
                index.yanked = yank;
                state
                    .registory_storage()
                    .put_all_index(&crate_name, data)
                    .await?;
            }

            OkResponse::success()
        } else {
            Err(ErrorResponse::version_not_found())
        }
    }

    pub async fn yank(
        state: State<Arc<Self>>,
        path: Path<(String, String)>,
    ) -> Result<Json<OkResponse>, ErrorResponse> {
        Self::yank_base(state, path, true).await
    }

    pub async fn unyank(
        state: State<Arc<Self>>,
        path: Path<(String, String)>,
    ) -> Result<Json<OkResponse>, ErrorResponse> {
        Self::yank_base(state, path, false).await
    }
}
