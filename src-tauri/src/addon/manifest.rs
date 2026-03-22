use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, AppResult};

pub const MANIFEST_NAME: &str = ".addon";

#[derive(Debug, Clone, Serialize)]
pub struct Manifest {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub github: GithubSource,
    pub preserve: Vec<String>,
    pub dependencies: Vec<GithubSource>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GithubSource {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub branch: String,
}

#[derive(Debug, Deserialize)]
struct RawManifest {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    github: GithubSource,
    #[serde(default)]
    preserve: Vec<String>,
    #[serde(default)]
    dependencies: Vec<GithubSource>,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

impl Manifest {
    pub fn load(path: &Path) -> AppResult<Self> {
        let content = fs::read_to_string(path)?;
        Self::load_from_str(&content)
    }

    pub fn load_from_slice(content: &[u8]) -> AppResult<Self> {
        Self::from_raw(serde_json::from_slice::<RawManifest>(content)?)
    }

    pub fn load_from_str(content: &str) -> AppResult<Self> {
        Self::from_raw(serde_json::from_str::<RawManifest>(content)?)
    }

    fn from_raw(raw: RawManifest) -> AppResult<Self> {
        let manifest = Self {
            name: raw.name,
            description: raw.description,
            author: raw.author,
            version: raw.version,
            github: raw.github,
            preserve: raw.preserve,
            dependencies: raw.dependencies,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> AppResult<()> {
        if !self.github.url.trim().is_empty() {
            parse_github_url(&self.github.url)?;
        }

        Ok(())
    }
}

pub fn parse_github_url(raw: &str) -> AppResult<(String, String)> {
    let value = raw.trim();
    let parts = value.split('/').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(AppError::InvalidGithubUrl(
            "Поле github.url должно быть в формате username/repo.".into(),
        ));
    }
    if value.contains("://") || value.starts_with("github.com/") {
        return Err(AppError::InvalidGithubUrl(
            "Поле github.url должно быть в формате username/repo.".into(),
        ));
    }

    Ok((
        parts[0].to_string(),
        parts[1].trim_end_matches(".git").to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{parse_github_url, Manifest};

    #[test]
    fn parses_current_manifest_schema() {
        let manifest = Manifest::load_from_str(
            r#"{
                "name": "Modern Addon",
                "description": "Description",
                "author": "Author",
                "version": "1.0.0",
                "github": {
                    "url": "username/test-addon",
                    "branch": "main"
                },
                "preserve": ["data", "cfg/server.cfg"]
            }"#,
        )
        .expect("manifest");

        assert_eq!(manifest.name, "Modern Addon");
        assert_eq!(manifest.description, "Description");
        assert_eq!(manifest.author, "Author");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.github.url, "username/test-addon");
        assert_eq!(manifest.github.branch, "main");
        assert_eq!(manifest.preserve, vec!["data", "cfg/server.cfg"]);
        assert!(manifest.dependencies.is_empty());

        let (owner, repo) = parse_github_url(&manifest.github.url).expect("url");
        assert_eq!(owner, "username");
        assert_eq!(repo, "test-addon");
    }

    #[test]
    fn parses_dependencies() {
        let manifest = Manifest::load_from_str(
            r#"{
                "name": "Depends",
                "version": "1.0.0",
                "github": {
                    "url": "username/root-addon",
                    "branch": "main"
                },
                "dependencies": [
                    {
                        "url": "username/lib-one",
                        "branch": "main"
                    },
                    {
                        "url": "username/lib-two",
                        "branch": "master"
                    }
                ]
            }"#,
        )
        .expect("manifest");

        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.dependencies[0].url, "username/lib-one");
        assert_eq!(manifest.dependencies[1].branch, "master");
    }
}
