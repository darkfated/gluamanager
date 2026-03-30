use std::sync::OnceLock;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, CACHE_CONTROL, PRAGMA, USER_AGENT};

use crate::addon::Manifest;
use crate::error::{AppError, AppResult};

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

pub async fn fetch_manifest_from_url(manifest_url: &str) -> AppResult<Manifest> {
    let response = client().get(manifest_url).send().await?;
    ensure_success(
        response.status(),
        format!("Failed to fetch metadata from {}.", manifest_url),
    )?;

    let bytes = response.bytes().await?;
    Manifest::load_from_url(&bytes)
}

pub async fn download_archive_from_url(archive_url: &str) -> AppResult<Vec<u8>> {
    let response = client().get(archive_url).send().await?;
    ensure_success(
        response.status(),
        format!("Failed to download archive from {}.", archive_url),
    )?;

    Ok(response.bytes().await?.to_vec())
}

fn ensure_success(status: reqwest::StatusCode, context: String) -> AppResult<()> {
    if status.is_success() {
        return Ok(());
    }

    Err(AppError::Unexpected(format!(
        "{context} Server returned {}.",
        status
    )))
}
