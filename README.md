# GLuaManager

Desktop addon manager for Garry's Mod.

Language: [🇺🇸 English](./README.md) | [🇷🇺 Русский](./README_RU.md)

GLuaManager helps keep a Garry's Mod addon folder tidy. It can scan what is already installed, show what is available, install new addons, and keep track of updates.

## 🔥 How it works

There are two ways to bring addons into GLuaManager: saved source indexes and a direct metadata link.

### 🔗 Data flow

The flow is simple:

- a **source** is a JSON list of addon metadata links
- each metadata file contains addon details and a direct archive link
- the archive is what gets downloaded and unpacked

If you already know one addon link, you can install it directly.

### 📦 Installed addons

- choose your Garry's Mod `addons` folder
- the app looks only at immediate subfolders that contain a `.addon` file
- each installed addon keeps its own local metadata
- the app stores the metadata link beside the addon
- update checks reload that link and compare versions
- if versions differ, the addon is marked as ready to update
- update downloads the archive from `url` and applies it to the local folder

### 🌐 Remote catalog

- add one or more source indexes in Settings
- each source points to addon metadata links
- the app loads those files and builds the addon catalog
- opening an addon shows its details, download link, dependencies, and local README if it exists
- install first shows the dependency tree and then downloads what you choose

> [!NOTE]
> GLuaManager is not a package manager. Garry's Mod addons share one filesystem namespace, so the app focuses on visibility, dependency warnings, and controlled installation.

## 🧩 Metadata format

`.addon` is a normal JSON file stored in the addon root.

Example:

```json
{
  "info": {
    "name": "Test addon",
    "description": "Example addon for documentation",
    "author": "darkfated"
  },
  "version": "1.0.0",
  "url": "https://example.com/test-addon.zip",
  "dependencies": [
    "https://example.com/library-one.json"
  ],
  "preserve": ["lua/autorun/myaddon_config.lua", "data", "materials/custom/**"]
}
```

Fields:

- `info.name` - addon name
- `info.description` - short description
- `info.author` - addon author
- `version` - installed and remote version number
- `url` - direct download URL for the addon archive
- `dependencies` - list of dependency metadata URLs
- `preserve` - files or paths that must not be overwritten during update

`preserve` supports:

- a specific file, for example `lua/autorun/myaddon_config.lua`
- a whole directory, for example `data`
- a glob pattern, for example `materials/custom/**`

All paths are relative to the addon folder.

## 🗂️ Sources and examples

Sources are stored as a list of source index URLs.

Example:

```json
[
  "https://example.com/default_source.json"
]
```

If you want a working example, look at the `exampleMeta/` folder. It contains sample addon metadata files for `mantle`, `newspawnmenu`, and `thirdperson`. The bundled `default_source.json` is an example source index that points to those files.

Source indexes and metadata files can live anywhere that serves valid JSON over `http` or `https`.

## 📸 Screenshots

### Home
<img width="1350" height="897" alt="Screenshot_2026-03-31_02-36-35" src="https://github.com/user-attachments/assets/8b42baf5-6c0d-42b8-91ed-fdbe137ba424" />

### Addon panel
<img width="1350" height="897" alt="Screenshot_2026-03-31_02-36-45" src="https://github.com/user-attachments/assets/3c105423-acfb-4357-8d21-908b098695d5" />

### Settings
<img width="1350" height="897" alt="Screenshot_2026-03-31_02-36-28" src="https://github.com/user-attachments/assets/7bf9ed10-b1f9-48a9-967b-2049c5a18e48" />

## ⚙️ Installation and updates

Everything starts with the addon metadata file.

Install an addon, and you get a clear dependency tree first. You choose what to keep, and only the selected parts are downloaded.
Update an addon, and it checks the current version before offering the newer one.

After that, the metadata link stays next to the addon, so future checks always point to the same place.

## 💻 CLI

GLuaManager can also be used from the terminal with the same `gluamanager` command.

- if you are already inside your `addons` folder, that folder becomes the working folder automatically
- `scan` shows the addons already sitting in that folder
- `available` shows addons from the sources you saved in the app, including a short `id`
- `install` accepts either a direct URL or an `addonId`
- `show` opens a clean addon card with the main details
- `update` checks the addon and offers the newer version if it exists
- `rollback` brings back the last saved version
- `remove` deletes an addon after confirmation
- Linux packages add a `gluamanager` command in `/usr/local/bin`
- Windows installers add the app folder to `PATH`

Typical commands:

```bash
gluamanager scan
gluamanager available
gluamanager install test-addon
gluamanager install https://example.com/test-addon.json
gluamanager show my-addon
gluamanager update my-addon
gluamanager rollback my-addon
gluamanager remove my-addon
gluamanager --json scan
```

Put `--json` before the command when you want machine-friendly output.

Use `gluamanager --help` or `gluamanager <command> --help` to see the full command list and the command-specific help.

## 🛠️ Build requirements

On Debian / Ubuntu Linux you need:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

You also need `tauri-cli`:

```bash
cargo install tauri-cli
```

Install frontend dependencies:

```bash
cd ui
npm install
```

## ▶️ Running

Development:

```bash
cd src-tauri
cargo tauri dev
```

Production build:

```bash
cd src-tauri
cargo tauri build
```
