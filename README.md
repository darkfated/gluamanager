# GLuaManager

GLuaManager is a desktop addon manager for Garry's Mod.

Each addon has a `.addon` file in its root directory. The application detects these folders, shows installed addons, checks their versions on GitHub, updates them, and can install new addons from external sources.

## How It Works

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
