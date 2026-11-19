use log::{error, warn};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

use crate::error::AppError;
use crate::plugins::plugin::{Plugin, PluginDescription};

use super::PluginManifest;

const MANIFEST_FILENAME: &str = "manifest.json";

///  Receives the plugin root folder (The folder that contains other plugins folder)
pub(super) fn scan_directory_for_plugins(
    plugin_folder_root: PathBuf,
) -> Result<Vec<PluginDescription>, AppError> {
    let entries = fs::read_dir(plugin_folder_root)?;

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
                        "Failed to read manfiest file: '{}'\nReason: {}",
                        manifest_path.display(),
                        e
                    );
                    return None;
                }
            };

            match serde_json::from_str::<PluginManifest>(&content) {
                Ok(mut manifest) => {
                    resolve_plugin_icon(&path, &mut manifest);

                    Some(PluginDescription { manifest, path })
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
/// Resolve the Plugin icon if it has one.
/// The expected behavior changes dependeing if it's a Local or Bridge Plugin
///
/// For Local plugin, the developer will probaly use a relative path
/// But Bridge plugin will probably use http/https since it may be running locally but through a TCP socket
/// or in a different computer.
fn resolve_plugin_icon(plugin_dir: &Path, manifest: &mut PluginManifest) {
    if let Some(ref icon_val) = manifest.icon {
        if icon_val.starts_with("http://") || icon_val.starts_with("https://") {
            return;
        }

        let icon_path = plugin_dir.join(icon_val);
        if icon_path.is_file() {
            let absolute_path_str = icon_path.to_string_lossy();
            manifest.icon = Some(format!("asset://{}", absolute_path_str));
        } else {
            warn!(
                "Plugin {} describes an icon, but it's neither a HTTP/HTTPS nor a valid file path.\nManifest.icon: {}",
                manifest.name,
                icon_val
            );

            // Remove it to prevent any trouble
            manifest.icon = None;
        }
    }
}

/// Returns the primary plugins directory inside app_data_dir, creating it if needed.
pub fn get_user_plugins_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let app_data = app.path().app_data_dir().inspect_err(|e| {
        error!(
            "Failed to get the XDG_STATE_HOME Path while searching for the Plugins folder. {}",
            e.to_string()
        )
    })?;

    let plugins_dir = app_data.join(PLUGIN_FOLDER);

    if !plugins_dir.exists() {
        let _ = fs::create_dir_all(&plugins_dir);
    }

    Ok(plugins_dir)
}

/// Discovers all local plugins across the user app_data plugins folder and workspace ./plugins.
pub fn discover_local_plugins(local_data_dir: &Path) -> Result<Vec<PluginDescription>, AppError> {
    let mut map: HashMap<String, Plugin> = HashMap::new();

    #[cfg(debug_assertions)]
    {
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(cwd_p) = cwd.parent() {
                let local_plugins_dir = cwd_p.join(PLUGIN_FOLDER);

                let plugins = scan_directory_for_plugins(&local_plugins_dir)?;

                for plugin in plugins {
                    map.insert(plugin.manifest.id.clone(), plugin);
                }
            }
        }
    }

    // 2. User app data directory plugins
    for plugin in scan_directory_for_plugins(&local_data_dir)? {
        map.entry(plugin.manifest.id.clone()).or_insert(plugin);
    }

    Ok(map.into_values().collect())
}
