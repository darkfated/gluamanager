# GLuaManager

Desktop addon manager for Garry's Mod.

Language: [🇺🇸 English](./README.md) | [🇷🇺 Русский](./README_RU.md)

GLuaManager keeps a Garry's Mod workspace organized. It scans installed addons, checks remote metadata, shows updates, and installs addons from metadata URLs or source indexes.

## 🔥 How it works

There are two ways to add addons: source indexes in Settings and a direct metadata URL on the main tab.

### 🔗 Data flow

The flow is simple:

- a **source** is a JSON index in Settings
- each source lists addon metadata URLs
- each metadata file contains addon `info` and a direct archive `url`
- the archive is what gets downloaded and installed

If you already have one addon metadata URL, use **Add addon** on the main tab.

### 📦 Installed addons

- choose your Garry's Mod addons folder
- the app scans folders that contain a `.addon` manifest
- each installed addon keeps its local metadata
- the app stores the metadata URL beside the addon
- update checks reload that URL and compare versions
- if versions differ, the addon is marked as updateable
- update downloads the archive from `url` and applies it to the local folder

### 🌐 Remote catalog

- add one or more source index URLs in Settings
- each source index points to addon metadata URLs
- the app loads those metadata files and builds the remote catalog
- use **Add addon** on the main tab for a single metadata URL
- opening an addon shows its metadata, download URL, dependencies, and local README if it exists
- install first resolves dependencies and then downloads what is needed

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

## ⚙️ Installation and updates

Installation and updates use the same metadata file.

- install: resolve dependencies, show the plan, then download the archive from `url`
- update: reload metadata from the saved URL and compare `version`
- match: the addon is up to date
- mismatch: the addon can be updated

After installation, GLuaManager stores the metadata URL next to the addon so future checks stay tied to the same entry.

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
