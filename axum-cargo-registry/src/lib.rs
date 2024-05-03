use axum::{routing::get, Router};
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

#[derive(Debug)]
pub struct Config {
    domain: String,
}

impl<RS> App<RS> {
    pub fn new(config: Config, registory_storage: RS) -> Self {
        Self {
            config,
            registory_storage,
        }
    }

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
    pub fn create_router(self) -> Router {
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
            .with_state(Arc::new(self))
    }
}

#[cfg(test)]
mod tests {}
