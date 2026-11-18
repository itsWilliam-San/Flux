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
        process::{Command, Stdio},
        sync::mpsc::{self, Sender},
        thread,
    };

    use crate::{error::AppError, plugins::loader::discover_local_plugins};

    use super::*;

    pub struct Plugin {
        pub description: PluginDescription,
        handle: PluginHandle,
    }

    struct PluginHandle {
        stdin_sender: Sender<RpcRequest>,
        thread_handle: thread::JoinHandle<()>,
    }

    pub struct PluginDescription {
        pub manifest: PluginManifest,
        pub path: PathBuf,
    }

    impl Plugin {
        /// Inicializa o processo do plugin e suas threads de I/O (stdin / stdout)
        pub fn start(
            description: PluginDescription,
            runtime: &PythonRuntime,
        ) -> Result<Self, AppError> {
            let (tx, rx) = mpsc::channel::<RpcRequest>();

            let path = description.path.clone();
            let base_cmd = runtime.base_command(&path);

            let thread_handle = thread::spawn(move || {
                let mut child = base_cmd
                    .arg(&path)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to execute Child Plugin");

                let mut stdin = child.stdin.take().expect("Failed to open stdin");
                let stdout = child.stdout.take().expect("Failed to open stdout");

                // Sub-thread opcional para ler o stdout do Python sem bloquear a escrita
                let _stdout_thread = thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(response_str) = line {
                            // TODO: Aqui você faria o parse da string para RpcResponse
                            // e despacharia para o canal global do Tauri/Core se necessário.
                            println!("[Plugin Stdout]: {}", response_str);
                        } else {
                            break;
                        }
                    }
                });

                // Loop principal da thread: aguarda requisições RPC para enviar ao stdin do processo
                while let Ok(request) = rx.recv() {
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
                    stdin_sender: tx,
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

    /// Descobre e inicializa todos os plugins locais encontrados no diretório
    pub fn start_plugins(local_data_dir: &Path) -> Result<Vec<Plugin>, AppError> {
        let plugin_descriptions: Vec<PluginDescription> = discover_local_plugins(local_data_dir)?;
        let mut loaded_plugins = Vec::new();

        // Exemplo de runtime mockado ou injetado (ajuste conforme a sua struct PythonRuntime)
        let runtime = PythonRuntime::default();

        for desc in plugin_descriptions {
            match Plugin::start(desc, &runtime) {
                Ok(plugin) => loaded_plugins.push(plugin),
                Err(e) => {
                    eprintln!("Falha ao carregar plugin: {:?}", e);
                    // Como especificado: um plugin falhar não derruba os outros
                }
            }
        }

        Ok(loaded_plugins)
    }
}
