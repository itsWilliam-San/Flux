use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tauri::Manager;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const RUNNER_SCRIPT: &str = r#"
import sys
import json
import importlib.util
from pathlib import Path

def main():
    if len(sys.argv) < 3:
        sys.stderr.write("Usage: runner.py <plugin_dir> <entrypoint>\n")
        sys.exit(1)

    plugin_dir = Path(sys.argv[1]).resolve()
    entrypoint = sys.argv[2]

    if ":" not in entrypoint:
        sys.stderr.write(f"Invalid entrypoint format: {entrypoint}. Expected 'file.py:function_name'\n")
        sys.exit(1)

    file_name, func_name = entrypoint.split(":", 1)
    target_file = plugin_dir / file_name

    if not target_file.is_file():
        sys.stderr.write(f"Plugin entry file not found: {target_file}\n")
        sys.exit(1)

    if str(plugin_dir) not in sys.path:
        sys.path.insert(0, str(plugin_dir))

    # Read stdin payload
    try:
        raw_in = sys.stdin.read()
        payload = json.loads(raw_in) if raw_in.strip() else {}
    except Exception as e:
        sys.stderr.write(f"Failed to parse stdin JSON payload: {e}\n")
        sys.exit(1)

    # Import module
    try:
        spec = importlib.util.spec_from_file_location("plugin_entry_module", str(target_file))
        if spec is None or spec.loader is None:
            sys.stderr.write(f"Could not load module specification from {target_file}\n")
            sys.exit(1)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
    except Exception as e:
        sys.stderr.write(f"Error loading module {file_name}: {e}\n")
        sys.exit(1)

    func = getattr(mod, func_name, None)
    if func is None or not callable(func):
        sys.stderr.write(f"Function '{func_name}' not found or not callable in {file_name}\n")
        sys.exit(1)

    try:
        result = func(payload)
        if result is None:
            result = {"status": "success"}
        sys.stdout.write(json.dumps(result))
    except Exception as e:
        sys.stderr.write(f"Exception during execution of {func_name}: {e}\n")
        sys.exit(1)

if __name__ == "__main__":
    main()
"#;

pub struct PythonRuntime;

impl PythonRuntime {
    /// Attempts to locate the portable Python executable across Linux, Windows, and macOS.
    /// Checks resource directory, local workspace ./python-runtime, exe directory, then system PATH.
    pub fn resolve_python_executable(app: Option<&tauri::AppHandle>) -> Result<PathBuf, String> {
        let mut candidate_dirs: Vec<PathBuf> = Vec::new();

        // 1. Tauri bundled resource directory
        if let Some(handle) = app {
            if let Ok(res_dir) = handle.path().resource_dir() {
                candidate_dirs.push(res_dir.join("python-runtime"));
                candidate_dirs.push(res_dir);
            }
        }

        // 2. Current working directory / project root
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(parent) = cwd.parent() {
                candidate_dirs.push(parent.join("python-runtime"));
            }
        }

        // 3. Executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                candidate_dirs.push(exe_dir.join("python-runtime"));
            }
        }

        // Check for platform-specific binary locations inside candidates
        for dir in candidate_dirs {
            // Linux & macOS paths
            let unix_bin1 = dir.join("bin").join("python3");
            let unix_bin2 = dir.join("bin").join("python");
            if unix_bin1.is_file() {
                return Ok(unix_bin1);
            }
            if unix_bin2.is_file() {
                return Ok(unix_bin2);
            }

            // Windows paths
            let win_bin1 = dir.join("python.exe");
            let win_bin2 = dir.join("Scripts").join("python.exe");
            if win_bin1.is_file() {
                return Ok(win_bin1);
            }
            if win_bin2.is_file() {
                return Ok(win_bin2);
            }
        }

        Err("No Python executable found. Please ensure the bundled portable Python runtime is installed in python-runtime.".to_string())
    }

    /// Executes an entrypoint for a local plugin using the isolated Python runtime.
    pub async fn execute_action(
        app: Option<&tauri::AppHandle>,
        plugin_dir: &Path,
        entrypoint: &str,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let python_exe = Self::resolve_python_executable(app)?;

        // Ensure entrypoint script exists
        let parts: Vec<&str> = entrypoint.split(':').collect();
        if parts.is_empty() {
            return Err(format!("Invalid entrypoint: {}", entrypoint));
        }
        let script_file = plugin_dir.join(parts[0]);
        if !script_file.is_file() {
            return Err(format!(
                "Plugin entry file not found: {}",
                script_file.display()
            ));
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

        // If running from an isolated portable runtime, set PYTHONHOME if appropriate
        if let Some(parent) = python_exe.parent() {
            if parent.file_name().and_then(|n| n.to_str()) == Some("bin") {
                if let Some(root) = parent.parent() {
                    cmd.env("PYTHONHOME", root.to_string_lossy().to_string());
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
                "status": "success",
                "message": "Action completed with no output"
            }));
        }

        match serde_json::from_str::<serde_json::Value>(&stdout) {
            Ok(json_val) => Ok(json_val),
            Err(_) => Ok(serde_json::json!({
                "status": "success",
                "output": stdout.trim()
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_python_action_execution() {
        let temp_dir = std::env::temp_dir().join(format!("sewerage_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let script_content = r#"
def run_test(payload):
    params = payload.get("parameters", {})
    return {
        "status": "success",
        "greeting": f"Hello {params.get('name', 'World')}"
    }
"#;
        fs::write(temp_dir.join("main.py"), script_content).unwrap();

        let payload = serde_json::json!({
            "action": "main.py:run_test",
            "parameters": { "name": "Antigravity" }
        });

        let result = PythonRuntime::execute_action(None, &temp_dir, "main.py:run_test", &payload)
            .await
            .expect("Action execution failed");

        assert_eq!(result["status"], "success");
        assert_eq!(result["greeting"], "Hello Antigravity");

        let _ = fs::remove_dir_all(temp_dir);
    }
}
