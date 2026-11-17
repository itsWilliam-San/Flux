use crate::network::System;
use crate::plugins::python_runtime::PythonRuntime;
use log::{debug, error, info, warn};
use std::fs;
use std::path::PathBuf;
use std::{collections::HashMap, sync::Arc};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri::{App, WebviewWindow};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::error::{JsonRpcError, PluginError, TransportError};
use crate::plugins::rpc::{RpcId, RpcParams, RpcRequest, RpcResponse};
use crate::plugins::{PluginDescriptor, PluginExport, PluginManifest, PluginOrigin};

pub struct AppData {
    // TODO
    pub plugins: Vec<Box<dyn Plugin>>,

    pub sockets: SocketMap,
    pub network: Arc<Mutex<Option<network::System>>>,
    pub python_runtime: PathBuf,
}

impl AppData {
    pub async fn update_network_data(&self, system: Option<System>) {
        let mut net_guard = self.network.lock().await;
        *net_guard = system;
    }
}

// TODO Remove this when finished
type SocketMap = Arc<Mutex<HashMap<String, (TcpStream, PluginManifest)>>>;

// TODO This probably should change to a config file in XDG_CONFIG_HOME or smth.
// Later I'll need to implement a database to hold the network data anyway so
// let's keep it like this 'till that day ig.
const ADDRESS: &str = "127.0.0.1:8080";
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
async fn list_plugins(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppData>,
) -> Result<Vec<PluginDescriptor>, String> {
    let sockets = state.sockets.lock().await;

    let mut plugins: Vec<PluginDescriptor> = sockets
        .values()
        .map(|(_, manifest)| PluginDescriptor {
            origin: PluginOrigin::Bridge,
            manifest: manifest.clone(),
            path: None,
        })
        .collect();

    // Discover local Python plugins
    let local_plugins = plugins::loader::discover_local_plugins(&app).map_err(|e| e.to_string())?;
    plugins.extend(local_plugins);

    Ok(plugins)
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

    RpcParams::new(params_map)
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

    // Send Function
    let response = plugins::python_runtime::PythonRuntime::execute_action(
        &state,
        &plugin_dir,
        &entrypoint,
        &payload,
    )
    .await
    .map_err(|e| e.to_string())?;
    // Send function

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

async fn execute_bridge_action(
    state: tauri::State<'_, AppData>,
    export: &PluginExport,
    parameters: Option<serde_json::Value>,
) -> Result<serde_json::Value, PluginError> {
    may_run(export, &state).await?;

    let should_add_network_data = export.require_network_data.unwrap_or(false);
    let params_map = set_plugin_parameters(should_add_network_data, parameters, &state).await;

    let id = RpcId::Number(1); // TODO Check if we actually need a ID for RPC. If we should, we must change this so ID is actually unique.

    let payload = RpcRequest::new(export.entrypoint, params_map, Some(id));

    let result_value = match rpc_response {
        RpcResponse::Error { error, .. } => return Err(error),
        RpcResponse::Success { result, .. } => result,
    };

    let is_extraction = export.update_network_data.unwrap_or(false);

    if is_extraction {
        let system = serde_json::from_value::<network::System>(result_value.to_owned())
            .map_err(|e| PluginError::NotANetworkData(e.to_string()))?;

        state.update_network_data(Some(system)).await;
    }

    return Ok(result_value);
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
    let is_bridge = {
        let sockets = state.sockets.lock().await;
        sockets.contains_key(&plugin_id)
    };

    // TODO Maybe we can make these two function just one and use Traits/Generics since they share a lot of common behaviour
    // Both need a PluginManifest, the way this is done differs, but when they get it it's all the same
    // Both need to send an action, if it's a bridge plugin, this is done through TCP, but if it's a local plugin, it'll use the PythonRuntime.
    // The start and end of these two functions changes, but all the rest remains the same
    // Both uses JSON-RPC to send commands, they both share the same way to send the parameters received fron the Frontend, and etc...
    let result_value = if is_bridge {
        // Get Function
        let mut sockets_guard = state.sockets.lock().await;
        let (socket, manifest) = sockets_guard
            .get_mut(&plugin_id)
            .ok_or_else(|| PluginError::PluginDisconnected(plugin_id).to_string())?;

        let export = manifest
            .get_export(&action)
            .ok_or(PluginError::ActionNotFound.to_string())?;
        //  Get Function
        execute_bridge_action(state, export, parameters)
            .await
            .map_err(|e| e.to_string())?
    } else {
        // Get Function
        let local_plugins =
            plugins::loader::discover_local_plugins(&app).map_err(|e| e.to_string())?;

        let target = local_plugins
            .into_iter()
            .find(|p| p.manifest.id == plugin_id)
            .ok_or(PluginError::PluginNotFound.to_string())?;

        let plugin_dir = std::path::PathBuf::from(plugin_dir_str);
        // Get Function

        execute_local_action(state, parameters, &export)
            .await
            .map_err(|e| e.to_string())?
    };

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
fn get_plugins_folder(app: tauri::AppHandle) -> Result<String, String> {
    let result = plugins::loader::get_user_plugins_dir(&app)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string());

    result
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
fn set_log(app_handle: &App) {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get the DataDir directory for Logging");

    fs::create_dir_all(&app_dir).expect("Failed to create the log directory");

    let log_file_path = app_dir.join("debug.log");

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
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            set_log(&app);
            set_tray(&mut app);

            let handle = app.handle();

            let network = Arc::new(Mutex::new(None));
            let socket_map = open_socket(network.clone(), handle.clone());

            let python_runtime_path = PythonRuntime::resolve_python_executable(handle)
                .expect("Failed to find a valid Python Executable");

            app.manage(AppData {
                sockets: socket_map,
                python_runtime: python_runtime_path,
                network,
            });

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

async fn handshake(socket: &mut TcpStream) -> Result<PluginManifest, PluginError> {
    let received_bytes = read_socket_payload(socket).await?;
    let test = received_bytes.as_slice();

    let manifest: PluginManifest =
        serde_json::from_slice(test).map_err(|e| PluginError::InvalidManifest(e.to_string()))?;

    Ok(manifest)
}

fn open_socket(
    _network: Arc<Mutex<Option<network::System>>>,
    _app_handle: tauri::AppHandle,
) -> SocketMap {
    let sockets = Arc::new(Mutex::new(HashMap::new()));

    let local_sockets = sockets.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let listener = match TcpListener::bind(ADDRESS).await {
                Ok(l) => l,
                Err(err) => {
                    error!("Could not bind socket on {}: {}", ADDRESS, err);
                    return;
                }
            };

            loop {
                let (mut socket, addr) = match listener.accept().await {
                    Ok(conn) => {
                        info!("New client: {:?}", conn.1);
                        conn
                    }
                    Err(e) => {
                        error!("Couldn't get client: {:?}", e);
                        break;
                    }
                };

                match handshake(&mut socket).await {
                    Ok(manifest) => {
                        info!(
                            "Handshake done with plugin: '{}' (ID: {}) v{} from {:?}",
                            manifest.name, manifest.id, manifest.version, addr
                        );

                        let mut guard = local_sockets.lock().await;
                        guard.insert(manifest.id.clone(), (socket, manifest));

                        // Tell Flux to update the plugins list
                        let _ = _app_handle.emit(SHOULD_UPDATE_PLUGIN_LIST_EVENT, true);
                    }
                    Err(e) => {
                        warn!("Handshake with client {:?} failed.", addr);
                        debug!("Detailed handshake error for {:?}: {:?}", addr, e);
                    }
                }
            }
        });
    });

    sockets
}

// TODO Write actually tests.
