pub mod manifest;
pub mod models;

pub use manifest::{parse_github_url, GithubSource, Manifest, MANIFEST_NAME};
pub use models::{AddonView, AvailableAddonView, InstallPlanItem, InstallPlanView, ReadmeView};
