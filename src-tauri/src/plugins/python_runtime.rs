use crate::error::{JsonRpcError, PluginError};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::error::PythonRuntimeError::{self};
use crate::plugins::rpc::{RpcRequest, RpcResponse};

const PYTHON_RUNTIME_FOLDER: &str = "python-runtime";
const PYTHON_MAIN_FILE: &str = "main.py";

pub struct PythonRuntime {
    pub path: PathBuf,
}

fn resolve_python_executable(resource_dir: &Path) -> Result<PathBuf, PythonRuntimeError> {
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
        //  bundled resource directory
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
    Err(PythonRuntimeError::NoPythonExecutableFound)
}

impl PythonRuntime {
    pub fn new(bundle_directory: &Path) -> Result<Self, PythonRuntimeError> {
        let path = resolve_python_executable(bundle_directory)?;

        Ok(Self { path })
    }

    pub fn start_plugin(&self, plugin_dir: &Path, method: &str) {
        let command = self.base_command(plugin_dir);

        let main_file = plugin_dir
            .join(PYTHON_MAIN_FILE)
            .to_string_lossy()
            .to_string();

        command.arg(main_file)
    }

    pub fn base_command(&self, plugin_dir: &Path) -> Command {
        let mut cmd = Command::new(&self.path);

        cmd.arg("-c")
            .current_dir(plugin_dir)
            .stdin(std::process::Stdio::piped())
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

        #[cfg(debug_assertions)]
        {
            if let Some(parent) = self.path.parent() {
                if parent.file_name().and_then(|n| n.to_str()) == Some("bin") {
                    if let Some(root) = parent.parent() {
                        cmd.env("PYTHONHOME", root.to_string_lossy().to_string());
                    }
                }
            }
        }

        cmd
    }

    /// Executes an entrypoint for a local plugin using the isolated Python runtime.
    pub async fn execute_action(
        &self,
        plugin_dir: &Path,
        payload: &RpcRequest,
    ) -> Result<RpcResponse, PythonRuntimeError> {
        let script_file = plugin_dir.join(PYTHON_MAIN_FILE);

        if !script_file.is_file() {
            return Err(PythonRuntimeError::MainFileNotFound);
        }

        let payload_str = serde_json::to_string(payload).expect("This shouldn't fail");

        let cmd = self
            .base_command(plugin_dir)
            .arg(script_file.to_string_lossy().to_string())

        let mut child = cmd.spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(payload_str.as_bytes()).await?;
        }

        let timeout_duration = Duration::from_secs(30);
        let output = tokio::time::timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| PythonRuntimeError::ScriptTimeOut)?
            .map_err(|e| PythonRuntimeError::PythonProcessWaitingError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(PythonRuntimeError::PluginError(PluginError::InternalError(
                stderr,
            )));
        }

        if stdout.trim().is_empty() {
            return Ok(RpcResponse::Notification);
        }

        let response = serde_json::from_str::<RpcResponse>(&stdout)
            .map_err(|e| PluginError::Rpc(JsonRpcError::NotValidRPCResponse(e.to_string())))?;

        Ok(response)
    }
}
