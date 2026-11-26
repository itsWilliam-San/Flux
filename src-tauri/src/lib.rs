use crate::{network::System};
use crate::plugins::plugin::PluginDescription;
use crate::plugins::python_runtime::{PythonRuntime, PythonRuntimeEvents};
use log::{error, info, warn};
use std::{fs, thread};
use std::path::Path;
use std::sync::mpsc::Receiver;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri::{App, WebviewWindow};

use std::sync::Mutex;

use crate::error::PluginError;
use crate::plugins::rpc::{RpcId, RpcRequest, RpcResponse};
use crate::plugins::PluginExport;

pub struct AppData {
    pub network: Mutex<Option<network::System>>,
    pub runtime: Mutex<PythonRuntime>,
}

impl AppData {
    pub fn update_network_data(&self, system: Option<System>) {
        let mut net_guard = self.network.lock().unwrap_or_else(|e| e.into_inner());
        *net_guard = system;
    }
}

const NETWORK_UPDATE_EVENT: &str = "network-data-changed";

mod error;
mod network;
mod plugins;

#[tauri::command]
/// This functions is supposed to update the plugins list, fetching it again.
/// This should also list states like if a plugins is loaded or not
fn list_plugins(state: tauri::State<'_, AppData>) -> Vec<PluginDescription> {
    let mut runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner());
    runtime.load_plugins();
    runtime.start_plugins();

    runtime.get_plugins()
}

/// Build the params map for an RPC call, optionally embedding the current network data.
fn insert_network_data(
    parameters: &mut serde_json::Map<String, serde_json::Value>,
    state: &tauri::State<'_, AppData>,
) {
    let net = state.network.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref system) = *net {
        if let Ok(net_val) = serde_json::to_value(system) {
            parameters.insert("network".to_string(), net_val);
        }
    }
}

/// Guard: return an error if the export requires network data but none is loaded.
fn may_run(
    export: &PluginExport,
    state: &tauri::State<'_, AppData>,
) -> Result<(), PluginError> {
    if export.require_network_data.unwrap_or(false) {
        let net = state.network.lock().unwrap_or_else(|e| e.into_inner());
        if net.is_none() {
            return Err(PluginError::RequiredNetworkDataNotFound);
        }
    }
    Ok(())
}

/// plugin_id  — the `id` field from the plugin's manifest.json
/// action     — the `id` of the export entry to run
/// parameters — key/value map matching that export's declared parameters
/// TODO We should probably create another function but for notification.
#[tauri::command]
fn execute_plugin_action(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppData>,
    plugin_id: String,
    action: String,
    mut parameters: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<serde_json::Value, String> {
    let description = state
        .runtime
        .lock().unwrap_or_else(|e| e.into_inner())
        .get_plugin(&plugin_id)
        .ok_or_else(|| PluginError::PluginNotFound.to_string())?;

    let export = description
        .manifest
        .get_export(&action)
        .ok_or_else(|| PluginError::ActionNotFound.to_string())?
        .clone();

    may_run(&export, &state).map_err(|e| e.to_string())?;

    let needs_network = export.require_network_data.unwrap_or(false);

    if needs_network {
        if let Some(param) = &mut parameters {
            insert_network_data(param, &state);
        }
    }

    // This always creates a Request with ID 1.
    let payload = RpcRequest::new(
        export.entrypoint.clone(),
        parameters,
        Some(RpcId::Number(1)),
    );

    let response = state
        .runtime.lock().unwrap_or_else(|e| e.into_inner())
        .execute_plugin_action(&plugin_id, payload)
        .map_err(|e| e.to_string())?;

    let updates_network = export.update_network_data.unwrap_or(false);
    match response {
        RpcResponse::Error { error, .. } => Err(error.message),
        RpcResponse::Success { result, .. } => {
            if updates_network {
                match serde_json::from_value::<Option<network::System>>(result.clone()) {
                    Ok(system) => {
                        state.update_network_data(system);
                        let _ = app.emit(NETWORK_UPDATE_EVENT, true);
                    }
                    Err(e) => {
                        return Err(PluginError::IncorrectNetworkData(e.to_string()).to_string());
                    }
                }
            }
            Ok(result)
        }
        RpcResponse::Notification => Ok(serde_json::Value::Null),
    }
}

#[tauri::command]
fn has_network_data(state: tauri::State<'_, AppData>) -> Result<bool, String> {
    let guard = state.network.lock().unwrap_or_else(|e| e.into_inner());
    Ok(guard.is_some())
}

#[tauri::command]
fn get_network_data(
    state: tauri::State<'_, AppData>,
) -> Result<Option<network::System>, String> {
    let guard = state.network.lock().unwrap_or_else(|e| e.into_inner());
    Ok(guard.clone())
}

#[tauri::command]
fn set_network_data(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppData>,
    system: network::System,
) -> Result<(), String> {
    let mut guard = state.network.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(system);
    let _ = app.emit(NETWORK_UPDATE_EVENT, true);
    Ok(())
}

#[tauri::command]
fn clear_network_data(
    state: tauri::State<'_, AppData>,
    app: tauri::AppHandle,
) -> Result<(), ()> {
    state.update_network_data(None);
    let _ = app.emit(NETWORK_UPDATE_EVENT, false);
    Ok(())
}

#[tauri::command]
async fn set_window_decorations(app: tauri::AppHandle, decorations: bool) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("plugins") {
        if let Err(e) = w.set_decorations(decorations) {
            warn!("Failed to set window decorations: {e}");
        }
    } else {
        warn!("Failed to get 'plugins' webview window.");
    }
    Ok(())
}

