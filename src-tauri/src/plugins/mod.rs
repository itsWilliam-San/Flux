pub mod loader;
pub mod python_runtime;
pub mod rpc;

use serde::{Deserialize, Serialize};

use crate::{
    error::PluginError,
    plugins::{
        python_runtime::PythonRuntime,
        rpc::{RpcRequest, RpcResponse},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Vec<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpcPayload {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<PluginParameter>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionExecutionPayload {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

pub mod plugin {
    use std::{
        io::{BufRead, BufReader, Write},
        path::{Path, PathBuf},
        process::{ChildStdout, Command, Stdio},
        sync::mpsc::{self, Receiver, Sender},
        thread,
    };

    use crate::{
        error::AppError,
        plugins::{loader::discover_local_plugins, rpc::RpcError},
    };

    use super::*;

    pub struct Plugin {
        description: PluginDescription,
        handle: PluginHandle,
    }

    struct PluginHandle {
        stdin_sender: Sender<RpcRequest>,
        stdou_receiver: Receiver<RpcResponse>,
        thread_handle: thread::JoinHandle<()>,
    }

    pub struct PluginDescription {
        pub manifest: PluginManifest,
        pub path: PathBuf,
    }

    impl Plugin {
        /// This function is meant to create a file that stores all erros that this plugin may have printed using stderr.

        ///  This Function is meant to start a plugin at the first time.
        pub fn start(
            description: PluginDescription,
            runtime: &PythonRuntime,
            log_dir: &Path,
        ) -> Result<Self, AppError> {
            let (tx_req, rx_req) = mpsc::channel::<RpcRequest>();
            let (tx_res, rx_res) = mpsc::channel::<RpcResponse>();

            let path = description.path.clone();
            let mut base_cmd = runtime.base_command(path);

            let thread_handle = thread::spawn(move || {
                let mut child = base_cmd
                    .arg(&path)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to execute Child Plugin");

                let mut stdin = child.stdin.take().expect("Failed to open stdin");
                let stdout = child.stdout.take().expect("Failed to open stdout");

                // Sub-thread opcional para ler o stdout do Python sem bloquear a escrita
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
                                eprintln!("Not a valid payload from plugin. {err}");
                            }
                        }
                    }
                });

                // Loop principal da thread: aguarda requisições RPC para enviar ao stdin do processo
                while let Ok(request) = rx_req.recv() {
                    if let Ok(json_string) = serde_json::to_string(&request) {
                        if let Err(e) = writeln!(stdin, "{}", json_string) {
                            eprintln!("Erro ao escrever no stdin do plugin: {}", e);
                            break;
                        }

                        if let Err(e) = stdin.flush() {
                            eprintln!("Erro ao dar flush no stdin: {}", e);
                            break;
                        }
                    }
                }
            });

            Ok(Self {
                description,
                handle: PluginHandle {
                    stdin_sender: tx_req,
                    stdou_receiver: rx_res,
                    thread_handle,
                },
            })
        }

        /// Envia uma requisição RPC para o plugin de forma assíncrona/não-bloqueante via canal
        pub fn send_rpc(&self, payload: RpcRequest) -> Result<(), PluginError> {
            self.handle
                .stdin_sender
                .send(payload)
                .map_err(|_| PluginError::ChannelClosed)?;
            Ok(())
        }
    }
}
