use crate::network::System;
use crate::plugins::plugin::PluginDescription;
use crate::plugins::python_runtime::PythonRuntime;
use log::{error, warn};
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri::{App, WebviewWindow};

use tokio::sync::Mutex;

use crate::error::{JsonRpcError, PluginError};
use crate::plugins::rpc::{RpcId, RpcParams, RpcRequest, RpcResponse};
use crate::plugins::PluginExport;

pub struct AppData {
    pub network: Arc<Mutex<Option<network::System>>>,
    pub runtime: PythonRuntime,
}

impl AppData {
    pub async fn update_network_data(&self, system: Option<System>) {
        let mut net_guard = self.network.lock().await;
        *net_guard = system;
    }
}

// TODO This probably should change to a config file in XDG_CONFIG_HOME or smth.
// Later I'll need to implement a database to hold the network data anyway so
// let's keep it like this 'till that day ig.
const NETWORK_UPDATE_EVENT: &str = "network-data-changed";
const SHOULD_UPDATE_PLUGIN_LIST_EVENT: &str = "should-update-plugin-list";

mod error;
mod network;
mod plugins;

fn try_to_response(buffer: &Vec<u8>, n: usize) -> Result<RpcResponse, JsonRpcError> {
    let response: RpcResponse = serde_json::from_slice(&buffer[..n])
        .map_err(|e| JsonRpcError::NotValidRPCResponse(e.to_string()))?;

    Ok(response)
}

#[tauri::command]
fn list_plugins(state: tauri::State<'_, AppData>) -> Vec<PluginDescription> {
    state.runtime.get_plugins()
}

async fn set_plugin_parameters(
    add_network_data: bool,
    parameters: Option<serde_json::Value>,
    state: &tauri::State<'_, AppData>,
) -> RpcParams {
    let mut params_map = serde_json::Map::new();

    // TODO Maybe this needs some rework, so we don't depend much on raw string
    params_map.insert(
        "parameters".to_string(),
        parameters.unwrap_or(serde_json::json!({})),
    );

    let _ = if add_network_data {
        let net = state.network.lock().await;
        if let Some(ref system) = *net {
            if let Ok(net_val) = serde_json::to_value(system) {
                params_map.insert("network".to_string(), net_val);
            }
        }
    };

    params_map.into()
}

/// Check if the action requires any prior data (Like the Network Data)
/// If it does, return an error, otherwise an Ok
async fn may_run(
    export: &PluginExport,
    state: &tauri::State<'_, AppData>,
) -> Result<(), PluginError> {
    let require_network_data = export.require_network_data.unwrap_or(false);

    if require_network_data {
        let net = state.network.lock().await;
        if net.is_none() {
            return Err(PluginError::RequiredNetworkDataNotFound);
        }
    }

    Ok(())
}

fn create_plugin_log_file(plugin_id: &str, log_dir: &Path) -> std::fs::File {
    let log_path = log_dir.join("plugins");
    fs::create_dir_all(&log_path).expect("Failed to create the plugins log directory");

    let safe_id: String = plugin_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let log_path = log_path.join(format!("{safe_id}.log"));

    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .append(true)
        .open(log_path)
        .expect("Failed to create the plugins log file")
}

async fn execute_local_action(
    state: tauri::State<'_, AppData>,
    parameters: Option<serde_json::Value>,
    export: &PluginExport,
) -> Result<serde_json::Value, PluginError> {
    let is_exporter = export.update_network_data.unwrap_or(false);

    let params = if is_exporter || parameters.is_some() {
        let params = set_plugin_parameters(is_exporter, parameters, &state).await;
        Some(params)
    } else {
        None
    };

    let payload = RpcRequest::new(export.entrypoint, params, Some(RpcId::Number(1)));
    let entrypoint = export.entrypoint;

    let result = state.runtime.execute_action(&export.id, &payload);

    let result = match response {
        RpcResponse::Error { error, .. } => serde_json::Value::from(RpcError),
        RpcResponse::Success { result, .. } => {
            let system = serde_json::from_value(result)
                .map_err(|e| InternalPluginError::IncorrectNetworkData(e.to_string()))?;

            if is_exporter {
                state.update_network_data(system).await;
                let _ = app.emit(NETWORK_UPDATE_EVENT, true);
            }

            serde_json::Value::Null
        }
        RpcResponse::Notification => serde_json::Value::Null,
    };

    Ok(result)
}

