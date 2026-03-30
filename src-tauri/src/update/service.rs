use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::addon::{
    AddonView, AvailableAddonView, InstallPlanItem, InstallPlanView, Manifest, ReadmeView,
    MANIFEST_NAME,
};
use crate::error::{AppError, AppResult};
use crate::github;

#[derive(Debug, Clone)]
struct DiscoveredAddon {
    manifest: Manifest,
    addon_path: PathBuf,
    source_url: Option<String>,
}

#[derive(Debug, Clone)]
struct ResolvedInstallPlan {
    root_name: String,
    targets: Vec<ResolvedAddon>,
}

#[derive(Debug, Clone)]
struct ResolvedAddon {
    manifest: Manifest,
    source_url: String,
}

const FETCH_CONCURRENCY_LIMIT: usize = 6;
const BACKUP_ROOT_DIR: &str = ".gluamanager-backups";
const LAST_UPDATE_BACKUP_DIR: &str = "last-update";
const SOURCE_INFO_NAME: &str = ".gluamanager-source.json";
const STATUS_ACTUAL: &str = "Actual";
const STATUS_UPDATE_AVAILABLE: &str = "Update available";

pub async fn scan_root(root: &Path) -> AppResult<Vec<AddonView>> {
    let addons = discover_root(root)?;
    Ok(addons
        .into_iter()
        .map(|addon| addon_view(&addon, None, false, false, "Found"))
        .collect())
}

pub async fn check_updates(root: &Path) -> AppResult<Vec<AddonView>> {
    let addons = discover_root(root)?;
    let mut views = run_limited(addons, |addon| async move {
        check_discovered_addon(addon).await
    })
    .await?;

    views.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(views)
}

pub async fn check_addon(addon_path: &Path) -> AppResult<AddonView> {
    let manifest_path = addon_path.join(MANIFEST_NAME);
    let manifest = Manifest::load(&manifest_path)?;
    let addon = DiscoveredAddon {
        manifest,
        addon_path: addon_path.to_path_buf(),
        source_url: load_source_url(addon_path)?,
    };

    Ok(check_discovered_addon(addon).await)
}

pub async fn load_addon_readme(addon_path: &Path) -> AppResult<Option<ReadmeView>> {
    if let Some(local_readme) = load_local_readme(addon_path)? {
        return Ok(Some(ReadmeView {
            content: local_readme,
            base_url: None,
            local_base_path: Some(addon_path.display().to_string()),
        }));
    }

    Ok(None)
}

pub async fn load_available_addon(root: &Path, source_url: &str) -> AppResult<AvailableAddonView> {
    let remote = github::fetch_manifest_from_url(source_url).await?;
    let installed = installed_source_map(root)
        .unwrap_or_default()
        .contains_key(&source_key(source_url));

    Ok(AvailableAddonView::from_manifest(
        &remote,
        Some(source_url.to_string()),
        installed,
    ))
}

pub async fn load_available_addon_readme(_source_url: &str) -> AppResult<Option<ReadmeView>> {
    Ok(None)
}

pub async fn preview_install(root: &Path, source_url: &str) -> AppResult<InstallPlanView> {
    let plan = resolve_install_plan(root, source_url).await?;
    Ok(InstallPlanView {
        root_name: plan.root_name,
        items: plan
            .targets
            .into_iter()
            .map(|remote| InstallPlanItem {
                name: install_name(&remote.manifest, &remote.manifest.info.name),
            })
            .collect(),
    })
}

pub async fn update_addon(addon_path: &Path) -> AppResult<AddonView> {
    let manifest_path = addon_path.join(MANIFEST_NAME);
    let local = Manifest::load(&manifest_path)?;
    let source_url = load_source_url(addon_path)?;
    let Some(source_url) = source_url else {
        return Err(AppError::Unexpected(
            "Addon source metadata is missing.".into(),
        ));
    };
    let remote = github::fetch_manifest_from_url(&source_url).await?;

    if versions_match(&local.version, &remote.version) {
        return Ok(AddonView::from_manifest(
            &remote,
            addon_path.display().to_string(),
            Some(source_url),
            Some(remote.version.clone()),
            false,
            false,
            STATUS_ACTUAL,
        ));
    }

    let archive = github::download_archive_from_url(&remote.url).await?;
    let preserve = merge_preserve(&local.preserve, &remote.preserve);
    let backup_path = create_update_backup(addon_path)?;
    let update_result = (|| -> AppResult<()> {
        let extracted = extract_archive(addon_path, &archive, &preserve)?;
        remove_stale_files(addon_path, &extracted, &preserve)?;
        fs::write(&manifest_path, serde_json::to_vec_pretty(&remote)?)?;
        Ok(())
    })();

    if let Err(error) = update_result {
        restore_addon_from_backup(addon_path, &backup_path)?;
        return Err(AppError::Unexpected(format!(
            "Failed to update addon. Rolled back. Reason: {error}"
        )));
    }

    Ok(AddonView::from_manifest(
        &remote,
        addon_path.display().to_string(),
        Some(source_url),
        Some(remote.version.clone()),
        false,
        false,
        STATUS_ACTUAL,
    ))
}

