# GLuaManager

Desktop addon manager for Garry's Mod.

Language: [🇺🇸 English](./README.md) | [🇷🇺 Русский](./README_RU.md)

GLuaManager helps you keep a Garry's Mod workspace organized: it scans installed addons, checks them against remote metadata, shows what can be updated, and installs new addons from metadata URLs.

## 🔥 How it works

There are two separate flows.

### 🔗 Data flow

The app follows a short chain:

- a **source** is a URL stored in Settings
- each source points to an addon **metadata JSON**
- the metadata contains the addon `info` and the direct `url` for the archive
- the archive is what gets downloaded and installed

That means a source can point to metadata on GitHub, on your own site, or anywhere else that serves a valid JSON file.

### 📦 Installed addons

- choose your Garry's Mod addons folder
- the app scans folders that contain a `.addon` manifest
- each installed addon keeps its own local metadata
- the app remembers where that addon came from through a small sidecar file
- when you check updates, it loads the saved metadata source again and compares versions
- if versions differ, the addon is marked as updateable
- when you update, the app downloads the archive from the manifest `url` and applies it over the local folder

### 🌐 Remote catalog

- add one or more source URLs in Settings
- each source points to an addon metadata JSON file
- the app loads those metadata files and builds the remote catalog
- opening an addon shows its metadata, download URL, dependencies, and local README if it exists
- installing an addon resolves dependency metadata first and shows a plan before anything is downloaded

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

Sources are stored as a list of metadata URLs.

Example:

```json
[
  "https://example.com/mantle.json",
  "https://example.com/newspawnmenu.json",
  "https://example.com/thirdperson.json"
]
```

If you want to see a working layout, look at the `exampleMeta/` folder in this repository. It contains sample metadata files for `mantle`, `newspawnmenu`, and `thirdperson`, and the default source list points to those examples.

They show the same format you can host anywhere that serves valid JSON.

## ⚙️ Installation and updates

Installation and updates follow the same rule: the app always uses the addon metadata source as the reference point.

- install: resolve dependencies, show the plan, then download the archive from `url`
- update: reload metadata from the saved source URL and compare `version`
- match: the addon is up to date
- mismatch: the addon can be updated

After installation, GLuaManager stores the source URL next to the addon so future checks stay tied to the same metadata source.

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
