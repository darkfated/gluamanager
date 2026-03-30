use std::collections::{HashMap, HashSet};
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use serde::Serialize;
use url::Url;

use crate::addon::{AddonView, AvailableAddonView, Manifest};
use crate::error::{AppError, AppResult};
use crate::github;
use crate::settings;
use crate::update;

#[derive(Debug, Parser)]
#[command(
    name = "gluamanager",
    version,
    about = "A friendly terminal for Garry's Mod addons.",
    arg_required_else_help = true,
    subcommand_required = true,
    after_help = "Quick examples:\n  gluamanager scan\n  gluamanager available\n  gluamanager install test-addon\n  gluamanager install https://example.com/test-addon.json\n  gluamanager show my-addon\n  gluamanager update my-addon\n  gluamanager rollback my-addon\n  gluamanager remove my-addon\n  gluamanager --json scan\n\nTip: put --json before the command when you need machine-friendly output."
)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show the quick guide.
    Help,
    /// Show addons in the current folder.
    Scan,
    /// Show addons from saved sources.
    Available,
    /// Install an addon by URL or addonId.
    Install { source_url: String },
    /// Show a clean card for one addon.
    Show { addon_path: PathBuf },
    /// Check an addon and offer the newer version.
    Update { addon_path: PathBuf },
    /// Restore the last saved backup.
    Rollback { addon_path: PathBuf },
    /// Delete an addon folder with confirmation.
    Remove { addon_path: PathBuf },
}

#[derive(Debug, Clone)]
struct TreeNode {
    manifest: Manifest,
    source_url: String,
    installed: bool,
    children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
struct SourceRecord {
    manifest: Manifest,
    source_url: String,
    children: Vec<String>,
}

pub fn run() -> AppResult<()> {
    let cli = Cli::parse();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move { dispatch(cli).await })
}

async fn dispatch(cli: Cli) -> AppResult<()> {
    let root = std::env::current_dir()?;

    match cli.command {
        Command::Help => {
            print_general_help()?;
        }
        Command::Scan => {
            let addons = update::scan_root(&root).await?;
            if cli.json {
                print_json(&addons)?;
            } else {
                print_addon_list("Installed addons", &root, &addons);
            }
        }
        Command::Available => {
            let settings = settings::load_cli()?;
            let sources = if settings.sources.is_empty() {
                settings::default_sources()
            } else {
                settings.sources
            };
            let addons = update::list_available_addons(&root, &sources).await?;
            if cli.json {
                print_json(&addons)?;
            } else {
                print_available_addons(&root, &sources, &addons);
            }
        }
        Command::Install { source_url } => {
            install_flow(&root, &source_url, cli.json).await?;
        }
        Command::Show { addon_path } => {
            let addon = update::inspect_addon(&addon_path).await?;
            if cli.json {
                print_json(&addon)?;
            } else {
                print_addon_details("Addon details", &addon);
            }
        }
        Command::Update { addon_path } => {
            update_flow(&addon_path, cli.json).await?;
        }
        Command::Rollback { addon_path } => {
            let addon = update::rollback_addon(&addon_path).await?;
            if cli.json {
                print_json(&addon)?;
            } else {
                print_addon_details("Rolled back addon", &addon);
            }
        }
        Command::Remove { addon_path } => {
            remove_flow(&addon_path, cli.json).await?;
        }
    }

    Ok(())
}

