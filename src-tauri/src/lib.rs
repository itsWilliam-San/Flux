use std::{collections::HashMap, sync::Arc};
use tauri::WebviewWindow;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::plugins::{PluginDescriptor, PluginManifest, PluginOrigin};

struct AppData {
    sockets: SocketMap,
}

type SocketMap = Arc<Mutex<HashMap<String, (TcpStream, PluginManifest)>>>;

const ADDRESS: &str = "127.0.0.1:8080";
const READ_BYTES_SOCKET: usize = 4096;

mod network;
mod plugins;

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
    // 1. Verify that this is not a bridge plugin
    {
        let sockets = state.sockets.lock().await;
        if sockets.contains_key(&plugin_id) {
            return Err(
                "Bridge plugins must be communicated with over their TCP connection.".to_string(),
            );
        }
    }

    // 2. Discover local plugins to find matching ID
    let local_plugins = plugins::loader::discover_local_plugins(&app);
    let target = local_plugins
        .into_iter()
        .find(|p| p.manifest.id == plugin_id)
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

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

    let payload = serde_json::json!({
        "action": export.entrypoint,
        "parameters": parameters.unwrap_or(serde_json::json!({}))
    });

    plugins::python_runtime::PythonRuntime::execute_action(
        Some(&app),
        &plugin_dir,
        &export.entrypoint,
        &payload,
    )
    .await
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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            let plugins_i = MenuItem::with_id(app, "plugins", "Plugin", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = if !cfg!(debug_assertions) {
                let restart_i =
                    MenuItem::with_id(app, "restart", "Restart Flux", true, None::<&str>)?;

                Menu::with_items(
                    app,
                    &[&plugins_i, &separator, &restart_i, &settings_i, &quit_i],
                )?
            } else {
                Menu::with_items(app, &[&plugins_i, &separator, &settings_i, &quit_i])?
            };

            let socket_map = open_socket();
            app.manage(AppData {
                sockets: socket_map,
            });

            let plugins = app.get_webview_window("plugins").unwrap();
            let settings = app.get_webview_window("settings").unwrap();

            let _ = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "plugins" => {
                        toggle_window(&plugins);
                    }
                    "settings" => toggle_window(&settings),
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
            get_plugins_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn handshake(socket: &mut TcpStream) -> Result<PluginManifest, ()> {
    let mut buf = Vec::with_capacity(READ_BYTES_SOCKET);

    let n = socket.read(&mut buf).await.map_err(|_| ())?;

    if n == 0 {
        return Err(());
    }

    let manifest: PluginManifest = serde_json::from_slice(&buf).map_err(|_| ())?;

    Ok(manifest)
}

fn open_socket() -> SocketMap {
    let sockets = Arc::new(Mutex::new(HashMap::new()));

    let local_sockets = sockets.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let listener = TcpListener::bind(ADDRESS).await.unwrap();
            println!("Server running on {}", ADDRESS);

            loop {
                let (mut socket, addr) = listener.accept().await.unwrap();

                if let Ok(manifest) = handshake(&mut socket).await {
                    println!("Client connected: {}", addr);
                    let mut guard = local_sockets.lock().await;
                    guard.insert(manifest.id.clone(), (socket, manifest));
                } else {
                    break;
                }
            }
        });
    });

    sockets
}