/// plugin_id = Identification of that plugin as written in its manifest
/// action: PluginExport.id
/// parameters: Parameters of that action as described in its manifest
#[tauri::command]
async fn execute_plugin_action(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppData>,
    plugin_id: String,
    action: String,
    parameters: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let local_plugins = plugins::loader::discover_local_plugins(&app).map_err(|e| e.to_string())?;

    let target = local_plugins
        .into_iter()
        .find(|p| p.manifest.id == plugin_id)
        .ok_or(PluginError::PluginNotFound.to_string())?;

    let plugin_dir = std::path::PathBuf::from(plugin_dir_str);
    // Get Function

    let result = execute_local_action(state, parameters, &export)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result_value)
}

#[tauri::command]
async fn has_network_data(state: tauri::State<'_, AppData>) -> Result<bool, String> {
    let guard = state.network.lock().await;
    Ok(guard.is_some())
}

#[tauri::command]
async fn get_network_data(
    state: tauri::State<'_, AppData>,
) -> Result<Option<network::System>, String> {
    let guard = state.network.lock().await;
    Ok(guard.clone())
}

#[tauri::command]
async fn set_network_data(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppData>,
    system: network::System,
) -> Result<(), String> {
    let mut guard = state.network.lock().await;
    *guard = Some(system);
    let _ = app.emit(NETWORK_UPDATE_EVENT, true);
    Ok(())
}

/// Clean the current network data
#[tauri::command]
async fn clear_network_data(
    state: tauri::State<'_, AppData>,
    app: tauri::AppHandle,
) -> Result<(), ()> {
    state.update_network_data(None).await;

    let _ = app.emit(NETWORK_UPDATE_EVENT, false);
    Ok(())
}

#[tauri::command]
async fn set_window_decorations(app: tauri::AppHandle, decorations: bool) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("plugins") {
        match w.set_decorations(decorations) {
            Ok(_) => {}
            Err(e) => warn!("Failed to set window decorations. {}", e.to_string()),
        }
    }

    warn!("Failed to get 'Plugins' webview window.");
    Ok(()) // TODO Analyse if an Ok return is ok in this case.
}

#[tauri::command]
async fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn get_plugins_folder(state: tauri::State<'_, AppData>) -> String {
    state
        .runtime
        .plugin_folder_root
        .to_string_lossy()
        .to_string()
}

fn toggle_window(window: &WebviewWindow) {
    if let Ok(is_visible) = window.is_visible() {
        if !is_visible {
            let _ = window.show();
        }
        let _ = window.set_focus();
        let _ = window.unminimize();
    }
}

/// Create and set up a logger backend for Env.
/// A line should look like [2026-01-01 10:10:00] [DEBUG] [flux_lib] - A **really** important stuff.
fn set_log(log_dir: &Path) {
    fs::create_dir_all(&log_dir).expect("Failed to create the log directory");

    let log_file_path = log_dir.join("flux.log");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(log_file_path).expect("Failed to open log file"))
        .apply()
        .expect("Failed to start global logger");
}

fn set_tray(app: &mut App) {
    let plugins_i = MenuItem::with_id(app, "plugins", "Plugin", true, None::<&str>)
        .expect("Failed to create the 'Plugin' Menu Item for the Tray System");
    let separator = PredefinedMenuItem::separator(app)
        .expect("Failed to create the 'Separator' Menu Item for the Tray System");
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .expect("Failed to create the 'Quit' Menu Item for the Tray System");

    let menu = Menu::with_items(app, &[&plugins_i, &separator, &quit_i])
        .expect("Failed to create the Menu for the Tray System");

    let plugins = app
        .get_webview_window("plugins")
        .expect("Failed to find the 'plugin' webview window");

    let _ = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "plugins" => {
                toggle_window(&plugins);
            }
            "restart" => app.request_restart(),
            _ => {}
        })
        .build(app)
        .unwrap_or_else(|e| panic!("Failed to create the TrayIcon. {}", e.to_string()));
}

pub fn run() {
    // Some stuff here should probably be delayed to later, like the plugins loadings
    // I think it improves the UX so the user know that Flux is running, but it's loading the plugins
    // and other necessary stuff, if something fails.
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            let log_dir = app.path().app_log_dir()?;

            set_log(&log_dir);
            set_tray(&mut app);

            let network = Arc::new(Mutex::new(None));

            let resource_dir = app
                .path()
                .resource_dir()
                .inspect_err(|e| error!("Failed to find the Resources path: {:?}", e))?;

            let mut plugin_dir = app.path().app_local_data_dir()?;
            plugin_dir.join("plugins");

            let runtime = PythonRuntime::new(resource_dir, plugin_dir)
                .inspect_err(|e| error!("Python Runtime Failed to Start. {:?}", e))?;

            runtime.load_plugins();

            app.manage(AppData { runtime, network });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_plugins,
            execute_plugin_action,
            get_plugins_folder,
            has_network_data,
            get_network_data,
            set_network_data,
            set_window_decorations,
            exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