async fn install_flow(root: &Path, source_url: &str, json: bool) -> AppResult<()> {
    let source_url = resolve_install_target(root, source_url).await?;
    let preview = update::load_available_addon(root, &source_url).await?;
    if !json {
        print_available_addon_card(&preview);
    }
    let installed = update::scan_root(root).await?;
    if let Some(existing) = find_installed_by_source(&installed, &source_url) {
        let addon_path = PathBuf::from(existing.addon_path.clone());
        let inspected = update::inspect_addon(&addon_path).await?;
        if json {
            print_json(&inspected)?;
        } else {
            print_addon_details("Installed addon", &inspected);
        }

        if inspected.has_update {
            let should_update = confirm(
                "A newer version is available. Update it now?",
                true,
                !json && io::stdin().is_terminal(),
            )?;
            if should_update {
                let updated = update::update_addon(&addon_path).await?;
                if json {
                    print_json(&updated)?;
                } else {
                    print_addon_details("Updated addon", &updated);
                }
            }
        } else if !json {
            println!("{}", "The addon is already up to date.".green().bold());
        }

        return Ok(());
    }

    let installed_keys = installed
        .iter()
        .filter_map(|addon| addon.source_url.as_ref().map(|value| source_key(value)))
        .collect::<HashSet<_>>();
    let tree = build_dependency_tree(&source_url, &installed_keys).await?;

    if !json {
        print_section("What will be installed");
        print_tree(&tree, 0, true, &HashSet::new());
    }

    let mut selected = HashSet::new();
    selected.insert(source_key(&tree.source_url));
    if !json {
        select_dependencies(&tree, &mut selected, !json && io::stdin().is_terminal())?;
        println!();
        print_section("Selected downloads");
        print_tree(&tree, 0, true, &selected);
    } else {
        collect_all_urls(&tree, &mut selected);
    }

    let should_install = confirm(
        "Continue with installation using the selected items?",
        true,
        !json && io::stdin().is_terminal(),
    )?;
    if !should_install {
        return Ok(());
    }

    let selected_urls = selected.into_iter().map(|key| key).collect::<Vec<_>>();
    let addon = update::install_addon_with_selection(root, &source_url, &selected_urls).await?;
    if json {
        print_json(&addon)?;
    } else {
        print_addon_details("Installed addon", &addon);
    }

    Ok(())
}

async fn resolve_install_target(root: &Path, value: &str) -> AppResult<String> {
    if Url::parse(value).is_ok() {
        return Ok(value.to_string());
    }

    let settings = settings::load_cli()?;
    let sources = if settings.sources.is_empty() {
        settings::default_sources()
    } else {
        settings.sources
    };
    let addons = update::list_available_addons(root, &sources).await?;
    let normalized = value.trim().to_lowercase();
    let mut matches = addons
        .into_iter()
        .filter(|addon| {
            addon.id.trim().eq_ignore_ascii_case(&normalized)
                || addon.name.trim().eq_ignore_ascii_case(&normalized)
        })
        .collect::<Vec<_>>();

    if matches.is_empty() {
        return Err(AppError::Unexpected(format!(
            "Could not find an addon with id '{value}' in your saved sources."
        )));
    }

    if matches.len() > 1 {
        return Err(AppError::Unexpected(format!(
            "More than one addon matches '{value}'. Please use the full source link or a more specific id."
        )));
    }

    Ok(matches
        .remove(0)
        .source_url
        .unwrap_or_else(|| value.to_string()))
}

async fn update_flow(addon_path: &Path, json: bool) -> AppResult<()> {
    let inspected = update::inspect_addon(addon_path).await?;
    if json {
        print_json(&inspected)?;
    } else {
        print_addon_details("Addon overview", &inspected);
    }

    if inspected.source_url.is_none() {
        if !json {
            println!("{}", "This addon has no saved source link.".yellow());
        }
        return Ok(());
    }

    if !inspected.has_update {
        if !json {
            println!("{}", "The addon is already up to date.".green().bold());
        }
        return Ok(());
    }

    let should_update = confirm(
        "Install the newer version now?",
        true,
        !json && io::stdin().is_terminal(),
    )?;
    if !should_update {
        return Ok(());
    }

    let updated = update::update_addon(addon_path).await?;
    if json {
        print_json(&updated)?;
    } else {
        print_addon_details("Updated addon", &updated);
    }

    Ok(())
}

