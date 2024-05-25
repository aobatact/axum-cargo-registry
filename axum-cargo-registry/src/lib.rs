use axum::{
    http::{StatusCode, Uri},
    routing::get,
    Router,
};
use registory_storage::RegistryStorage;
use std::sync::Arc;

pub mod crates;
pub mod header_util;
pub mod index;
pub mod registory_storage;

#[derive(Debug)]
pub struct App<RS> {
    config: Config,
    registory_storage: RS,
}

impl<RS> App<RS> {
    /// Create a new [`App`]
    pub fn new(config: Config, registory_storage: RS) -> Self {
        Self {
            config,
            registory_storage,
        }
    }

    /// Returns a reference to the registry storage.
    pub fn registory_storage(&self) -> &RS {
        &self.registory_storage
    }

    fn domain(&self) -> &str {
        &self.config.domain
    }

    fn dl_name(&self) -> String {
        format!("{}/crates/{{crate}}/{{version}}/download", self.domain())
    }
}

impl<RS> App<RS>
where
    RS: RegistryStorage,
{
    /// Consume and create a new [`Router`].
    pub fn create_router(self) -> Router {
        tracing::info!("Creating router");

        Router::new()
            .nest(
                "/index",
                Router::new()
                    .route("/config.json", get(Self::get_index_config_json))
                    .route("/*crate_name", get(Self::get_crate_index)),
            )
            .route(
                "/crates/:crate_name/:version/download",
                get(Self::get_crate),
            )
            .fallback(fallback)
            .with_state(Arc::new(self))
    }
}

async fn fallback(uri: Uri) -> StatusCode {
    tracing::info!(uri = %uri, "Not found");
    StatusCode::NOT_FOUND
}

#[derive(Debug)]
pub struct Config {
    /// Domain of the registry.
    pub domain: String,
}

impl Config {
    /// Create a new [`Config`]
    pub fn new(domain: String) -> Self {
        Self { domain }
    }
}

#[cfg(test)]
mod tests {}
