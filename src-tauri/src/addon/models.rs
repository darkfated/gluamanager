use serde::Serialize;

use crate::addon::{GithubSource, Manifest};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonView {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub repository_url: String,
    pub branch: String,
    pub preserve: Vec<String>,
    pub dependencies: Vec<GithubSource>,
    pub addon_path: String,
    pub remote_version: Option<String>,
    pub has_update: bool,
    pub has_error: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableAddonView {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub repository_url: String,
    pub branch: String,
    pub preserve: Vec<String>,
    pub dependencies: Vec<GithubSource>,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlanItem {
    pub name: String,
    pub repository_url: String,
    pub branch: String,
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
        remote_version: Option<String>,
        has_update: bool,
        has_error: bool,
        status: impl Into<String>,
    ) -> Self {
        Self {
            name: manifest.name.clone(),
            description: manifest.description.clone(),
            author: manifest.author.clone(),
            version: manifest.version.clone(),
            repository_url: manifest.github.url.clone(),
            branch: manifest.github.branch.clone(),
            preserve: manifest.preserve.clone(),
            dependencies: manifest.dependencies.clone(),
            addon_path,
            remote_version,
            has_update,
            has_error,
            status: status.into(),
        }
    }
}

impl AvailableAddonView {
    pub fn from_manifest(manifest: &Manifest, installed: bool) -> Self {
        Self {
            name: manifest.name.clone(),
            description: manifest.description.clone(),
            author: manifest.author.clone(),
            version: manifest.version.clone(),
            repository_url: manifest.github.url.clone(),
            branch: manifest.github.branch.clone(),
            preserve: manifest.preserve.clone(),
            dependencies: manifest.dependencies.clone(),
            installed,
        }
    }
}
