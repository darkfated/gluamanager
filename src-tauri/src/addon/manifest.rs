use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{AppError, AppResult};

pub const MANIFEST_NAME: &str = ".addon";

#[derive(Debug, Clone, Serialize)]
pub struct Manifest {
    pub info: Info,
    pub version: String,
    pub url: String,
    #[serde(default)]
    pub preserve: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
}

#[derive(Debug, Deserialize)]
struct RawManifest {
    #[serde(default)]
    info: RawInfo,
    #[serde(default)]
    version: String,
    url: String,
    #[serde(default)]
    preserve: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawInfo {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    author: String,
}

impl Manifest {
    pub fn load(path: &Path) -> AppResult<Self> {
        let content = fs::read_to_string(path)?;
        Self::load_from_str(&content)
    }

    pub fn load_from_str(content: &str) -> AppResult<Self> {
        Self::from_raw(serde_json::from_str::<RawManifest>(content)?)
    }

    pub fn load_from_url(content: &[u8]) -> AppResult<Self> {
        Self::from_raw(serde_json::from_slice::<RawManifest>(content)?)
    }

    fn from_raw(raw: RawManifest) -> AppResult<Self> {
        let manifest = Self {
            info: Info {
                name: raw.info.name,
                description: raw.info.description,
                author: raw.info.author,
            },
            version: raw.version,
            url: raw.url,
            preserve: raw.preserve,
            dependencies: raw.dependencies,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> AppResult<()> {
        if self.info.name.trim().is_empty() {
            return Err(AppError::Unexpected(
                "Manifest is missing info.name.".into(),
            ));
        }

        validate_url(&self.url, "url")?;

        for dependency in &self.dependencies {
            validate_url(dependency, "dependency")?;
        }

        Ok(())
    }
}

fn validate_url(value: &str, field: &str) -> AppResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Unexpected(format!(
            "Manifest is missing {field}."
        )));
    }

    let parsed = Url::parse(trimmed)
        .map_err(|error| AppError::Unexpected(format!("Invalid {field} URL: {error}")))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => Err(AppError::Unexpected(format!(
            "Invalid {field} URL: only http and https schemes are supported."
        ))),
    }
}