async fn remove_flow(addon_path: &Path, json: bool) -> AppResult<()> {
    let addon = update::inspect_addon(addon_path).await?;
    if json {
        print_json(&addon)?;
    } else {
        print_addon_details("Delete addon", &addon);
    }

    let should_remove = confirm(
        &format!("Remove '{}' permanently?", addon.name),
        false,
        !json && io::stdin().is_terminal(),
    )?;
    if !should_remove {
        if !json {
            println!("{}", "Deletion cancelled.".dimmed());
        }
        return Ok(());
    }

    update::remove_addon(addon_path).await?;
    if json {
        print_json(&serde_json::json!({
            "removed": addon_path,
        }))?;
    } else {
        println!(
            "{} {}",
            "Deleted addon".green().bold(),
            addon_path.display()
        );
    }

    Ok(())
}

async fn build_dependency_tree(root_url: &str, installed: &HashSet<String>) -> AppResult<TreeNode> {
    let mut queue = vec![root_url.trim().to_string()];
    let mut seen = HashSet::new();
    let mut records = HashMap::<String, SourceRecord>::new();

    while let Some(source_url) = queue.pop() {
        let key = source_key(&source_url);
        if !seen.insert(key.clone()) {
            continue;
        }

        let manifest = github::fetch_manifest_from_url(&source_url).await?;
        let children = manifest
            .dependencies
            .iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>();

        for child in &children {
            queue.push(child.clone());
        }

        records.insert(
            key,
            SourceRecord {
                manifest,
                source_url,
                children,
            },
        );
    }

    let mut path = HashSet::new();
    materialize_tree(root_url, &records, installed, &mut path)
}

fn materialize_tree(
    source_url: &str,
    records: &HashMap<String, SourceRecord>,
    installed: &HashSet<String>,
    path: &mut HashSet<String>,
) -> AppResult<TreeNode> {
    let key = source_key(source_url);
    if !path.insert(key.clone()) {
        return Err(AppError::Unexpected(format!(
            "Dependency cycle detected while building the install tree: {}",
            source_url
        )));
    }

    let record = records.get(&key).ok_or_else(|| {
        AppError::Unexpected(format!(
            "Failed to build dependency tree for {}.",
            source_url
        ))
    })?;

    let mut children = Vec::new();
    for child_url in &record.children {
        let child_key = source_key(child_url);
        if records.contains_key(&child_key) {
            children.push(materialize_tree(child_url, records, installed, path)?);
        }
    }

    path.remove(&key);

    Ok(TreeNode {
        manifest: record.manifest.clone(),
        source_url: record.source_url.clone(),
        installed: installed.contains(&key),
        children,
    })
}

fn collect_all_urls(node: &TreeNode, selected: &mut HashSet<String>) {
    selected.insert(source_key(&node.source_url));
    for child in &node.children {
        collect_all_urls(child, selected);
    }
}

fn select_dependencies(
    node: &TreeNode,
    selected: &mut HashSet<String>,
    interactive: bool,
) -> AppResult<()> {
    for child in &node.children {
        let child_key = source_key(&child.source_url);
        if child.installed {
            selected.insert(child_key);
            continue;
        }

        let include = confirm(
            &format!("Include dependency '{}'?", child.manifest.info.name),
            false,
            interactive,
        )?;

        if include {
            selected.insert(child_key);
            select_dependencies(child, selected, interactive)?;
        }
    }

    Ok(())
}

fn find_installed_by_source(addons: &[AddonView], source_url: &str) -> Option<AddonView> {
    let key = source_key(source_url);
    addons
        .iter()
        .find(|addon| addon.source_url.as_deref().map(source_key).as_deref() == Some(key.as_str()))
        .cloned()
}

fn print_addon_list(title: &str, root: &Path, addons: &[AddonView]) {
    print_heading(title, Some(root));
    if addons.is_empty() {
        println!("{}", "No addons were found in this folder.".dimmed());
        return;
    }

    for addon in addons {
        print_line_card(
            &pretty_name(&addon.name),
            &[
                ("Version", addon.version.as_str()),
                ("State", &status_label(addon)),
                ("Folder", &pretty_path(&addon.addon_path)),
            ],
        );
    }
}

