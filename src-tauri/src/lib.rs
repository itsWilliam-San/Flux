use log::{debug, error, info, warn};
use std::fs;
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

use crate::error::HandshakeError;
use crate::plugins::{PluginDescriptor, PluginManifest, PluginOrigin};

pub struct AppData {
    pub sockets: SocketMap,
    pub network: Arc<Mutex<Option<network::System>>>,
}

type SocketMap = Arc<Mutex<HashMap<String, (TcpStream, PluginManifest)>>>;

const ADDRESS: &str = "127.0.0.1:8080";
const READ_BYTES_SOCKET: usize = 4096;

mod error;
mod network;
mod plugins;

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
    let local_plugins = plugins::loader::discover_local_plugins(&app);
    plugins.extend(local_plugins);

    Ok(plugins)
}

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

    if is_bridge {
        let mut sockets_guard = state.sockets.lock().await;
        let (socket, manifest) = sockets_guard
            .get_mut(&plugin_id)
            .ok_or_else(|| format!("Bridge plugin '{}' disconnected", plugin_id))?;

        let is_exporter = matches!(manifest.category, plugins::PluginCategory::Exporter);
        let is_extraction = matches!(manifest.category, plugins::PluginCategory::Extraction);

        if is_exporter {
            let net = state.network.lock().await;
            if net.is_none() {
                return Err(
                    "Cannot execute export plugin: No hydraulic network data is loaded. Please import network data first."
                        .to_string(),
                );
            }
        }

        let export = manifest
            .exports
            .iter()
            .find(|e| e.id == action || e.entrypoint == action)
            .ok_or_else(|| format!("Action '{}' not found in plugin exports", action))?;

        // Vou provavelmente remover isso pois me parece uma camada a mais desencessária.
        let mut params_map = serde_json::Map::new();
        params_map.insert(
            "parameters".to_string(),
            parameters.unwrap_or(serde_json::json!({})),
        );

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

        let mut payload_bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
        payload_bytes.push(b'\n');

        socket
            .write_all(&payload_bytes)
            .await
            .map_err(|e| format!("Failed to send data to bridge: {}", e))?;
        socket
            .flush()
            .await
            .map_err(|e| format!("Failed to flush data to bridge: {}", e))?;

        let mut buf = vec![0u8; 65536];
        let n = socket
            .read(&mut buf)
            .await
            .map_err(|e| format!("Failed to read data from bridge: {}", e))?;
        if n == 0 {
            return Err("Bridge connection closed by peer".to_string());
        }

        let raw_response: serde_json::Value = serde_json::from_slice(&buf[..n])
            .map_err(|e| format!("Invalid JSON response from bridge: {}", e))?;

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

    let is_exporter = matches!(target.manifest.category, plugins::PluginCategory::Exporter);
    let is_extraction = matches!(
        target.manifest.category,
        plugins::PluginCategory::Extraction
    );

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
        Some(&app),
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
        let _ = w.set_decorations(decorations);
    }
    Ok(())
}

