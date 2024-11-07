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

impl<RS> IndexConfig<RS> {
    pub fn has_api(&self) -> bool {
        self.app.has_api()
    }
}

impl<RS> Serialize for IndexConfig<RS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut config =
            serializer.serialize_struct("IndexConfig", 1 + if self.has_api() { 1 } else { 0 })?;
        config.serialize_field("dl", &self.app.dl_name())?;
        if self.has_api() {
            config.serialize_field("api", self.app.domain())?;
        }
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

#[cfg_attr(feature = "api", derive(Deserialize, Serialize))]
#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: String,
    pub registry: Option<String>,
    pub package: Option<String>,
}

#[cfg_attr(feature = "api", derive(Deserialize, Serialize))]
#[derive(Debug)]
pub struct IndexData {
    pub name: String,
    pub vers: String,
    pub deps: Vec<Dependency>,
    pub cksum: String,
    pub features: HashMap<String, Vec<String>>,
    pub yanked: bool,
    pub links: Option<String>,
    pub v: u32,
    pub features2: Option<HashMap<String, Vec<String>>>,
    pub rust_version: Option<String>,
}
