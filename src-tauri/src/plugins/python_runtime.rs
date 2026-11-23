use log::warn;

use crate::plugins::loader::find_plugins;
use crate::plugins::plugin::{Plugin, PluginDescription};
use crate::plugins::rpc::RpcRequest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::PythonRuntimeError::{self};

const PYTHON_RUNTIME_FOLDER: &str = "python-runtime";
const PYTHON_MAIN_FILE: &str = "main.py";

pub struct PythonRuntime {
    python_path: PathBuf,
    pub plugin_folder_root: PathBuf,
    plugins: HashMap<String, Plugin>,
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
    Err(PythonRuntimeError::PythonExecutableNotFound)
}

impl PythonRuntime {
    pub fn get_user_plugins_dir(&self) -> &Path {
        let plugin_folder = &self.plugin_folder_root;

        plugin_folder
    }

    pub fn get_plugins(&self) -> Vec<PluginDescription> {
        self.plugins
            .iter()
            .map(|(_, plugin)| plugin.description.clone())
            .collect()
    }

    pub fn new(
        python_path: PathBuf,
        plugin_folder_root: PathBuf,
    ) -> Result<Self, PythonRuntimeError> {
        let python_path = resolve_python_executable(python_path)?;
        let plugins = HashMap::new();

        Ok(Self {
            python_path,
            plugin_folder_root,
            plugins,
        })
    }

    pub fn execute_plugin_action(
        &self,
        plugin_id: &str,
        payload: RpcRequest,
    ) -> Result<(), PythonRuntimeError> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or(PythonRuntimeError::PluginNotFound(plugin_id.to_owned()))?;

        plugin.send_request(payload)?;

        todo!()
    }

    pub fn load_plugins(&mut self) {
        let found_plugins = find_plugins(self.plugin_folder_root.clone()).expect("FIX IT");

        for plugin in found_plugins {
            self.plugins
                .insert(plugin.manifest.id.clone(), Plugin::new(plugin));
        }
    }

    pub fn start_plugins(&mut self, log_dir: &Path) {
        let python_path = &self.python_path;

        for (plugin_id, plugin) in &mut self.plugins {
            let cmd = base_command(python_path);
            let _ = plugin.start(cmd, log_dir).inspect_err(|err| {
                warn!(
                    "Failed to initialize plugin {}. Reason: {}",
                    plugin_id,
                    err.to_string()
                );
            });
        }
    }

    pub fn get_plugin_response(&self, id: &str) -> _ {
        todo!()
    }
}

fn base_command(python_path: &Path) -> Command {
    let mut cmd = Command::new(python_path);

    cmd.arg("-c")
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
