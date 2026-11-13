use crate::error::PluginError::{self, BridgeConnectionClosed};
use log::{debug, error, info, warn};
use serde_json::Map;
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
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::error::{HandshakeError, JsonRpcError, TransportError};
use crate::plugins::rpc::{RpcError, RpcId, RpcRequest, RpcResponse};
use crate::plugins::{PluginDescriptor, PluginExport, PluginManifest, PluginOrigin};

pub struct AppData {
    pub sockets: SocketMap,
    pub network: Arc<Mutex<Option<network::System>>>,
    pub python_runtime: PathBuf,
}

type SocketMap = Arc<Mutex<HashMap<String, (TcpStream, PluginManifest)>>>;

// TODO This probably should change to a config file in XDG_CONFIG_HOME or smth.
// Later I'll need to implement a database to hold the network data anyway so
// let's keep it like this 'till that day ig.
const ADDRESS: &str = "127.0.0.1:8080";
const READ_BYTES_SOCKET: usize = 4096;

mod error;
mod network;
mod plugins;

async fn send_buffer(message: Vec<u8>, socket: &mut TcpStream) -> Result<(), TransportError> {
    socket.write_all(&message).await?;
    socket.flush().await?;

    Ok(())
}

async fn read_buffer(
    buffer: &mut Vec<u8>,
    socket: &mut TcpStream,
) -> Result<usize, TransportError> {
    let n = socket.read(buffer).await?;

    if n == 0 {
        return Err(TransportError::ConnectionClosed);
    }

    Ok(n)
}

fn try_to_response(buffer: &Vec<u8>, n: usize) -> Result<RpcResponse, JsonRpcError> {
    let response: RpcResponse =
        serde_json::from_slice(&buffer[..n]).map_err(|_| JsonRpcError::NotValidRPCResponse)?;

    Ok(response)
}

fn extract_system_from_value(val: &serde_json::Value) -> Result<network::System, String> {
    if let Some(inner) = val.get("system") {
        let sys =
            serde_json::from_value::<network::System>(inner.clone()).map_err(|e| e.to_string())?;

        return Ok(sys);
    }

    return Err("No field \"system\" found inside the received response".into());
}

#[tauri::command]
async fn list_plugins(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppData>,
) -> Result<Vec<PluginDescriptor>, PluginError> {
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
    let local_plugins = plugins::loader::discover_local_plugins(&app)?;
    plugins.extend(local_plugins);

    Ok(plugins)
}

