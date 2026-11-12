use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::error::PythonRuntimeError::{self, MainFileNotFound};
use crate::AppData;

const PYTHON_RUNTIME_FOLDER: &str = "python-runtime";
const PYTHON_RUNNER: &str = include_str!("./runner.py");
const PYTHON_MAIN_FILE: &str = "main.py";

pub struct PythonRuntime;

impl PythonRuntime {
    pub fn resolve_python_executable(
        app: &tauri::AppHandle,
    ) -> Result<PathBuf, PythonRuntimeError> {
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
            // 1. Tauri bundled resource directory
            if let Ok(res_dir) = app.path().resource_dir() {
                let dir = res_dir.join(PYTHON_RUNTIME_FOLDER);

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
        }

        Err(PythonRuntimeError::NoPythonExecutableFound)
    }

    /// Executes an entrypoint for a local plugin using the isolated Python runtime.
    pub async fn execute_action(
        state: &tauri::State<'_, AppData>,
        plugin_dir: &Path,
        entrypoint: &str,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, PythonRuntimeError> {
        let python_exe = state.python_runtime.clone();

        let script_file = plugin_dir.join(PYTHON_MAIN_FILE);

        if !script_file.is_file() {
            return Err(PythonRuntimeError::MainFileNotFound);
        }

        let payload_str = serde_json::to_string(payload)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;

        let mut cmd = Command::new(&python_exe);
        cmd.arg("-c")
            .arg(RUNNER_SCRIPT)
            .arg(plugin_dir.to_string_lossy().to_string())
            .arg(entrypoint)
            .current_dir(plugin_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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
            if let Some(parent) = python_exe.parent() {
                if parent.file_name().and_then(|n| n.to_str()) == Some("bin") {
                    if let Some(root) = parent.parent() {
                        cmd.env("PYTHONHOME", root.to_string_lossy().to_string());
                    }
                }
            }
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn Python process ({:?}): {}", python_exe, e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(payload_str.as_bytes())
                .await
                .map_err(|e| format!("Failed to write to Python stdin: {}", e))?;
        }

        let timeout_duration = Duration::from_secs(30);
        let output = tokio::time::timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| "Plugin execution timed out after 30 seconds".to_string())?
            .map_err(|e| format!("Error waiting for Python process: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(format!(
                "Python script execution failed with exit code {:?}.\nStderr:\n{}\nStdout:\n{}",
                output.status.code(),
                stderr.trim(),
                stdout.trim()
            ));
        }

        if stdout.trim().is_empty() {
            return Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "status": "success",
                    "message": "Action completed with no output"
                },
                "id": payload.get("id").cloned().unwrap_or(serde_json::json!(1))
            }));
        }

        match serde_json::from_str::<serde_json::Value>(&stdout) {
            Ok(json_val) => {
                if let Some(err_obj) = json_val.get("error") {
                    let msg = err_obj
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown JSON-RPC error");
                    return Err(msg.to_string());
                }
                Ok(json_val)
            }
            Err(_) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "status": "success",
                    "output": stdout.trim()
                },
                "id": payload.get("id").cloned().unwrap_or(serde_json::json!(1))
            })),
        }
    }
}
