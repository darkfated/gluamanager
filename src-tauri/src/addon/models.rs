use serde::Serialize;

use crate::addon::Manifest;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonView {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub url: String,
    pub preserve: Vec<String>,
    pub dependencies: Vec<String>,
    pub addon_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub remote_version: Option<String>,
    pub has_update: bool,
    pub has_error: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableAddonView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub url: String,
    pub preserve: Vec<String>,
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlanItem {
    pub name: String,
    pub source_url: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlanView {
    pub root_name: String,
    pub items: Vec<InstallPlanItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadmeView {
    pub content: String,
    pub base_url: Option<String>,
    pub local_base_path: Option<String>,
}

impl AddonView {
    pub fn from_manifest(
        manifest: &Manifest,
        addon_path: String,
        source_url: Option<String>,
        remote_version: Option<String>,
        has_update: bool,
        has_error: bool,
        status: impl Into<String>,
    ) -> Self {
        Self {
            name: manifest.info.name.clone(),
            description: manifest.info.description.clone(),
            author: manifest.info.author.clone(),
            version: manifest.version.clone(),
            url: manifest.url.clone(),
            preserve: manifest.preserve.clone(),
            dependencies: manifest.dependencies.clone(),
            addon_path,
            source_url,
            remote_version,
            has_update,
            has_error,
            status: status.into(),
        }
    }
}

impl AvailableAddonView {
    pub fn from_manifest(
        manifest: &Manifest,
        source_url: Option<String>,
        installed: bool,
        id: String,
    ) -> Self {
        Self {
            id,
            name: manifest.info.name.clone(),
            description: manifest.info.description.clone(),
            author: manifest.info.author.clone(),
            version: manifest.version.clone(),
            url: manifest.url.clone(),
            preserve: manifest.preserve.clone(),
            dependencies: manifest.dependencies.clone(),
            source_url,
            installed,
        }
    }
}
