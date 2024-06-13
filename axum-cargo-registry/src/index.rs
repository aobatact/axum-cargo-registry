use crate::{App, RegistryStorage};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct IndexConfig<RS> {
    app: Arc<App<RS>>,
}

impl<RS> Serialize for IndexConfig<RS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut config = serializer.serialize_struct("IndexConfig", 1)?;
        config.serialize_field("dl", &self.app.dl_name())?;
        config.end()
    }
}

impl<RS> App<RS>
where
    RS: RegistryStorage,
{
    pub fn index_config(self: &Arc<Self>) -> IndexConfig<RS> {
        IndexConfig { app: self.clone() }
    }

    /// Function that is used to register to the router for `/index/config.json`
    pub async fn get_index_config_json(state: State<Arc<Self>>) -> Response {
        Json(state.index_config()).into_response()
    }

    /// Function that is used to register to the router for `/index/*crate_index_path`
    pub async fn get_crate_index(
        State(state): State<Arc<Self>>,
        Path(crate_index_path): Path<String>,
        headers: HeaderMap,
    ) -> Response {
        tracing::trace!(crate_index_path = %crate_index_path, "Getting index");
        state
            .registory_storage()
            .get_index_file(&headers, &crate_index_path)
            .await
            .into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    name: String,
    req: String,
    features: Vec<String>,
    optional: bool,
    default_features: bool,
    target: Option<String>,
    kind: String,
    registry: Option<String>,
    package: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Index {
    name: String,
    vers: String,
    deps: Vec<Dependency>,
    cksum: String,
    features: HashMap<String, Vec<String>>,
    yanked: bool,
    links: Option<String>,
    v: u32,
    features2: Option<HashMap<String, Vec<String>>>,
    rust_version: String,
}
