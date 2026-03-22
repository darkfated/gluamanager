use std::sync::OnceLock;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, CACHE_CONTROL, PRAGMA, USER_AGENT};
use serde::Deserialize;

use crate::addon::{parse_github_url, GithubSource, Manifest, MANIFEST_NAME};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct RepoRef {
    pub owner: String,
    pub repo: String,
    pub branch: String,
}

#[derive(Debug, Clone)]
pub struct RemoteManifest {
    pub repo: RepoRef,
    pub manifest: Manifest,
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SourceRepository {
    Url(String),
    Repo { url: String, branch: String },
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .default_headers(default_headers())
            .timeout(Duration::from_secs(20))
            .build()
            .expect("failed to create reqwest client")
    })
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("GLuaManager"));
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers
}

pub fn resolve_repo(manifest: &Manifest) -> AppResult<RepoRef> {
    if manifest.github.url.trim().is_empty() {
        return Err(AppError::Unexpected(
            "В .addon не заполнено поле github.url.".into(),
        ));
    }
    if manifest.github.branch.trim().is_empty() {
        return Err(AppError::Unexpected(
            "В .addon не заполнено поле github.branch.".into(),
        ));
    }

    let (owner, repo) = parse_github_url(&manifest.github.url)?;
    Ok(RepoRef {
        owner,
        repo,
        branch: manifest.github.branch.trim().to_string(),
    })
}

pub async fn load_source_repositories(source_url: &str) -> AppResult<Vec<RepoRef>> {
    let response = client().get(source_url).send().await?;

    if !response.status().is_success() {
        return Err(AppError::Unexpected(format!(
            "Не удалось получить источник {}. HTTP {}.",
            source_url,
            response.status()
        )));
    }

    let entries: Vec<SourceRepository> = response.json().await?;
    let mut repositories = Vec::new();
    for entry in entries {
        repositories.push(resolve_source_repository(entry)?);
    }

    Ok(repositories)
}

pub async fn fetch_remote_manifest_from_repo_url(
    repository_url: &str,
    branch: &str,
) -> AppResult<RemoteManifest> {
    let repo = resolve_repo_from_url(repository_url, branch)?;
    fetch_remote_manifest_by_repo(repo).await
}

pub async fn fetch_remote_manifest(manifest: &Manifest) -> AppResult<RemoteManifest> {
    let repo = resolve_repo(manifest)?;
    fetch_remote_manifest_by_repo(repo).await
}

pub async fn fetch_readme(manifest: &Manifest) -> AppResult<Option<String>> {
    let repo = resolve_repo(manifest)?;
    fetch_readme_by_repo(&repo).await
}

pub async fn fetch_readme_from_repo_url(
    repository_url: &str,
    branch: &str,
) -> AppResult<Option<String>> {
    let repo = resolve_repo_from_url(repository_url, branch)?;
    fetch_readme_by_repo(&repo).await
}

pub async fn download_repository_archive(remote: &RemoteManifest) -> AppResult<Vec<u8>> {
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        remote.repo.owner, remote.repo.repo, remote.repo.branch
    );

    let response = client().get(url).send().await?;
    ensure_success(
        response.status(),
        format!("Не удалось скачать архив ветки {}.", remote.repo.branch),
    )?;

    Ok(response.bytes().await?.to_vec())
}

fn resolve_repo_from_url(repository_url: &str, branch: &str) -> AppResult<RepoRef> {
    let (owner, repo) = parse_github_url(repository_url)?;
    let branch = branch.trim().to_string();
    if branch.is_empty() {
        return Err(AppError::Unexpected(format!(
            "Для репозитория {} не указана ветка.",
            repository_url
        )));
    }

    Ok(RepoRef {
        owner,
        repo,
        branch,
    })
}

fn resolve_source_repository(entry: SourceRepository) -> AppResult<RepoRef> {
    let github = match entry {
        SourceRepository::Url(url) => GithubSource {
            url,
            branch: String::new(),
        },
        SourceRepository::Repo { url, branch } => GithubSource { url, branch },
    };

    if github.url.trim().is_empty() {
        return Err(AppError::Unexpected(
            "В источнике не заполнено поле url.".into(),
        ));
    }

    if github.branch.trim().is_empty() {
        return Err(AppError::Unexpected(format!(
            "Для репозитория {} в источнике не заполнено поле branch.",
            github.url
        )));
    }

    let (owner, repo) = parse_github_url(&github.url)?;
    Ok(RepoRef {
        owner,
        repo,
        branch: github.branch.trim().to_string(),
    })
}

async fn fetch_remote_manifest_by_repo(repo: RepoRef) -> AppResult<RemoteManifest> {
    let raw = fetch_manifest_bytes(&repo).await?;
    let mut manifest = Manifest::load_from_slice(&raw)?;
    if manifest.github.url.trim().is_empty() {
        manifest.github.url = format!("https://github.com/{}/{}", repo.owner, repo.repo);
    }
    if manifest.github.branch.trim().is_empty() {
        manifest.github.branch = repo.branch.clone();
    }

    Ok(RemoteManifest {
        repo,
        manifest,
        raw,
    })
}

async fn fetch_manifest_bytes(repo: &RepoRef) -> AppResult<Vec<u8>> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        repo.owner, repo.repo, repo.branch, MANIFEST_NAME
    );

    let response = client().get(url).send().await?;
    ensure_success(
        response.status(),
        format!(
            "Не удалось получить удалённый .addon по ветке {}.",
            repo.branch
        ),
    )?;

    Ok(response.bytes().await?.to_vec())
}

async fn fetch_readme_by_repo(repo: &RepoRef) -> AppResult<Option<String>> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/README.md",
        repo.owner, repo.repo, repo.branch
    );

    let response = client().get(url).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    ensure_success(
        response.status(),
        format!("Не удалось получить README.md по ветке {}.", repo.branch),
    )?;

    let bytes = response.bytes().await?;
    let content = String::from_utf8_lossy(&bytes).into_owned();
    Ok(Some(content))
}

fn ensure_success(status: reqwest::StatusCode, context: String) -> AppResult<()> {
    if status.is_success() {
        return Ok(());
    }

    Err(AppError::Unexpected(format!(
        "{context} GitHub вернул {}.",
        status
    )))
}