async fn set_plugin_parameters(
    add_network_data: bool,
    parameters: Option<serde_json::Value>,
    state: &tauri::State<'_, AppData>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut params_map = serde_json::Map::new();
    params_map.insert(
        "parameters".to_string(),
        parameters.unwrap_or(serde_json::json!({})),
    );

    if add_network_data {
        let net = state.network.lock().await;
        if let Some(ref system) = *net {
            if let Ok(net_val) = serde_json::to_value(system) {
                params_map.insert("network".to_string(), net_val);
            }
        }
    }

    params_map
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
            return Err(PluginError::RequiredNetworkDataNotFound(format!(
                "The action {} required the Network Data pre-loaded before it can run",
                export.id
            )));
        }
    }

    Ok(())
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
) -> Result<serde_json::Value, PluginError> {
    let is_bridge = {
        let sockets = state.sockets.lock().await;
        sockets.contains_key(&plugin_id)
    };

    if is_bridge {
        let mut sockets_guard = state.sockets.lock().await;
        let (socket, manifest) = sockets_guard
            .get_mut(&plugin_id)
            .ok_or_else(|| PluginError::PluginDisconnected(plugin_id))?;

        let export = manifest
            .get_export(&action)
            .ok_or(PluginError::ActionNotFound)?;

        may_run(export, &state).await?;

        let should_add_network_data = export.require_network_data.unwrap_or(false);
        let params_map = set_plugin_parameters(should_add_network_data, parameters, &state).await;

        let id = RpcId::Number(1); // TODO Check if we actually need a ID for RPC. If we should, we must change this so ID is actually unique.
        let payload = RpcRequest::new(action, params_map, Some(id));

        // TODO continue from here
        let mut payload_bytes = serde_json::to_vec(&payload)?;
        payload_bytes.push(b'\n');

        send_buffer(payload_bytes, &mut socket).await?;

        let mut buf = vec![0u8; 65536];
        read_buffer(&mut buf, &mut socket).await?;

        // TODO This code tries to detect if it's an Error Message or an RpcResponse
        // This should be put as a function probably.
        if let Some(err_val) = raw_response.get("error") {
            let err_msg = err_val
                .get("message")
                .and_then(|m| m.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| err_val.to_string());
            return Err(err_msg);
        }

        let result_value = raw_response
            .get("result")
            .cloned()
            .unwrap_or_else(|| raw_response.clone());

        if is_extraction {
            let system = extract_system_from_value(&result_value)?;
            let mut net_guard = state.network.lock().await;
            *net_guard = Some(system);
            let _ = app.emit("network-data-changed", true);
        }

        return Ok(result_value);
    }

    // 2. Discover local plugins to find matching ID
    let local_plugins = plugins::loader::discover_local_plugins(&app);
    let target = local_plugins
        .into_iter()
        .find(|p| p.manifest.id == plugin_id)
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

    // If it's an export plugin, block execution if no network data is stored
    if is_exporter {
        let net = state.network.lock().await;
        if net.is_none() {
            return Err(
                "Cannot execute export plugin: No hydraulic network data is loaded. Please import network data first."
                    .to_string(),
            );
        }
    }

    let plugin_dir_str = target
        .path
        .ok_or_else(|| format!("Plugin '{}' does not have a local path", plugin_id))?;
    let plugin_dir = std::path::PathBuf::from(plugin_dir_str);

    // Verify the action exists in exports
    let export = target
        .manifest
        .exports
        .iter()
        .find(|e| e.id == action || e.entrypoint == action)
        .ok_or_else(|| format!("Action '{}' not found in plugin exports", action))?;

    // Build JSON-RPC 2.0 params map
    let mut params_map = serde_json::Map::new();
    params_map.insert(
        "parameters".to_string(),
        parameters.unwrap_or(serde_json::json!({})),
    );

    // If network data exists, attach it to params
    {
        let net = state.network.lock().await;
        if let Some(ref system) = *net {
            if let Ok(net_val) = serde_json::to_value(system) {
                params_map.insert("network".to_string(), net_val);
            }
        }
    }

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": export.entrypoint,
        "params": serde_json::Value::Object(params_map),
        "id": 1
    });

    let raw_response = plugins::python_runtime::PythonRuntime::execute_action(
        &state,
        &plugin_dir,
        &export.entrypoint,
        &payload,
    )
    .await?;

    let result_value = raw_response
        .get("result")
        .cloned()
        .unwrap_or_else(|| raw_response.clone());

    // If extraction/import plugin, save returned network into AppData
    if is_extraction {
        let system = extract_system_from_value(&result_value)?;
        let mut net_guard = state.network.lock().await;
        *net_guard = Some(system);
        let _ = app.emit("network-data-changed", true);
    }

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
    let _ = app.emit("network-data-changed", true);
    Ok(())
}

fn should_update_plugin_list(app: &tauri::AppHandle) -> Result<(), String> {
    let _ = app.emit("should-update-plugin-list", true);
    Ok(())
}

/// Clean the current network data
#[tauri::command]
async fn clear_network_data(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppData>,
) -> Result<(), String> {
    let mut guard = state.network.lock().await;
    *guard = None;
    let _ = app.emit("network-data-changed", false);
    Ok(())
}

#[tauri::command]
async fn set_window_decorations(app: tauri::AppHandle, decorations: bool) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("plugins") {
        match w.set_decorations(decorations) {
            Ok(_) => {}
            Err(e) => warn!("Failed to set window decorations. {}", e.into()),
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

            let network = Arc::new(Mutex::new(None));
            let socket_map = open_socket(network.clone(), app.handle().clone());

            let python_runtime_path = PythonRuntime::resolve_python_executable(&app);

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

async fn handshake(socket: &mut TcpStream) -> Result<PluginManifest, HandshakeError> {
    let mut buf = vec![0u8; READ_BYTES_SOCKET];

    let n = socket.read(&mut buf).await?;

    if n == 0 {
        error!("Conection closed by the client during the Handhsake fase");
        return Err(HandshakeError::ConnectionClosed);
    }

    let received_bytes = &buf[..n];

    let manifest: PluginManifest = serde_json::from_slice(received_bytes)?;

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
                        should_update_plugin_list(&app);
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