fn print_available_addons(root: &Path, sources: &[String], addons: &[AvailableAddonView]) {
    print_heading("Available addons", Some(root));
    println!(
        "{} {}",
        "Saved sources".cyan().bold(),
        format!("({})", sources.len()).dimmed()
    );
    if !sources.is_empty() {
        for source in sources {
            println!("  {} {}", "•".bright_cyan(), source);
        }
    }

    if addons.is_empty() {
        println!("{}", "No addons were found in your saved sources.".dimmed());
        return;
    }

    for addon in addons {
        let status = if addon.installed {
            "installed".green().bold().to_string()
        } else {
            "not installed".dimmed().to_string()
        };
        print_available_addon_card_with_status(addon, &status);
    }
}

fn print_addon_details(title: &str, addon: &AddonView) {
    print_heading(title, None);
    print_hero(addon);
    let update_state = if addon.has_update {
        "yes".yellow().bold().to_string()
    } else {
        "no".green().bold().to_string()
    };
    let attention_state = if addon.has_error {
        "needed".red().bold().to_string()
    } else {
        "clear".green().bold().to_string()
    };
    print_detail_line("Author", &pretty_name(&addon.author));
    print_detail_line("Version", &addon.version);
    print_detail_line("Status", &status_label(addon));
    print_detail_line("Folder", &pretty_path(&addon.addon_path));
    print_detail_line("Update available", &update_state);
    print_detail_line("Attention", &attention_state);
    print_detail_line(
        "Latest version",
        addon.remote_version.as_deref().unwrap_or("-"),
    );
    if let Some(source) = addon.source_url.as_deref() {
        print_detail_line("Source", source);
    }
    print_detail_line("Dependencies", &addon.dependencies.len().to_string());
    print_detail_line("Preserve rules", &addon.preserve.len().to_string());

    if !addon.description.trim().is_empty() {
        println!();
        print_section("Description");
        println!("{}", addon.description);
    }

    if !addon.dependencies.is_empty() {
        println!();
        print_section("Dependencies");
        for item in &addon.dependencies {
            println!("  - {item}");
        }
    }

    if !addon.preserve.is_empty() {
        println!();
        print_section("Preserve rules");
        for item in &addon.preserve {
            println!("  - {item}");
        }
    }
}

fn print_tree(node: &TreeNode, depth: usize, last: bool, selected: &HashSet<String>) {
    let mut prefix = String::new();
    for _ in 0..depth.saturating_sub(1) {
        prefix.push_str("│  ");
    }
    if depth > 0 {
        prefix.push_str(if last { "└─ " } else { "├─ " });
    }

    let key = source_key(&node.source_url);
    let marker = if node.installed {
        "[installed]"
    } else if selected.contains(&key) {
        "[x]"
    } else {
        "[ ]"
    };

    let label = format!(
        "{} v{} by {}",
        node.manifest.info.name,
        node.manifest.version,
        if node.manifest.info.author.trim().is_empty() {
            "unknown"
        } else {
            node.manifest.info.author.as_str()
        }
    );
    println!("{prefix}{marker} {label}");

    for (index, child) in node.children.iter().enumerate() {
        print_tree(child, depth + 1, index + 1 == node.children.len(), selected);
    }
}

fn print_heading(title: &str, root: Option<&Path>) {
    println!("{}", format!(" {} ", title).black().on_bright_cyan().bold());
    if let Some(root) = root {
        println!("{} {}", "Folder".bold().cyan(), root.display());
    }
}

fn print_section(title: &str) {
    println!("{}", title.cyan().bold());
}

fn print_hero(addon: &AddonView) {
    let name = if addon.name.trim().is_empty() {
        "Unnamed addon"
    } else {
        addon.name.as_str()
    };
    let author = if addon.author.trim().is_empty() {
        "unknown author"
    } else {
        addon.author.as_str()
    };
    println!(
        "{}",
        format!(" {name}  •  {author} ")
            .bold()
            .white()
            .bg_rgb::<32, 39, 52>()
    );
    println!(
        "{}",
        format!(
            "{}  •  {}",
            addon.version.as_str().green().bold(),
            status_label(addon)
        )
        .dimmed()
    );
}

fn print_detail_line(label: &str, value: &str) {
    println!("  {} {}", format!("{label}:").cyan().bold(), value);
}

