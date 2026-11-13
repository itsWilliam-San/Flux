use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Network I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection closed prematurely by peer (0 bytes read).")]
    ConnectionClosed,
}

#[derive(Error, Debug)]
pub enum PythonRuntimeError {
    #[error("Python executable hasn't been found.")]
    NoPythonExecutableFound,

    #[error("Couldn't find main.py file.")]
    MainFileNotFound,
}

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("Transport error during handshake: {0}")]
    Transport(#[from] TransportError),

    #[error("Failed to deserialize handshake JSON manifest: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum JsonRpcError {
    #[error("Not a valid RPC parameters structure.")]
    NotValidRPCParams,

    #[error("Not a valid JSON-RPC response message.")]
    NotValidRPCResponse,

    #[error("RPC bridge connection was closed.")]
    BridgeConnectionClosed,
}

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("A Tauri error occurred: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("Python runtime error: {0}")]
    Python(#[from] PythonRuntimeError),

    #[error("Handshake error: {0}")]
    Handshake(#[from] HandshakeError),

    #[error("RPC communication error: {0}")]
    Rpc(#[from] JsonRpcError),

    #[error("The requested plugin '{0}' is disconnected.")]
    PluginDisconnected(String),

    #[error("Requested action was not found in the manifest.")]
    ActionNotFound,

    #[error("Action '{0}' requires a loaded hydraulic system first.")]
    RequiredNetworkDataNotFound(String),
}
