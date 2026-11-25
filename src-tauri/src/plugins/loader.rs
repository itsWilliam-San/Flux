use log::warn;
use std::fs;
use std::path::PathBuf;

use super::PluginManifest;
use crate::error::AppError;
use crate::plugins::plugin::PluginDescription;

const MANIFEST_FILENAME: &str = "manifest.json";

/// Receives the plugin root folder (the folder that contains other plugin subfolders)
/// and returns all valid plugin descriptions found inside.
pub(crate) fn find_plugins(
    plugin_folder_root: PathBuf,
) -> Result<Vec<PluginDescription>, AppError> {
    let entries = fs::read_dir(&plugin_folder_root)?;

    let plugins = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            let manifest_path = path.join(MANIFEST_FILENAME);

            let content = match fs::read_to_string(&manifest_path) {
                Ok(content) => content,
                Err(e) => {
                    warn!(
                        "Failed to read manifest file: '{}'\nReason: {}",
                        manifest_path.display(),
                        e
                    );
                    return None;
                }
            };

            match serde_json::from_str::<PluginManifest>(&content) {
                Ok(manifest) => {
                    let mut description = PluginDescription { manifest, path };
                    description.resolve_plugin_icon();
                    Some(description)
                }
                Err(e) => {
                    warn!(
                        "Failed to deserialize manifest file: '{}'\nReason: {}",
                        manifest_path.display(),
                        e
                    );
                    None
                }
            }
        })
        .collect();

    Ok(plugins)
}
