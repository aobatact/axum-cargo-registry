use crate::{
    crate_utils::crate_name_to_index,
    index::IndexData,
    registory_storage::{RegistryError, RegistryStorage},
    App,
};
use axum::{body::Bytes, extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};

use super::ErrorResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: String,
    pub registry: Option<String>,
    pub explicit_name_in_toml: Option<String>,
}

impl Dependency {
    pub fn to_index_dependency(self) -> crate::index::Dependency {
        crate::index::Dependency {
            name: self.name,
            req: self.version_req,
            features: self.features,
            optional: self.optional,
            default_features: self.default_features,
            target: self.target,
            kind: self.kind,
            registry: self.registry,
            package: self.explicit_name_in_toml,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub vers: String,
    pub deps: Vec<Dependency>,
    pub features: HashMap<String, Vec<String>>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub badges: HashMap<String, HashMap<String, String>>,
    pub links: Option<String>,
    pub rust_version: Option<String>,
}

type Features = (
    HashMap<String, Vec<String>>,
    Option<HashMap<String, Vec<String>>>,
);

impl Package {
    pub fn into_index(self, checksum: String) -> IndexData {
        let (features, features2) = Package::features_selector(self.features);

        IndexData {
            name: self.name,
            vers: self.vers,
            deps: self
                .deps
                .into_iter()
                .map(Dependency::to_index_dependency)
                .collect(),
            cksum: checksum,
            features,
            yanked: false,
            links: self.links,
            v: 2,
            features2,
            rust_version: self.rust_version,
        }
    }

    fn features_selector(features: HashMap<String, Vec<String>>) -> Features {
        if features.iter().any(|(_, v)| {
            v.iter()
                .any(|feat| feat.starts_with("dep") || feat.contains('?'))
        }) {
            (HashMap::new(), Some(features))
        } else {
            (features, None)
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Warnings {
    pub invalid_categories: Vec<String>,
    pub invalid_badges: Vec<String>,
    pub other: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PublishResponse {
    pub warnings: Warnings,
}

impl<RS: RegistryStorage> App<RS> {
    pub async fn publish(
        State(state): State<Arc<Self>>,
        data: Bytes,
    ) -> Result<Json<PublishResponse>, ErrorResponse> {
        let (pacakge, dot_crate) = parse_publish_data(&data).map_err(|e| {
            tracing::trace!("Failed to parse publish data: {:?}", e);
            (StatusCode::BAD_REQUEST, RegistryError::new(e))
        })?;

        let prev_index_array = state.get_index_data(&pacakge).await?;
        tracing::trace!(prev_index_array = ?prev_index_array, "Got index data");
        if prev_index_array.iter().any(|v| v.vers == pacakge.vers) {
            Err((StatusCode::CONFLICT, RegistryError::Duplicate))?;
        }

        let mut hasher = Sha256::new();
        hasher.update(dot_crate);
        let sha_hash = hasher.finalize();

        let crate_name = pacakge.name.clone();
        let version = pacakge.vers.clone();
        let index = pacakge.into_index(format!("{:x}", sha_hash));

        tracing::trace!(crate_name = %crate_name, version = %version, "Save to index");
        state
            .registory_storage
            .put_index(&crate_name_to_index(&crate_name), index, prev_index_array)
            .await
            .map_err(|e| {
                tracing::trace!("Failed to put index: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e)
            })?;

        tracing::trace!(crate_name = %crate_name, version = %version, "Save to crates");
        state
            .registory_storage
            .put_crate(&crate_name, &version, dot_crate)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        Ok(Json(PublishResponse {
            warnings: Warnings::default(),
        }))
    }

    pub async fn get_index_data(
        self: &Arc<Self>,
        pacakge: &Package,
    ) -> Result<Vec<IndexData>, (StatusCode, RegistryError)> {
        let res = self.registory_storage.get_index_data(&pacakge.name).await;
        match res {
            Ok(Some(vec)) => {
                if vec.iter().any(|index| index.vers == pacakge.vers) {
                    Err((StatusCode::CONFLICT, RegistryError::Duplicate))
                } else {
                    Ok(vec)
                }
            }
            Ok(None) => Ok(Vec::new()),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
        }
    }
}

pub fn parse_publish_data(
    data: &[u8],
) -> Result<(Package, &[u8]), Box<dyn std::error::Error + Send + Sync>> {
    const ERROR_MESSAGE: &str = "Invalid Data";
    let (len_bytes, data) = data.split_first_chunk().ok_or(ERROR_MESSAGE)?;

    let len = u32::from_le_bytes(*len_bytes) as usize;
    let package = data.get(0..len).ok_or(ERROR_MESSAGE)?;
    let package: Package = serde_json::from_slice(package)?;
    tracing::trace!(package = ?package, "Parsed package");
    let (len_bytes, data) = data
        .get(len..)
        .ok_or(ERROR_MESSAGE)?
        .split_first_chunk()
        .ok_or(ERROR_MESSAGE)?;
    let crate_len = u32::from_le_bytes(*len_bytes) as usize;
    let crate_data = data.get(0..crate_len).ok_or(ERROR_MESSAGE)?;
    Ok((package, crate_data))
}