#[tauri::command]
async fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn get_plugins_folder(state: tauri::State<'_, AppData>) -> String {
    state
        .runtime
        .lock().unwrap_or_else(|e| e.into_inner())
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

/// Initialise the fern logger. Log lines look like:
/// `[2026-01-01 10:10:00] [DEBUG] [flux_lib] - message`
fn set_log(log_dir: &Path) {
    fs::create_dir_all(log_dir).expect("Failed to create the log directory");
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
    let plugins_i = MenuItem::with_id(app, "plugins", "Plugins", true, None::<&str>)
        .expect("Failed to create 'Plugins' tray menu item");
    let separator = PredefinedMenuItem::separator(app).expect("Failed to create tray separator");
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .expect("Failed to create 'Quit' tray menu item");

    let menu = Menu::with_items(app, &[&plugins_i, &separator, &quit_i])
        .expect("Failed to create tray menu");

    let plugins_window = app
        .get_webview_window("plugins")
        .expect("Failed to find the 'plugins' webview window");

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "plugins" => toggle_window(&plugins_window),
            "restart" => app.request_restart(),
            _ => {}
        })
        .build(app)
        .unwrap_or_else(|e| panic!("Failed to create the tray icon: {e}"));
}


pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            let log_dir = app.path().app_log_dir()?;
            set_log(&log_dir);
            info!("Flux started. Log system activated.");

            set_tray(app);

            let network = Mutex::new(None);

            let resource_dir = app
                .path()
                .resource_dir()
                .inspect_err(|e| error!("Failed to find resources path: {e:?}"))?;

            // Fix: use push() so the path is actually modified in place.
            let mut plugin_dir = app.path().app_local_data_dir()?;
            plugin_dir.push("plugins");

            let (mut runtime, rx) = PythonRuntime::new(resource_dir, plugin_dir, log_dir)
                .inspect_err(|e| error!("Python runtime failed to start: {e:?}"))?;

            let handle = app.handle().clone();
            create_event_thread(rx, handle);

            // Load and start plugins before handing the runtime to Tauri's state.
            runtime.load_plugins();
            runtime.start_plugins();

            let runtime = Mutex::new(runtime);

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
            clear_network_data,
            set_window_decorations,
            exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_event_thread(rx: Receiver<PythonRuntimeEvents>, app: tauri::AppHandle) {
    let _ = thread::spawn(move || {
        while let Ok(message) = rx.recv() {
            match message {
                PythonRuntimeEvents::PluginStateChanged(plugin_id, state) => {
                    let id = plugin_id.0;
                    let _ = app.emit_to("plugins", "plugin-status-changed", (id.clone(), state))
                        .inspect_err(|e| warn!("Failed to send Plugin event from Plugin {id}. Reason: {e}"));
                }
            }
        }
    });
}
