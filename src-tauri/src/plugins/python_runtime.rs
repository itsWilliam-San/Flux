use log::warn;

use crate::error::PythonRuntimeError;
use crate::plugins::loader::find_plugins;
use crate::plugins::plugin::{Plugin, PluginDescription};
use crate::plugins::rpc::{RpcRequest, RpcResponse};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Mutex};

const PYTHON_RUNTIME_FOLDER: &str = "python-runtime";

/// Wrapping the HashMap in a std::sync::Mutex makes PythonRuntime Send + Sync,
/// which is required for Tauri's managed AppData state.
pub struct PythonRuntime {
    python_path: PathBuf,
    log_dir: PathBuf,
    pub plugin_folder_root: PathBuf,
    plugins: Mutex<HashMap<String, Plugin>>,
    tx: Sender<PythonRuntimeEvents>,
}

fn resolve_python_executable(resource_dir: PathBuf) -> Result<PathBuf, PythonRuntimeError> {
    if cfg!(debug_assertions) {
        let output = if cfg!(target_os = "windows") {
            std::process::Command::new("where.exe")
                .arg("python")
                .output()
        } else {
            std::process::Command::new("which").arg("python3").output()
        };

        if let Ok(res) = output {
            if res.status.success() {
                let stdout = String::from_utf8_lossy(&res.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let path = PathBuf::from(first_line.trim());
                    return Ok(path);
                }
            }
        }
    } else {
        let dir = resource_dir.join(PYTHON_RUNTIME_FOLDER);

        if cfg!(target_os = "windows") {
            let win_bin1 = dir.join("python.exe");
            let win_bin2 = dir.join("Scripts").join("python.exe");
            if win_bin1.is_file() {
                return Ok(win_bin1);
            }
            if win_bin2.is_file() {
                return Ok(win_bin2);
            }
        } else {
            let unix_bin1 = dir.join("bin").join("python3");
            let unix_bin2 = dir.join("bin").join("python");
            if unix_bin1.is_file() {
                return Ok(unix_bin1);
            }
            if unix_bin2.is_file() {
                return Ok(unix_bin2);
            }
        }
    }
    Err(PythonRuntimeError::PythonExecutableNotFound)
}

impl PythonRuntime {
    pub fn get_plugin(&self, plugin_id: &str) -> Option<PluginDescription> {
        let plugins = self.plugins.lock().unwrap_or_else(|e| e.into_inner());

        plugins
            .get(plugin_id)
            .map(|plugin| plugin.description.clone())
    }

    pub fn new(
        resource_dir: PathBuf,
        plugin_folder_root: PathBuf,
        log_dir: PathBuf,
    ) -> Result<(Self, Receiver<PythonRuntimeEvents>), PythonRuntimeError> {
        let python_path = resolve_python_executable(resource_dir)?;

        let (tx, rx) = mpsc::channel::<PythonRuntimeEvents>();

        Ok((
            Self {
                python_path,
                plugin_folder_root,
                log_dir,
                plugins: Mutex::new(HashMap::new()),
                tx,
            },
            rx,
        ))
    }

    /// Returns a snapshot of all loaded plugin descriptions for the frontend.
    pub fn get_plugins(&self) -> Vec<PluginDescription> {
        self.plugins
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .map(|plugin| plugin.description.clone())
            .collect()
    }

    pub fn update_plugins(&mut self) {
        match find_plugins(&self.plugin_folder_root) {
            Ok(found) => {
                if let Ok(mut guard) = self.plugins.lock() {
                    for plugin_desc in found {
                        let id = plugin_desc.manifest.id.clone();

                        guard
                            .entry(id)
                            .and_modify(|state| {
                                let plugin = &mut *state;
                                plugin.description = plugin_desc.clone();
                            })
                            .or_insert_with(|| Plugin::new(plugin_desc, self.tx.clone()));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to scan plugins directory: {e}");
            }
        }
    }

    /// Start all loaded plugins (spawn their processes).
    pub fn start_plugins(&mut self) {
        let python_path = self.python_path.clone();
        let mut guard = self.plugins.lock().unwrap_or_else(|e| e.into_inner());
        for (plugin_id, plugin) in guard.iter_mut() {
            let _ = plugin
                .start(&python_path, &self.log_dir)
                .inspect_err(|err| {
                    warn!(
                        "Failed to initialize plugin '{}'. Reason: {}",
                        plugin_id, err
                    );
                });
        }
    }

    /// Send an RPC request to a running plugin and wait for its response.
    pub fn execute_plugin_action(
        &self,
        plugin_id: &str,
        payload: RpcRequest,
    ) -> Result<RpcResponse, PythonRuntimeError> {
        let guard = self.plugins.lock().unwrap_or_else(|e| e.into_inner());
        let plugin = guard
            .get(plugin_id)
            .ok_or_else(|| PythonRuntimeError::PluginNotFound(plugin_id.to_owned()))?;

        plugin
            .send_request(payload)
            .map_err(PythonRuntimeError::Plugin)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginId(pub String);

#[derive(Debug)]
pub enum PythonRuntimeEvents {
    PluginStateChanged(PluginId, PluginState),
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum PluginState {
    Loaded,
    Unloaded,
}
