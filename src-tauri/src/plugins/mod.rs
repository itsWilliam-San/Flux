pub mod loader;
pub mod python_runtime;
pub mod rpc;

use std::{path::Path, process::Command};

use serde::{Deserialize, Serialize};

use crate::plugins::rpc::{RpcRequest, RpcResponse};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Vec<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(alias = "downloadUrl", skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub id: String,
    pub exports: Vec<PluginExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginExport {
    pub id: String,
    pub label: String,
    pub description: String,
    pub entrypoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<PluginParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_network_data: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_network_data: Option<bool>,
}

impl PluginManifest {
    pub fn get_export(&self, export_id: &str) -> Option<&PluginExport> {
        self.exports.iter().find(|export| export.id == export_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginParameter {
    pub name: String,
    #[serde(alias = "paramType", alias = "type", rename = "param_type")]
    pub param_type: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

pub mod plugin {
    use std::{
        io::{BufReader, Write},
        path::Path,
        sync::{
            mpsc::{self, Receiver, Sender},
            Mutex,
        },
        thread,
        time::Duration,
    };

    use log::{info, warn};

    use crate::error::{AppError, PluginError};

    use super::*;

    // Wrapping PythonRuntime's HashMap in a std::sync::Mutex makes AppData Send + Sync.
    // Plugin itself only needs to be Send (threads move it). The Mutex<Receiver> on
    // PluginHandle already satisfies the shared-access requirement.

    pub struct Plugin {
        pub description: PluginDescription,
        handle: Option<PluginHandle>,
    }

    struct PluginHandle {
        stdin_sender: Sender<RpcRequest>,
        stdout_receiver: Mutex<Receiver<RpcResponse>>,
        // Keep the join handle so the thread isn't detached
        _thread_handle: thread::JoinHandle<()>,
    }

    impl PluginHandle {
        /// Send an RPC request to the plugin's stdin.
        pub fn send_rpc(&self, payload: RpcRequest) -> Result<(), PluginError> {
            self.stdin_sender
                .send(payload)
                .map_err(|_| PluginError::PluginDisconnected("send channel closed".into()))
        }

        /// Block until a response arrives or the timeout expires.
        pub fn recv_response(&self, timeout: Duration) -> Result<RpcResponse, PluginError> {
            let guard = self
                .stdout_receiver
                .lock()
                .map_err(|_| PluginError::InternalError("receiver mutex poisoned".into()))?;
            guard
                .recv_timeout(timeout)
                .map_err(|_| PluginError::InternalError("plugin response timed out".into()))
        }
    }

    // Safety: Mutex<Receiver<_>> is Send but not Sync. We only expose it through
    // &self methods that lock the mutex, so sharing PluginHandle across threads
    // via &AppData is safe.
    unsafe impl Sync for PluginHandle {}

    #[derive(Serialize, Deserialize, Clone)]
    pub struct PluginDescription {
        pub manifest: PluginManifest,
        pub path: std::path::PathBuf,
    }

    impl PluginDescription {
        pub fn resolve_plugin_icon(&mut self) {
            if let Some(ref icon_val) = self.manifest.icon {
                if icon_val.starts_with("http://") || icon_val.starts_with("https://") {
                    return;
                }

                let icon_path = self.path.join(icon_val);
                if icon_path.is_file() {
                    let absolute_path_str = icon_path.to_string_lossy();
                    self.manifest.icon = Some(format!("asset://{}", absolute_path_str));
                } else {
                    warn!(
                        "Plugin '{}' has an icon path that is not a valid file: {}",
                        self.manifest.name, icon_val
                    );
                    self.manifest.icon = None;
                }
            }
        }
    }

    impl Plugin {
        pub fn new(pd: PluginDescription) -> Self {
            Self {
                description: pd,
                handle: None,
            }
        }

        /// TODO Maybe some code here shouldn't belong here but in the PythonRuntime
        /// while some others should belong here, or maybe everythin here should belong to PythonRuntime
        pub fn start(&mut self, python_path: &Path, log_dir: &Path) -> Result<(), AppError> {
            let (tx_req, rx_req) = mpsc::channel::<RpcRequest>();
            let (tx_res, rx_res) = mpsc::channel::<RpcResponse>();

            let plugin_main = self.description.path.join("main.py");

            let mut command = base_command(python_path);
            command.arg(&plugin_main).arg(log_dir).current_dir(&self.description.path);

            let plugin_id = self.description.manifest.id.clone();

            let thread_handle = thread::spawn(move || {
                let mut child = match command.spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to spawn plugin process: {e}");
                        return;
                    }
                };

                let mut stdin = match child.stdin.take() {
                    Some(s) => s,
                    None => {
                        warn!("Plugin process has no stdin pipe");
                        return;
                    }
                };
                let stdout = match child.stdout.take() {
                    Some(s) => s,
                    None => {
                        warn!("Plugin process has no stdout pipe");
                        return;
                    }
                };

                // Reader thread: deserialise newline-delimited JSON responses from stdout
                let _stdout_thread = thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    let stream =
                        serde_json::Deserializer::from_reader(reader).into_iter::<RpcResponse>();
                    for item in stream {
                        match item {
                            Ok(response) => {
                                if tx_res.send(response).is_err() {
                                    break;
                                }
                            }
                            Err(err) => {
                                warn!("Plugin sent an invalid JSON payload: {err}");
                            }
                        }
                    }
                });

                // Writer loop: forward queued RPC requests to the plugin's stdin
                while let Ok(request) = rx_req.recv() {
                    match serde_json::to_string(&request) {
                        Ok(json_string) => {
                            if let Err(e) = writeln!(stdin, "{}", json_string) {
                                warn!("Failed to write to plugin stdin: {e}");
                                break;
                            }
                            if let Err(e) = stdin.flush() {
                                warn!("Failed to flush plugin stdin: {e}");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to serialise RPC request: {e}");
                        }
                    }
                }

                drop(stdin);

                match child.wait() {
                    Ok(status) => {
                        info!("Plugin '{plugin_id}' process exited with status: {status}")
                    }
                    Err(e) => warn!(
                        "Plugin '{plugin_id}' Failed while waiting for plugin process to exit: {e}"
                    ),
                }

                let _ = _stdout_thread.join();
                info!("Plugin '{plugin_id}' stopped and I/O threads finished");
            });

            self.handle = Some(PluginHandle {
                stdin_sender: tx_req,
                stdout_receiver: Mutex::new(rx_res),
                _thread_handle: thread_handle,
            });

            Ok(())
        }

        /// Send a request and block until the plugin responds (30 s timeout).
        pub fn send_request(&self, request: RpcRequest) -> Result<RpcResponse, PluginError> {
            let handle = self.handle.as_ref().ok_or(PluginError::PluginNotStarted)?;
            handle.send_rpc(request)?;
            handle.recv_response(Duration::from_secs(30))
        }
    }
}

pub fn base_command(python_path: &Path) -> Command {
    let mut cmd = Command::new(python_path);

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("PYTHONUNBUFFERED", "1")
        .env("PYTHONNOUSERSITE", "1");

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    // In release builds, set PYTHONHOME so the bundled interpreter finds its stdlib.
    #[cfg(not(debug_assertions))]
    {
        if let Some(parent) = python_path.parent() {
            if parent.file_name().and_then(|n| n.to_str()) == Some("bin") {
                if let Some(root) = parent.parent() {
                    cmd.env("PYTHONHOME", root.to_string_lossy().to_string());
                }
            }
        }
    }

    cmd
}

// TODO: This is meant as a way to PythonRuntime close threads of plugins that stopped working for any reason.
// This is not used yet but it's something that must be done before Beta Release.
pub enum PluginEvent {
    Exited,
}