pub async fn rollback_addon(addon_path: &Path) -> AppResult<AddonView> {
    let backup_path = last_update_backup_path(addon_path)?;
    if !backup_path.exists() {
        return Err(AppError::Unexpected(
            "No saved rollback available for this addon after update.".into(),
        ));
    }

    restore_addon_from_backup(addon_path, &backup_path)?;
    check_addon(addon_path).await
}

pub async fn remove_addon(addon_path: &Path) -> AppResult<()> {
    let manifest_path = addon_path.join(MANIFEST_NAME);
    if !manifest_path.exists() {
        return Err(AppError::Unexpected(format!(
            "Addon manifest was not found: {}",
            manifest_path.display()
        )));
    }
    if !addon_path.exists() || !addon_path.is_dir() {
        return Err(AppError::Unexpected(format!(
            "Addon folder does not exist: {}",
            addon_path.display()
        )));
    }

    if let Ok(backup_path) = last_update_backup_path(addon_path) {
        if let Some(backup_root) = backup_path.parent() {
            if backup_root.exists() {
                let _ = fs::remove_dir_all(backup_root);
            }
        }
    }

    fs::remove_dir_all(addon_path)?;
    Ok(())
}

pub async fn list_available_addons(
    root: &Path,
    source_urls: &[String],
) -> AppResult<Vec<AvailableAddonView>> {
    let installed_sources = installed_source_map(root)?;
    let installed_keys = Arc::new(
        installed_sources
            .keys()
            .cloned()
            .collect::<HashSet<String>>(),
    );
    let mut addon_urls = Vec::new();
    let mut seen_addons = HashSet::<String>::new();

    for source_url in source_urls {
        let source_index = match github::fetch_source_index_from_url(source_url).await {
            Ok(items) => items,
            Err(_) => continue,
        };

        for addon_url in source_index {
            let normalized_key = source_key(&addon_url);
            if !seen_addons.insert(normalized_key) {
                continue;
            }
            addon_urls.push(addon_url);
        }
    }

    let mut available = run_limited(addon_urls, move |addon_url| {
        let installed_keys = Arc::clone(&installed_keys);
        async move {
            let remote = match github::fetch_manifest_from_url(&addon_url).await {
                Ok(remote) => remote,
                Err(_) => return None,
            };
            let installed = installed_keys.contains(&source_key(&addon_url));
            Some(AvailableAddonView::from_manifest(
                &remote,
                Some(addon_url),
                installed,
            ))
        }
    })
    .await?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    available.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(available)
}

pub async fn install_addon(root: &Path, source_url: &str) -> AppResult<AddonView> {
    validate_root(root)?;
    let plan = resolve_install_plan(root, source_url).await?;
    let root_key = source_key(source_url);

    for remote in &plan.targets {
        install_remote(root, remote).await?;
    }

    if let Some(installed_root) = find_installed_addon_by_source(root, &root_key)? {
        return check_addon(&installed_root.addon_path).await;
    }

    Err(AppError::Unexpected(format!(
        "Failed to find installed addon {} after installation.",
        source_url
    )))
}

async fn check_discovered_addon(addon: DiscoveredAddon) -> AddonView {
    let Some(source_url) = addon.source_url.clone() else {
        return addon_view(
            &addon,
            None,
            false,
            true,
            "Verification failed: addon source metadata is missing.",
        );
    };

    match github::fetch_manifest_from_url(&source_url).await {
        Ok(remote) => {
            let has_update = !versions_match(&addon.manifest.version, &remote.version);
            addon_view(
                &addon,
                Some(remote.version),
                has_update,
                false,
                if has_update {
                    STATUS_UPDATE_AVAILABLE
                } else {
                    STATUS_ACTUAL
                },
            )
        }
        Err(error) => addon_view(
            &addon,
            None,
            false,
            true,
            format!("Verification failed: {error}"),
        ),
    }
}