fn print_line_card(title: &str, fields: &[(&str, &str)]) {
    println!("{}", format!(" {} ", title).black().on_bright_cyan().bold());
    for (label, value) in fields {
        print_detail_line(label, value);
    }
    println!("{}", "────────────────────────────────".dimmed());
}

fn print_available_addon_card(addon: &AvailableAddonView) {
    let status = if addon.installed {
        "installed".green().bold().to_string()
    } else {
        "not installed".dimmed().to_string()
    };
    print_available_addon_card_with_status(addon, &status);
}

fn print_available_addon_card_with_status(addon: &AvailableAddonView, status: &str) {
    println!(
        "{}",
        format!(" {} ", pretty_name(&addon.name))
            .black()
            .on_bright_cyan()
            .bold()
    );
    print_detail_line("ID", &addon.id);
    print_detail_line("Version", &addon.version);
    print_detail_line("Status", status);
    print_detail_line("Author", &pretty_name(&addon.author));
    if !addon.installed {
        print_detail_line("Install", &format!("gluamanager install {}", addon.id));
    }
    println!("{}", "────────────────────────────────".dimmed());
}

fn pretty_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "Unknown".into()
    } else {
        trimmed.into()
    }
}

fn pretty_path(value: &str) -> String {
    let path = Path::new(value);
    path.file_name()
        .and_then(|item| item.to_str())
        .map(|item| item.to_string())
        .unwrap_or_else(|| value.trim().to_string())
}

fn status_label(addon: &AddonView) -> String {
    if addon.has_error {
        "needs attention".red().bold().to_string()
    } else if addon.has_update {
        "update available".yellow().bold().to_string()
    } else {
        "up to date".green().bold().to_string()
    }
}

fn confirm(prompt: &str, default: bool, interactive: bool) -> AppResult<bool> {
    if !interactive {
        return Ok(default);
    }

    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    print!("{prompt} {suffix} ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let value = input.trim().to_lowercase();

    if value.is_empty() {
        return Ok(default);
    }

    Ok(matches!(value.as_str(), "y" | "yes"))
}

fn print_json<T: Serialize>(value: &T) -> AppResult<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn source_key(source_url: &str) -> String {
    source_url.trim().to_lowercase()
}

fn print_general_help() -> AppResult<()> {
    println!("{}", " GLuaManager ".black().on_bright_cyan().bold());
    println!("{}", "A simple terminal for Garry's Mod addons.".bold());
    println!(
        "{}",
        "Your current folder is treated as the addon root.".dimmed()
    );
    println!();
    println!("{}", "Commands".cyan().bold());
    println!(
        "  {:<12} {}",
        "scan".bold(),
        "Look through the current folder."
    );
    println!(
        "  {:<12} {}",
        "available".bold(),
        "Show what is available from your saved sources."
    );
    println!(
        "  {:<12} {}",
        "install".bold(),
        "Install by URL or addonId."
    );
    println!(
        "  {:<12} {}",
        "show".bold(),
        "Open a clean card for one addon."
    );
    println!(
        "  {:<12} {}",
        "update".bold(),
        "Check an addon and offer to update it."
    );
    println!(
        "  {:<12} {}",
        "rollback".bold(),
        "Restore the last saved backup."
    );
    println!(
        "  {:<12} {}",
        "remove".bold(),
        "Delete an addon folder after confirmation."
    );
    println!();
    println!("{}", "Options".cyan().bold());
    println!(
        "  {:<12} {}",
        "--json".bold(),
        "Show machine-friendly output."
    );
    println!(
        "  {:<12} {}",
        "--help".bold(),
        "Show the built-in help for any command."
    );
    println!();
    println!("{}", "Examples".cyan().bold());
    println!("  gluamanager scan");
    println!("  gluamanager available");
    println!("  gluamanager install test-addon");
    println!("  gluamanager install https://example.com/test-addon.json");
    println!("  gluamanager show my-addon");
    println!("  gluamanager update my-addon");
    println!("  gluamanager rollback my-addon");
    println!("  gluamanager remove my-addon");
    println!("  gluamanager --json scan");
    println!();
    Ok(())
}
