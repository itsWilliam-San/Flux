use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

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
pub fn scan_directory_for_plugins(dir: &Path) -> Vec<PluginDescriptor> {
    let mut plugins = Vec::new();
    if !dir.is_dir() {
        return plugins;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Could not read plugin directory {:?}: {}", dir, err);
            return plugins;
        }
    };

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

    // 1. Workspace / current directory ./plugins (highest priority during dev)
    if let Ok(cwd) = std::env::current_dir() {
        let local_plugins_dir = if cfg!(debug_assertions) {
            cwd.join("../plugins")
        } else {
            cwd.join("plugins")
        };

        for plugin in scan_directory_for_plugins(&local_plugins_dir) {
            map.insert(plugin.manifest.id.clone(), plugin);
        }
    }

    // 2. User app data directory plugins
    if let Ok(user_dir) = get_user_plugins_dir(app) {
        for plugin in scan_directory_for_plugins(&user_dir) {
            map.entry(plugin.manifest.id.clone()).or_insert(plugin);
        }
    }

    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_deserialization() {
        let json_data = r#"{
            "name": "Excel Exporter",
            "description": "SANCAD to Excel. XLSV and CSV support",
            "version": "1.0.0",
            "author": "William",
            "repository": "",
            "download_url": "",
            "icon": "data:image/png;base64,abc",
            "id": "com.william.excel-exporter",
            "category": "exporter",
            "exports": [
                {
                    "id": "export_csv_detalhado",
                    "label": "Exportar Relatório (CSV)",
                    "description": "Gera um CSV completo com filtros avançados",
                    "entrypoint": "main.py:run_detailed_export",
                    "parameters": [
                        {
                            "name": "separator",
                            "param_type": "select",
                            "label": "Separador",
                            "options": [",", ";", "\t"],
                            "default": ";"
                        }
                    ]
                }
            ]
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json_data).unwrap();
        assert_eq!(manifest.id, "com.william.excel-exporter");
        assert_eq!(manifest.exports.len(), 1);
        assert_eq!(
            manifest.exports[0].entrypoint,
            "main.py:run_detailed_export"
        );
    }

    #[test]
    fn test_scan_directory_for_plugins() {
        let temp_dir =
            std::env::temp_dir().join(format!("sewerage_loader_test_{}", std::process::id()));
        let plugin_folder = temp_dir.join("test-plugin");
        let _ = fs::create_dir_all(&plugin_folder);

        let manifest_content = r#"{
            "name": "Test Plugin",
            "description": "Test plugin description",
            "version": "0.1.0",
            "author": "Tester",
            "id": "com.test.plugin",
            "category": "extraction",
            "exports": []
        }"#;

        fs::write(plugin_folder.join("manifest.json"), manifest_content).unwrap();

        let plugins = scan_directory_for_plugins(&temp_dir);
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].origin, PluginOrigin::Local);
        assert_eq!(plugins[0].manifest.id, "com.test.plugin");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_sample_excel_exporter_loaded() {
        let manifest_path = Path::new("../plugins/excel-exporter/manifest.json");
        if manifest_path.exists() {
            let content = fs::read_to_string(manifest_path).unwrap();
            let manifest: PluginManifest = serde_json::from_str(&content).unwrap();
            assert_eq!(manifest.id, "com.william.excel-exporter");
            assert_eq!(manifest.category, super::super::PluginCategory::Exporter);
            assert_eq!(manifest.exports.len(), 2);
        }
    }
}