async fn run_limited<T, F, Fut, R>(items: Vec<T>, task_fn: F) -> AppResult<Vec<R>>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = R> + Send + 'static,
{
    let semaphore = Arc::new(Semaphore::new(FETCH_CONCURRENCY_LIMIT));
    let task_fn = Arc::new(task_fn);
    let mut join_set = JoinSet::new();

    for item in items {
        let semaphore = Arc::clone(&semaphore);
        let task_fn = Arc::clone(&task_fn);
        join_set.spawn(async move {
            let _permit = semaphore
                .acquire_owned()
                .await
                .expect("semaphore should stay available");
            task_fn(item).await
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(value) => results.push(value),
            Err(error) => {
                return Err(AppError::Unexpected(format!(
                    "Background metadata check failed: {error}"
                )));
            }
        }
    }

    Ok(results)
}

fn validate_root(root: &Path) -> AppResult<()> {
    if !root.exists() {
        return Err(AppError::Unexpected(format!(
            "Folder does not exist: {}",
            root.display()
        )));
    }
    if !root.is_dir() {
        return Err(AppError::Unexpected(format!(
            "Specified path is not a directory: {}",
            root.display()
        )));
    }
    Ok(())
}

fn versions_match(local: &str, remote: &str) -> bool {
    local.trim() == remote.trim()
}

async fn resolve_install_plan(root: &Path, source_url: &str) -> AppResult<ResolvedInstallPlan> {
    validate_root(root)?;

    let installed_by_source = installed_source_map(root)?;
    let mut resolved = HashMap::<String, ResolvedAddon>::new();
    let mut ordered_keys = Vec::<String>::new();
    let mut stack = vec![(source_url.trim().to_string(), false)];
    let mut root_name = String::new();

    while let Some((url, expanded)) = stack.pop() {
        let key = source_key(&url);
        if ordered_keys.iter().any(|item| item == &key) {
            continue;
        }
        if !expanded && resolved.contains_key(&key) {
            continue;
        }

        if expanded {
            if resolved.contains_key(&key) && !installed_by_source.contains_key(&key) {
                ordered_keys.push(key);
            }
            continue;
        }

        let remote = github::fetch_manifest_from_url(&url).await?;
        if root_name.is_empty() {
            root_name = install_name(&remote, &remote.info.name);
        }

        stack.push((url.clone(), true));
        for dependency_url in remote.dependencies.iter().rev() {
            let dependency_url = dependency_url.trim();
            if dependency_url.is_empty() {
                return Err(AppError::Unexpected(format!(
                    "Addon {} dependencies contain an empty URL.",
                    install_name(&remote, &remote.info.name)
                )));
            }

            let dependency_key = source_key(dependency_url);
            if !installed_by_source.contains_key(&dependency_key) {
                stack.push((dependency_url.to_string(), false));
            }
        }
        resolved.insert(
            key,
            ResolvedAddon {
                manifest: remote,
                source_url: url,
            },
        );
    }

    let targets = ordered_keys
        .into_iter()
        .filter_map(|key| resolved.remove(&key))
        .collect::<Vec<_>>();

    Ok(ResolvedInstallPlan { root_name, targets })
}

fn discover_root(root: &Path) -> AppResult<Vec<DiscoveredAddon>> {
    validate_root(root)?;

    let mut found = Vec::new();
    if let Some(addon) = try_manifest(root)? {
        found.push(addon);
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(addon) = try_manifest(&path)? {
                found.push(addon);
            }
        }
    }

    found.sort_by(|left, right| {
        left.manifest
            .info
            .name
            .to_lowercase()
            .cmp(&right.manifest.info.name.to_lowercase())
    });

    Ok(found)
}

fn installed_source_map(root: &Path) -> AppResult<HashMap<String, DiscoveredAddon>> {
    Ok(discover_root(root)?
        .into_iter()
        .filter_map(|addon| {
            let source_url = addon.source_url.clone()?;
            Some((source_key(&source_url), addon))
        })
        .collect())
}

fn find_installed_addon_by_source(
    root: &Path,
    source_key: &str,
) -> AppResult<Option<DiscoveredAddon>> {
    Ok(installed_source_map(root)?.remove(source_key))
}

