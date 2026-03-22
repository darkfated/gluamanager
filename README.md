# GLuaManager

Desktop addon manager for Garry's Mod.

Each addon has a `.addon` file in its root directory. The application detects these folders, shows installed addons, checks their versions on GitHub, updates them, and can install new addons from external sources.

## 🔥 How It Works

The application has two main workflows.

The first workflow is working with already installed addons:

- select an addons directory
- the application scans it for folders containing a `.addon` file
- for each addon, it downloads the remote `.addon` directly from the GitHub branch
- if the remote `version` differs from the local one, the addon is considered updatable
- when updating, it downloads the archive from the same branch and extracts it over the local folder

The second workflow is installing new addons:

- sources are added in the settings
- a source is a URL to a JSON file containing an array of `url + branch` objects
- the application reads these entries, downloads remote `.addon` files, and shows the available addons
- the selected addon is installed into the current addons directory

> [!WARNING]
> This is not a full-featured package manager like modern solutions (npm, pip, etc.). Due to Garry's Mod's architecture, addons cannot be isolated from each other, which limits dependency management. Additionally, there is no real version control - instead of git releases or commits, version designations are stored in the `.addon` metafile.

## 👀 Media

### Main Page

<img width="1353" height="897" alt="image" src="https://github.com/user-attachments/assets/e4c23927-b356-4d1f-8bfb-5d98e9945c9e" />

### Settings Page

<img width="1353" height="897" alt="image" src="https://github.com/user-attachments/assets/6944c179-ecad-46ac-9aa2-48e601d4b232" />

### Modal Menu

<img width="1353" height="897" alt="image" src="https://github.com/user-attachments/assets/bcb75ec8-6e76-4d27-bcf2-2e135e170c99" />

## `.addon` Format

`.addon` is a regular JSON file located in the root of the addon.

Example:

```json
{
  "name": "My Addon",
  "description": "Short description",
  "author": "username",
  "version": "1.2.0",
  "github": {
    "url": "username/repo",
    "branch": "master"
  },
  "dependencies": [
    {
      "url": "username/library-one",
      "branch": "master"
    }
  ],
  "preserve": ["lua/autorun/myaddon_config.lua", "data", "materials/custom/**"]
}
```

Fields:

- `name` - addon name
- `description` - short description
- `author` - author
- `version` - current addon version
- `github.url` - GitHub repository in `username/repo` format
- `github.branch` - branch used to fetch the `.addon` file and the archive for installation and updates
- `dependencies` - list of dependencies that will be installed together with the addon
- `preserve` - list of files and paths that must not be overwritten or deleted during update

`preserve` supports:

- a specific file, for example `lua/autorun/myaddon_config.lua`
- an entire directory, for example `data`
- a glob pattern, for example `materials/custom/**`

All paths are relative to the addon folder.

`dependencies` is defined as an array of objects in the same format as `github`:

```json
[
  {
    "url": "username/library-one",
    "branch": "main"
  },
  {
    "url": "username/library-two",
    "branch": "master"
  }
]
```

During installation, the application resolves dependencies recursively, builds the final list, and shows a confirmation dialog before downloading.

## Sources

A source is a URL to a JSON file containing an array of repositories with an explicitly specified branch.

Example:

```json
[
  {
    "url": "username/addon-one",
    "branch": "main"
  },
  {
    "url": "username/addon-two",
    "branch": "master"
  }
]
```

## Build

On Debian / Ubuntu Linux, the following system dependencies are required:

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

## Running

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
