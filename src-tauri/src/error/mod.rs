use thiserror::Error;

// TODO: Change description to EN-US later.
#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("Erro de I/O na rede durante o handshake: {0}")]
    Io(#[from] std::io::Error),

    #[error("Conexao fechada pelo peer prematuramente (0 bytes lidos)")]
    ConnectionClosed,

    #[error("Falha ao desserializar o manifesto JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Erro de I/O Durante leitura do diretorio de Plugins: {0}")]
    Io(#[from] std::io::Error),

    #[error("Falha ao desserializar o manifest JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("O Plugin {0} requesitado está desconectado.")]
    PluginDisconnected(String),

    #[error("Ação requisitada não foi encontrada no manifest")]
    ActionNotFound,

    #[error("A ação {0} necessita de um sistema hidraulico carregado primeiro.")]
    RequiredNetworkDataNotFound,
}

#[derive(Error, Debug)]
pub enum PythonRuntimeError {
    #[error("Python exectuable haven't been found.")]
    NoPythonExecutableFound,

    #[error("Couldn't find main.py file")]
    MainFileNotFound,
}
