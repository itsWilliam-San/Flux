use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

use crate::error::PluginError;

use super::{PluginDescriptor, PluginManifest, PluginOrigin};

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        result.push(TABLE[(b0 >> 2) as usize] as char);
        result.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            result.push(TABLE[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(TABLE[(b2 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Resolves the plugin icon into a Base64 data URI if it refers to a local file.
fn resolve_plugin_icon(plugin_dir: &Path, manifest: &mut PluginManifest) {
    if let Some(ref icon_val) = manifest.icon {
        if icon_val.starts_with("data:") {
            return;
        }

        let icon_path = plugin_dir.join(icon_val);
        if icon_path.is_file() {
            if let Ok(bytes) = fs::read(&icon_path) {
                let mime = if icon_val.ends_with(".svg") {
                    "image/svg+xml"
                } else if icon_val.ends_with(".jpg") || icon_val.ends_with(".jpeg") {
                    "image/jpeg"
                } else {
                    "image/png"
                };
                manifest.icon = Some(format!("data:{};base64,{}", mime, base64_encode(&bytes)));
                return;
            }
        }
    }

    // Default icon checks if icon is missing or placeholder
    for filename in &["icon.png", "icon.svg", "icon.jpg"] {
        let p = plugin_dir.join(filename);
        if p.is_file() {
            if let Ok(bytes) = fs::read(&p) {
                let mime = if filename.ends_with(".svg") {
                    "image/svg+xml"
                } else {
                    "image/png"
                };
                manifest.icon = Some(format!("data:{};base64,{}", mime, base64_encode(&bytes)));
                break;
            }
        }
    }
}

/// Scans a specific directory for subdirectories containing `manifest.json`.
pub fn scan_directory_for_plugins(dir: &Path) -> Result<Vec<PluginDescriptor>, PluginError> {
    let mut plugins = Vec::new();

    let entries = fs::read_dir(dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.is_file() {
                match fs::read_to_string(&manifest_path) {
                    Ok(content) => match serde_json::from_str::<PluginManifest>(&content) {
                        Ok(mut manifest) => {
                            resolve_plugin_icon(&path, &mut manifest);
                            plugins.push(PluginDescriptor {
                                origin: PluginOrigin::Local,
                                manifest,
                                path: Some(path.to_string_lossy().to_string()),
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to parse manifest at {:?}: {}", manifest_path, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read manifest file at {:?}: {}", manifest_path, e);
                    }
                }
            }
        }
    }

    plugins
}

/// Returns the primary plugins directory inside app_data_dir, creating it if needed.
pub fn get_user_plugins_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not get app data directory: {}", e))?;
    let plugins_dir = app_data.join("plugins");
    if !plugins_dir.exists() {
        let _ = fs::create_dir_all(&plugins_dir);
    }
    Ok(plugins_dir)
}

/// Discovers all local plugins across the user app_data plugins folder and workspace ./plugins.
pub fn discover_local_plugins(app: &tauri::AppHandle) -> Vec<PluginDescriptor> {
    let mut map: HashMap<String, PluginDescriptor> = HashMap::new();

    if cfg!(debug_assertions) {
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(cwd_p) = cwd.parent() {
                let local_plugins_dir = cwd_p.join("plugins");

                for plugin in scan_directory_for_plugins(&local_plugins_dir) {
                    map.insert(plugin.manifest.id.clone(), plugin);
                }
            }
        }
    } else {
        // 2. User app data directory plugins
        if let Ok(user_dir) = get_user_plugins_dir(app) {
            for plugin in scan_directory_for_plugins(&user_dir) {
                map.entry(plugin.manifest.id.clone()).or_insert(plugin);
            }
        }
    }

    map.into_values().collect()
}
