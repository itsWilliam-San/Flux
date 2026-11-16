use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Network I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection closed prematurely by peer (0 bytes read).")]
    ConnectionClosed,

    #[error("Failed to deserialize handshake JSON manifest: {0}")]
    Json(#[from] serde_json::Error),
}

/// This is similar to InternalPluginError
/// but while the InternalPluginError handles script errors (Like incorrect message passing)
/// This kind of error handles incorrect structure of the PluginError, like missing a manifest.json, or a main.py
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Couldn't find main.py file.")]
    MainFileNotFound,

    #[error("Manifest not found")]
    ManifestNotFound,

    #[error("Plugin not found")]
    PluginNotFound,

    #[error("Requested action was not found in the manifest.")]
    ActionNotFound,
}

#[derive(Error, Debug)]
pub enum PythonRuntimeError {
    #[error("Plugin error {0}")]
    PluginError(#[from] PluginError),

    #[error("Internal Plugin error {0}")]
    InternalPluginError(#[from] InternalPluginError),

    #[error("Plugin execution timed out.")]
    ScriptTimeOut,

    #[error("Error waiting for Python Process {0}")]
    PythonProcessWaitingError(String),

    #[error("I/O Error {0}")]
    Io(#[from] std::io::Error),

    #[error("Python executable hasn't been found.")]
    NoPythonExecutableFound,
}

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("Transport error during handshake: {0}")]
    Transport(#[from] TransportError),
}

#[derive(Error, Debug)]
pub enum JsonRpcError {
    #[error("Not a valid RPC parameters structure.")]
    NotValidRPCParams,

    #[error("Not a valid JSON-RPC response message. {0}")]
    NotValidRPCResponse(String),

    #[error("RPC bridge connection was closed.")]
    BridgeConnectionClosed,
}

// Captures any error that a Plugin may have, like incorrent IPC messages, internal error, Handshake error,
// or anything that might go wrong with a third-party script.
#[derive(Error, Debug)]
pub enum InternalPluginError {
    #[error("A Tauri error occurred: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("Handshake error: {0}")]
    Handshake(#[from] HandshakeError),

    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("RPC communication error: {0}")]
    Rpc(#[from] JsonRpcError),

    #[error("The requested plugin '{0}' is disconnected.")]
    PluginDisconnected(String),

    #[error("Action requires a loaded hydraulic system first.")]
    RequiredNetworkDataNotFound,

    #[error("The 'value' data is not a valid network data object, {0}")]
    NotANetworkData(String),

    #[error("Internal error while executing Python script. {0}")]
    InternalError(String),

    #[error("The received network data is not in the correct format. {0}")]
    IncorrectNetworkData(String),
}