fn try_manifest(dir: &Path) -> AppResult<Option<DiscoveredAddon>> {
    let manifest_path = dir.join(MANIFEST_NAME);
    if !manifest_path.exists() {
        return Ok(None);
    }

    let manifest = Manifest::load(&manifest_path)?;
    Ok(Some(DiscoveredAddon {
        manifest,
        addon_path: dir.to_path_buf(),
        source_url: load_source_url(dir)?,
    }))
}

fn source_info_path(addon_path: &Path) -> PathBuf {
    addon_path.join(SOURCE_INFO_NAME)
}

fn load_source_url(addon_path: &Path) -> AppResult<Option<String>> {
    let path = source_info_path(addon_path);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let record: SourceRecord = serde_json::from_str(&content)?;
    let value = record.source_url.trim().to_string();
    if value.is_empty() {
        return Ok(None);
    }

    Ok(Some(value))
}

fn save_source_url(addon_path: &Path, source_url: &str) -> AppResult<()> {
    let path = source_info_path(addon_path);
    let record = SourceRecord {
        source_url: source_url.trim().to_string(),
    };
    fs::write(path, serde_json::to_vec_pretty(&record)?)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct SourceRecord {
    source_url: String,
}

fn load_local_readme(addon_path: &Path) -> AppResult<Option<String>> {
    let readme_path = addon_path.join("README.md");
    if readme_path.exists() && readme_path.is_file() {
        return Ok(Some(fs::read_to_string(readme_path)?));
    }

    Ok(None)
}

fn addon_view(
    addon: &DiscoveredAddon,
    remote_version: Option<String>,
    has_update: bool,
    has_error: bool,
    status: impl Into<String>,
) -> AddonView {
    AddonView::from_manifest(
        &addon.manifest,
        addon.addon_path.display().to_string(),
        addon.source_url.clone(),
        remote_version,
        has_update,
        has_error,
        status,
    )
}

async fn install_remote(root: &Path, remote: &ResolvedAddon) -> AppResult<()> {
    let source_key = source_key(&remote.source_url);
    if find_installed_addon_by_source(root, &source_key)?.is_some() {
        return Ok(());
    }

    let target_path = unique_install_path(root, &remote.manifest.info.name);
    fs::create_dir_all(&target_path)?;

    let archive = github::download_archive_from_url(&remote.manifest.url).await?;
    let extracted = extract_archive(&target_path, &archive, &remote.manifest.preserve)?;
    remove_stale_files(&target_path, &extracted, &remote.manifest.preserve)?;
    fs::write(
        target_path.join(MANIFEST_NAME),
        serde_json::to_vec_pretty(&remote.manifest)?,
    )?;
    save_source_url(&target_path, &remote.source_url)?;
    Ok(())
}

fn merge_preserve(local: &[String], remote: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for item in local.iter().chain(remote.iter()) {
        if seen.insert(item.clone()) {
            merged.push(item.clone());
        }
    }

    merged
}

fn source_key(source_url: &str) -> String {
    source_url.trim().to_lowercase()
}

fn install_name(manifest: &Manifest, fallback: &str) -> String {
    let value = manifest.info.name.trim();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn unique_install_path(root: &Path, preferred_name: &str) -> PathBuf {
    let base_name = sanitize_dir_name(preferred_name);
    let mut path = root.join(&base_name);
    let mut index = 2usize;

    while path.exists() {
        path = root.join(format!("{base_name}-{index}"));
        index += 1;
    }

    path
}

fn create_update_backup(addon_path: &Path) -> AppResult<PathBuf> {
    validate_root(addon_path)?;
    let backup_path = last_update_backup_path(addon_path)?;
    if backup_path.exists() {
        fs::remove_dir_all(&backup_path)?;
    }
    copy_dir_recursive(addon_path, &backup_path)?;
    Ok(backup_path)
}

fn restore_addon_from_backup(addon_path: &Path, backup_path: &Path) -> AppResult<()> {
    validate_root(addon_path)?;
    validate_root(backup_path)?;
    clear_directory(addon_path)?;
    copy_dir_recursive(backup_path, addon_path)?;
    Ok(())
}

fn last_update_backup_path(addon_path: &Path) -> AppResult<PathBuf> {
    let parent = addon_path.parent().ok_or_else(|| {
        AppError::Unexpected(format!(
            "Failed to determine parent directory for addon: {}",
            addon_path.display()
        ))
    })?;
    let addon_name = addon_path.file_name().ok_or_else(|| {
        AppError::Unexpected(format!(
            "Failed to determine addon directory name: {}",
            addon_path.display()
        ))
    })?;

    Ok(parent
        .join(BACKUP_ROOT_DIR)
        .join(addon_name)
        .join(LAST_UPDATE_BACKUP_DIR))
}

fn copy_dir_recursive(from: &Path, to: &Path) -> AppResult<()> {
    fs::create_dir_all(to)?;

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
}

fn clear_directory(path: &Path) -> AppResult<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(entry_path)?;
        } else {
            fs::remove_file(entry_path)?;
        }
    }

    Ok(())
}