#[tauri::command]
async fn exit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn get_plugins_folder(app: tauri::AppHandle) -> Result<String, String> {
    plugins::loader::get_user_plugins_dir(&app).map(|p| p.to_string_lossy().to_string())
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
                "[{}] [{}] [{}] - {}\n",
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

    info!("Flux started. Log system activated");
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            set_log(&app);

            let plugins_i = MenuItem::with_id(app, "plugins", "Plugin", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = if !cfg!(debug_assertions) {
                let restart_i =
                    MenuItem::with_id(app, "restart", "Restart Flux", true, None::<&str>)?;

                Menu::with_items(app, &[&plugins_i, &separator, &restart_i, &quit_i])?
            } else {
                Menu::with_items(app, &[&plugins_i, &separator, &quit_i])?
            };

            let network = Arc::new(Mutex::new(None));
            let socket_map = open_socket(network.clone(), Some(app.handle().clone()));
            app.manage(AppData {
                sockets: socket_map,
                network,
            });

            let plugins = app.get_webview_window("plugins").unwrap();

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
                .build(app)?;

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

async fn handshake(socket: &mut TcpStream) -> Result<PluginManifest, HandshakeError> {
    let mut buf = vec![0u8; READ_BYTES_SOCKET];

    let n = socket.read(&mut buf).await?;

    if n == 0 {
        error!("Conection closed by the client during the Handhsake fase");
        return Err(HandshakeError::ConnectionClosed);
    }

    let received_bytes = &buf[..n];

    debug!("{}", String::from_utf8_lossy(received_bytes));

    let manifest: PluginManifest = serde_json::from_slice(received_bytes)?;

    Ok(manifest)
}

fn open_socket(
    _network: Arc<Mutex<Option<network::System>>>,
    _app_handle: Option<tauri::AppHandle>,
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
                            "Handhskae done with plugin: '{}' (ID: {}) v{} from {:?}",
                            manifest.name, manifest.id, manifest.version, addr
                        );

                        let mut guard = local_sockets.lock().await;
                        guard.insert(manifest.id.clone(), (socket, manifest));
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_system_from_various_envelopes() {
        let sample_system_json = serde_json::json!({
            "Primary": [
                {
                    "ID": "PV-01",
                    "TerrainLevel": 100.0,
                    "InvertLevel": 50.0,
                    "Depth": 50.0
                }
            ],
            "Intermediates": [],
            "Segments": []
        });

        // 1. Direct
        let sys1 = extract_system_from_value(&sample_system_json);
        assert!(sys1.is_ok());
        assert_eq!(sys1.unwrap().primaries[0].id.0, "PV-01");

        // 2. Wrapped in "system"
        let wrapped_sys = serde_json::json!({
            "status": "success",
            "system": sample_system_json
        });
        let sys2 = extract_system_from_value(&wrapped_sys);
        assert!(sys2.is_ok());
        assert_eq!(sys2.unwrap().primaries[0].id.0, "PV-01");

        // 3. Wrapped in "network"
        let wrapped_net = serde_json::json!({
            "status": "success",
            "network": sample_system_json
        });
        let sys3 = extract_system_from_value(&wrapped_net);
        assert!(sys3.is_ok());
        assert_eq!(sys3.unwrap().primaries[0].id.0, "PV-01");
    }

    #[tokio::test]
    async fn test_import_and_export_plugins_flow() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let plugins_dir = root.join("plugins");

        let importer_dir = plugins_dir.join("network-importer");
        let exporter_dir = plugins_dir.join("excel-exporter");

        if !importer_dir.is_dir() || !exporter_dir.is_dir() {
            eprintln!("Plugins directories not found, skipping plugin execution test");
            return;
        }

        // 1. Execute Network Importer
        let import_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "main.py:import_network",
            "params": {
                "parameters": { "source": "sample_network" }
            },
            "id": 1
        });

        let import_response = plugins::python_runtime::PythonRuntime::execute_action(
            None,
            &importer_dir,
            "main.py:import_network",
            &import_payload,
        )
        .await
        .expect("Import action failed");

        let import_result = import_response
            .get("result")
            .cloned()
            .unwrap_or(import_response);
        let extracted_system = extract_system_from_value(&import_result)
            .expect("Could not extract system from importer response");

        assert_eq!(extracted_system.primaries.len(), 2);
        assert_eq!(extracted_system.primaries[0].id.0, "PV-01");
        assert_eq!(extracted_system.intermediates.len(), 1);

        // 2. Try executing Excel Exporter WITHOUT network -> must fail
        let empty_export_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "main.py:run_detailed_export",
            "params": {
                "parameters": { "separator": ";" }
            },
            "id": 2
        });

        let fail_result = plugins::python_runtime::PythonRuntime::execute_action(
            None,
            &exporter_dir,
            "main.py:run_detailed_export",
            &empty_export_payload,
        )
        .await;

        assert!(
            fail_result.is_err(),
            "Exporter should fail when no network is provided"
        );

        // 3. Execute Excel Exporter WITH imported network -> must succeed with real stations
        let net_val = serde_json::to_value(&extracted_system).unwrap();
        let export_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "main.py:run_detailed_export",
            "params": {
                "parameters": { "separator": ";", "include_headers": true },
                "network": net_val.clone(),
                "system": net_val.clone()
            },
            "id": 3
        });

        let export_response = plugins::python_runtime::PythonRuntime::execute_action(
            None,
            &exporter_dir,
            "main.py:run_detailed_export",
            &export_payload,
        )
        .await
        .expect("Exporter action failed with valid network");

        let export_result = export_response
            .get("result")
            .cloned()
            .unwrap_or(export_response);
        assert_eq!(export_result["status"], "success");
        assert_eq!(export_result["rows_count"], 3);
        let preview = export_result["content_preview"].as_str().unwrap();
        assert!(preview.contains("PV-01"));
        assert!(preview.contains("PV-02"));
        assert!(preview.contains("L-01"));

        // 4. Test XLSX Export
        let xlsx_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "main.py:run_export_csv",
            "params": {
                "parameters": { "sheet_name": "TestPlanilha" },
                "network": net_val
            },
            "id": 4
        });

        let xlsx_response = plugins::python_runtime::PythonRuntime::execute_action(
            None,
            &exporter_dir,
            "main.py:run_export_csv",
            &xlsx_payload,
        )
        .await
        .expect("XLSX export action failed");

        let xlsx_result = xlsx_response
            .get("result")
            .cloned()
            .unwrap_or(xlsx_response);
        assert_eq!(xlsx_result["status"], "success");
        assert_eq!(xlsx_result["rows_count"], 3);
        assert_eq!(xlsx_result["sheet_name"], "TestPlanilha");
    }

    #[tokio::test]
    async fn test_bridge_socket_communication_flow() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Spawn mock client representing the external bridge plugin
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let n = socket.read(&mut buf).await.unwrap();
            let req: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
            assert_eq!(req["jsonrpc"], "2.0");
            assert_eq!(req["method"], "main.py:test_bridge_action");

            let res = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "status": "success",
                    "message": "Bridge action executed successfully"
                },
                "id": req["id"]
            });
            let res_bytes = serde_json::to_vec(&res).unwrap();
            socket.write_all(&res_bytes).await.unwrap();
        });

        // Backend side connection
        let mut client_stream = TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "main.py:test_bridge_action",
            "params": {},
            "id": 1
        });
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        client_stream.write_all(&payload_bytes).await.unwrap();

        let mut buf = vec![0u8; 4096];
        let n = client_stream.read(&mut buf).await.unwrap();
        let response: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
        assert_eq!(
            response["result"]["message"],
            "Bridge action executed successfully"
        );
    }
}
