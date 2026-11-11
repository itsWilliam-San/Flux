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
