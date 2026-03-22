use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

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
}

#[derive(Debug, Clone)]
struct ResolvedInstallPlan {
    root_name: String,
    targets: Vec<github::RemoteManifest>,
}

const GITHUB_CONCURRENCY_LIMIT: usize = 6;
const BACKUP_ROOT_DIR: &str = ".gluamanager-backups";
const LAST_UPDATE_BACKUP_DIR: &str = "last-update";

pub async fn scan_root(root: &Path) -> AppResult<Vec<AddonView>> {
    let addons = discover_root(root)?;
    Ok(addons
        .into_iter()
        .map(|addon| addon_view(&addon, None, false, false, "Найден"))
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

    let manifest_path = addon_path.join(MANIFEST_NAME);
    let manifest = Manifest::load(&manifest_path)?;
    let (owner, repo) = crate::addon::parse_github_url(&manifest.github.url)?;
    let branch = manifest.github.branch.trim();
    let content = github::fetch_readme(&manifest).await?;

    Ok(content.map(|content| ReadmeView {
        content,
        base_url: Some(format!(
            "https://raw.githubusercontent.com/{owner}/{repo}/{branch}/"
        )),
        local_base_path: None,
    }))
}

pub async fn load_available_addon(
    root: &Path,
    repository_url: &str,
    branch: &str,
) -> AppResult<AvailableAddonView> {
    let remote = github::fetch_remote_manifest_from_repo_url(repository_url, branch).await?;
    let installed = discover_root(root)
        .unwrap_or_default()
        .into_iter()
        .any(|addon| {
            addon
                .manifest
                .github
                .url
                .trim()
                .eq_ignore_ascii_case(repository_url.trim())
        });

    Ok(AvailableAddonView::from_manifest(
        &remote.manifest,
        installed,
    ))
}

pub async fn load_available_addon_readme(
    repository_url: &str,
    branch: &str,
) -> AppResult<Option<ReadmeView>> {
    let (owner, repo) = crate::addon::parse_github_url(repository_url)?;
    let branch = branch.trim();
    let content = github::fetch_readme_from_repo_url(repository_url, branch).await?;

    Ok(content.map(|content| ReadmeView {
        content,
        base_url: Some(format!(
            "https://raw.githubusercontent.com/{owner}/{repo}/{branch}/"
        )),
        local_base_path: None,
    }))
}

pub async fn preview_install(
    root: &Path,
    repository_url: &str,
    branch: &str,
) -> AppResult<InstallPlanView> {
    let plan = resolve_install_plan(root, repository_url, branch).await?;
    Ok(InstallPlanView {
        root_name: plan.root_name,
        items: plan
            .targets
            .into_iter()
            .map(|remote| InstallPlanItem {
                name: install_name(&remote.manifest, &remote.repo.repo),
                repository_url: remote.manifest.github.url.clone(),
                branch: remote.manifest.github.branch.clone(),
            })
            .collect(),
    })
}

pub async fn update_addon(addon_path: &Path) -> AppResult<AddonView> {
    let manifest_path = addon_path.join(MANIFEST_NAME);
    let local = Manifest::load(&manifest_path)?;
    let remote = github::fetch_remote_manifest(&local).await?;

    if versions_match(&local.version, &remote.manifest.version) {
        return Ok(AddonView::from_manifest(
            &remote.manifest,
            addon_path.display().to_string(),
            Some(remote.manifest.version.clone()),
            false,
            false,
            "Actual",
        ));
    }

    let archive = github::download_repository_archive(&remote).await?;
    let preserve = merge_preserve(&local.preserve, &remote.manifest.preserve);
    let backup_path = create_update_backup(addon_path)?;
    let update_result = (|| -> AppResult<()> {
        let extracted = extract_archive(addon_path, &archive, &preserve)?;
        remove_stale_files(addon_path, &extracted, &preserve)?;
        fs::write(&manifest_path, &remote.raw)?;
        Ok(())
    })();

    if let Err(error) = update_result {
        restore_addon_from_backup(addon_path, &backup_path)?;
        return Err(AppError::Unexpected(format!(
            "Failed to update addon. Rolled back. Reason: {error}"
        )));
    }

    Ok(AddonView::from_manifest(
        &remote.manifest,
        addon_path.display().to_string(),
        Some(remote.manifest.version.clone()),
        false,
        false,
        "Actual",
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

pub async fn list_available_addons(
    root: &Path,
    source_urls: &[String],
) -> AppResult<Vec<AvailableAddonView>> {
    let installed_addons = discover_root(root).unwrap_or_default();
    let installed_by_repo: HashMap<String, String> = installed_addons
        .iter()
        .filter(|addon| !addon.manifest.github.url.trim().is_empty())
        .map(|addon| {
            (
                addon.manifest.github.url.trim().to_lowercase(),
                addon.addon_path.display().to_string(),
            )
        })
        .collect();

    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    for source_url in source_urls {
        let repositories = github::load_source_repositories(source_url).await?;
        for repository in repositories {
            let repository_url = format!("{}/{}", repository.owner, repository.repo);
            let normalized = repository_url.trim().to_lowercase();
            if normalized.is_empty() || !seen.insert(normalized.clone()) {
                continue;
            }

            let installed_path = installed_by_repo.get(&normalized).cloned();
            candidates.push((repository_url, repository.branch, installed_path));
        }
    }

    let mut available = run_limited(
        candidates,
        |(repository_url, branch, installed_path)| async move {
            match github::fetch_remote_manifest_from_repo_url(&repository_url, &branch).await {
                Ok(remote) => Some(AvailableAddonView::from_manifest(
                    &remote.manifest,
                    installed_path.is_some(),
                )),
                Err(_) => None,
            }
        },
    )
    .await?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    available.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(available)
}

pub async fn install_addon(
    root: &Path,
    repository_url: &str,
    branch: &str,
) -> AppResult<AddonView> {
    validate_root(root)?;
    let plan = resolve_install_plan(root, repository_url, branch).await?;
    let root_key = repository_key(repository_url);

    for remote in &plan.targets {
        install_remote(root, remote).await?;
    }

    if let Some(installed_root) = find_installed_addon_by_repo(root, &root_key)? {
        return check_addon(&installed_root.addon_path).await;
    }

    Err(AppError::Unexpected(format!(
        "Failed to find installed addon {} after installation.",
        repository_url
    )))
}

async fn check_discovered_addon(addon: DiscoveredAddon) -> AddonView {
    match github::fetch_remote_manifest(&addon.manifest).await {
        Ok(remote) => {
            let has_update = !versions_match(&addon.manifest.version, &remote.manifest.version);
            addon_view(
                &addon,
                Some(remote.manifest.version),
                has_update,
                false,
                if has_update {
                    "Update available"
                } else {
                    "Actual"
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
    let semaphore = Arc::new(Semaphore::new(GITHUB_CONCURRENCY_LIMIT));
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
                    "Background GitHub check failed: {error}"
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

async fn resolve_install_plan(
    root: &Path,
    repository_url: &str,
    branch: &str,
) -> AppResult<ResolvedInstallPlan> {
    validate_root(root)?;

    let installed_by_repo = installed_repo_map(root)?;
    let mut branch_by_repo = HashMap::<String, String>::new();
    let mut resolved = HashMap::<String, github::RemoteManifest>::new();
    let mut ordered_keys = Vec::<String>::new();
    let mut stack = vec![(
        repository_url.trim().to_string(),
        branch.trim().to_string(),
        false,
    )];
    let mut root_name = String::new();

    while let Some((url, current_branch, expanded)) = stack.pop() {
        let key = repository_key(&url);
        if ordered_keys.iter().any(|item| item == &key) {
            continue;
        }
        if !expanded && resolved.contains_key(&key) {
            continue;
        }

        if expanded {
            if resolved.contains_key(&key) && !installed_by_repo.contains_key(&key) {
                ordered_keys.push(key);
            }
            continue;
        }

        if let Some(existing_branch) = branch_by_repo.get(&key) {
            if existing_branch != &current_branch {
                return Err(AppError::Unexpected(format!(
                    "Repository {} has conflicting dependency branches: {} and {}.",
                    url, existing_branch, current_branch
                )));
            }
        } else {
            branch_by_repo.insert(key.clone(), current_branch.clone());
        }

        let remote = github::fetch_remote_manifest_from_repo_url(&url, &current_branch).await?;
        if root_name.is_empty() {
            root_name = install_name(&remote.manifest, &remote.repo.repo);
        }

        stack.push((url.clone(), current_branch.clone(), true));
        for dependency in remote.manifest.dependencies.iter().rev() {
            let dependency_url = dependency.url.trim();
            let dependency_branch = dependency.branch.trim();
            if dependency_url.is_empty() || dependency_branch.is_empty() {
                return Err(AppError::Unexpected(format!(
                    "Addon {} dependencies missing URL or branch.",
                    install_name(&remote.manifest, &remote.repo.repo)
                )));
            }

            let dependency_key = repository_key(dependency_url);
            if let Some(existing_branch) = branch_by_repo.get(&dependency_key) {
                if existing_branch != dependency_branch {
                    return Err(AppError::Unexpected(format!(
                        "Repository {} has conflicting dependency branches: {} and {}.",
                        dependency_url, existing_branch, dependency_branch
                    )));
                }
            } else {
                branch_by_repo.insert(dependency_key, dependency_branch.to_string());
            }

            if !installed_by_repo.contains_key(&repository_key(dependency_url)) {
                stack.push((
                    dependency_url.to_string(),
                    dependency_branch.to_string(),
                    false,
                ));
            }
        }
        resolved.insert(key, remote);
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
            .name
            .to_lowercase()
            .cmp(&right.manifest.name.to_lowercase())
    });

    Ok(found)
}

fn installed_repo_map(root: &Path) -> AppResult<HashMap<String, DiscoveredAddon>> {
    Ok(discover_root(root)?
        .into_iter()
        .filter(|addon| !addon.manifest.github.url.trim().is_empty())
        .map(|addon| (repository_key(&addon.manifest.github.url), addon))
        .collect())
}

fn find_installed_addon_by_repo(
    root: &Path,
    repository_key: &str,
) -> AppResult<Option<DiscoveredAddon>> {
    Ok(installed_repo_map(root)?.remove(repository_key))
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
    }))
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
        remote_version,
        has_update,
        has_error,
        status,
    )
}

async fn install_remote(root: &Path, remote: &github::RemoteManifest) -> AppResult<()> {
    let repository_key = repository_key(&remote.manifest.github.url);
    if find_installed_addon_by_repo(root, &repository_key)?.is_some() {
        return Ok(());
    }

    let target_path = unique_install_path(root, &remote.repo.repo);
    fs::create_dir_all(&target_path)?;

    let archive = github::download_repository_archive(remote).await?;
    let extracted = extract_archive(&target_path, &archive, &remote.manifest.preserve)?;
    remove_stale_files(&target_path, &extracted, &remote.manifest.preserve)?;
    fs::write(target_path.join(MANIFEST_NAME), &remote.raw)?;
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

fn repository_key(repository_url: &str) -> String {
    repository_url.trim().to_lowercase()
}

fn install_name(manifest: &Manifest, fallback: &str) -> String {
    let value = manifest.name.trim();
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
    fs::create_dir_all(&backup_path)?;
    copy_dir_contents(addon_path, &backup_path)?;
    Ok(backup_path)
}

fn restore_addon_from_backup(addon_path: &Path, backup_path: &Path) -> AppResult<()> {
    validate_root(addon_path)?;
    validate_root(backup_path)?;
    clear_directory(addon_path)?;
    copy_dir_contents(backup_path, addon_path)?;
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

fn copy_dir_contents(from: &Path, to: &Path) -> AppResult<()> {
    fs::create_dir_all(to)?;

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
            continue;
        }

        if file_type.is_file() {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::should_preserve;

    #[test]
    fn preserves_files_and_dirs() {
        assert!(should_preserve("data/config.json", &[String::from("data")]));
        assert!(should_preserve(
            "cfg/server.cfg",
            &[String::from("cfg/server.cfg")]
        ));
        assert!(should_preserve(
            "materials/custom/icon.png",
            &[String::from("materials/custom/**")]
        ));
        assert!(!should_preserve("lua/init.lua", &[String::from("data")]));
    }
}
