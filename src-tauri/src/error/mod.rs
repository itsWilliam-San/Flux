use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("A Tauri error occurred: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("Python executable hasn't been found.")]
    NoPythonExecutableFound,

    #[error("Couldn't access the plugins root directory: {0}")]
    PluginsRootDirectoryNotFound(#[from] std::io::Error),
}

/// Handles raw network transport, socket reads, and initial payload parsing.
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Network I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection closed prematurely by peer (0 bytes read).")]
    ConnectionClosed,

    #[error("Failed to deserialize JSON payload: {0}")]
    Json(#[from] serde_json::Error),
}

/// Errors specific to the JSON-RPC communication protocol.
#[derive(Error, Debug)]
pub enum JsonRpcError {
    #[error("Not a valid RPC parameters structure.")]
    NotValidRPCParams,

    #[error("Not a valid JSON-RPC response message: {0}")]
    NotValidRPCResponse(String),

    #[error("RPC bridge connection was closed.")]
    BridgeConnectionClosed,
}

/// Handles structural validation of the plugin directory (files and manifests).
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Handshake error: {0}")]
    Handshake(#[from] TransportError),

    #[error("RPC communication error: {0}")]
    Rpc(#[from] JsonRpcError),

    #[error("The requested plugin '{0}' is disconnected.")]
    PluginDisconnected(String),

    #[error("Action requires a loaded hydraulic system first.")]
    RequiredNetworkDataNotFound,

    #[error("The 'value' data is not a valid network data object: {0}")]
    NotANetworkData(String),

    #[error("Internal error while executing Python script: {0}")]
    InternalError(String),

    #[error("The received network data is not in the correct format: {0}")]
    IncorrectNetworkData(String),

    #[error("Couldn't find main.py file.")]
    MainFileNotFound,

    #[error("Manifest not found.")]
    ManifestNotFound,

    #[error("Plugin not found.")]
    PluginNotFound,

    #[error("Requested action was not found in the manifest.")]
    ActionNotFound,

    #[error("The received manifest is not correct. {0}")]
    InvalidManifest(String),
}

/// Top-level errors managing the Python execution lifecycle.
#[derive(Error, Debug)]
pub enum PythonRuntimeError {
    #[error("Plugin structure or execution error: {0}")]
    Plugin(#[from] PluginError),

    #[error("Plugin execution timed out.")]
    ScriptTimeOut,

    #[error("Error waiting for Python process: {0}")]
    PythonProcessWaitingError(String),

    #[error("Process I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Couldn't find a valid Python Executable.")]
    PythonExecutableNotFound,
}