fn sanitize_dir_name(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();

    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        "addon".to_string()
    } else {
        trimmed.to_string()
    }
}

fn extract_archive(
    addon_path: &Path,
    archive_bytes: &[u8],
    preserve: &[String],
) -> AppResult<HashSet<String>> {
    let cursor = Cursor::new(archive_bytes);
    let mut archive = ZipArchive::new(cursor)?;
    let mut extracted = HashSet::new();

    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        let Some(relative) = sanitize_archive_path(file.name())? else {
            continue;
        };

        let relative_string = to_slash_path(&relative);
        extracted.insert(relative_string.clone());

        if should_preserve(&relative_string, preserve) {
            continue;
        }

        let target_path = addon_path.join(&relative);
        if file.is_dir() {
            fs::create_dir_all(&target_path)?;
            continue;
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let mut output = fs::File::create(&target_path)?;
        output.write_all(&buffer)?;
    }

    Ok(extracted)
}

fn sanitize_archive_path(name: &str) -> AppResult<Option<PathBuf>> {
    let path = Path::new(name);
    let mut components = path.components();

    if components.next().is_none() {
        return Ok(None);
    }

    let mut clean = PathBuf::new();
    for component in components {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::Unexpected(format!(
                    "Archive contains an unsafe path: {name}"
                )))
            }
        }
    }

    if clean.as_os_str().is_empty() {
        return Ok(None);
    }

    Ok(Some(clean))
}

fn remove_stale_files(
    addon_path: &Path,
    extracted: &HashSet<String>,
    preserve: &[String],
) -> AppResult<()> {
    let mut directories = Vec::new();

    for entry in WalkDir::new(addon_path).min_depth(1) {
        let entry = entry.map_err(|error| AppError::Unexpected(error.to_string()))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(addon_path)
            .map_err(|error| AppError::Unexpected(error.to_string()))?;
        let relative_string = to_slash_path(relative);

        if entry.file_type().is_dir() {
            directories.push(path.to_path_buf());
            continue;
        }

        if should_preserve(&relative_string, preserve) || extracted.contains(&relative_string) {
            continue;
        }

        fs::remove_file(path)?;
    }

    directories.sort_by(|left, right| right.components().count().cmp(&left.components().count()));
    for directory in directories {
        if directory == addon_path {
            continue;
        }
        if fs::read_dir(&directory)?.next().is_none() {
            let _ = fs::remove_dir(&directory);
        }
    }

    Ok(())
}

fn should_preserve(path: &str, preserve: &[String]) -> bool {
    let normalized = path.trim_matches('/');

    preserve.iter().any(|rule| {
        let rule = rule.replace('\\', "/").trim().trim_matches('/').to_string();
        if rule.is_empty() {
            return false;
        }

        if normalized == rule || normalized.starts_with(&(rule.clone() + "/")) {
            return true;
        }

        if let Some(prefix) = rule.strip_suffix("/**") {
            return normalized == prefix || normalized.starts_with(&(prefix.to_string() + "/"));
        }

        glob_matches(normalized, &rule)
    })
}

fn glob_matches(path: &str, pattern: &str) -> bool {
    glob_match::glob_match(pattern.as_bytes(), path.as_bytes())
}

fn to_slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

mod glob_match {
    pub fn glob_match(pattern: &[u8], value: &[u8]) -> bool {
        let (mut p, mut v) = (0usize, 0usize);
        let (mut star, mut star_match) = (None, 0usize);

        while v < value.len() {
            if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == value[v]) {
                p += 1;
                v += 1;
            } else if p < pattern.len() && pattern[p] == b'*' {
                star = Some(p);
                star_match = v;
                p += 1;
            } else if let Some(star_index) = star {
                p = star_index + 1;
                star_match += 1;
                v = star_match;
            } else {
                return false;
            }
        }

        while p < pattern.len() && pattern[p] == b'*' {
            p += 1;
        }

        p == pattern.len()
    }
}
