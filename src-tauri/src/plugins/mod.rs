pub mod loader;
pub mod python_runtime;
pub mod rpc;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use crate::{
    error::{PluginError, TransportError}, plugins::{python_runtime::PythonRuntime, rpc::{RpcRequest, RpcResponse}},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginOrigin {
    Local,
    Bridge,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginDescriptor {
    pub origin: PluginOrigin,
    pub manifest: PluginManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Vec<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub id: String,
    pub exports: Vec<PluginExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginExport {
    pub id: String,
    pub label: String,
    pub description: String,
    pub entrypoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<PluginParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_network_data: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_network_data: Option<bool>,
}

impl PluginManifest {
    pub fn get_export(&self, export_id: &str) -> Option<&PluginExport> {
        self.exports.iter().find(|export| export.id == export_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginParameter {
    pub name: String,
    #[serde(alias = "paramType", alias = "type", rename = "param_type")]
    pub param_type: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpcPayload {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<PluginParameter>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionExecutionPayload {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

trait Plugin {
    async fn send_rpc(&mut self, payload: &RpcRequest) -> Result<RpcResponse, PluginError>;
}

struct BridgePlugin {
    socket: TcpStream,
    manifest: PluginManifest,
}

struct LocalPlugin {
    manifest: PluginManifest,
    path: PathBuf,
}

const READ_BYTES_SOCKET: usize = 4096;

impl BridgePlugin {
    // TODO Depending on the size of the Network Data, this might not be nigh enough for holding all the content
    // so it'll be necessary in the future to create and manage a data base.
    // I don't know if I'll add it in the 1.0 or maybe 2.0 since for my use case (and the others working with this software)
    // 4kb might be enough.
    async fn read_socket_payload(&self) -> Result<Vec<u8>, TransportError> {
        let mut buf = vec![0u8; READ_BYTES_SOCKET];

        let n = self.socket.read(&mut buf).await?;

        if n == 0 {
            return Err(TransportError::ConnectionClosed);
        }

        // TODO: This creates unecessary allocation but improves DX ig.
        // I need to use cargo-flamegraph or smth so I may know if this is acceptable or not.
        // Maybe this won't be enough and some real use-case might be more usefull idk.
        // Create an issue later if this is a real problem and we refactor it. For know let's keep it like this
        let buf = buf[..n].to_owned();

        Ok(buf)
    }
}

impl Plugin for BridgePlugin {
    async fn send_rpc(&mut self, payload: &RpcRequest) -> Result<RpcResponse, PluginError> {
        let mut payload_bytes = serde_json::to_vec(&self)?;
        payload_bytes.push(b'\n');

        self.socket.write_all(&payload_bytes).await?;
        self.socket.flush().await?;

        let raw_payload = self.read_socket_payload().await?;

        let rpc_response = RpcResponse::new(&raw_payload)?;
        Ok(rpc_response)
    }
}

impl Plugin for LocalPlugin {
    async fn send_rpc(&mut self, payload: &RpcRequest) -> Result<RpcResponse, PluginError> {
        let export = self
            .manifest
            .exports
            .into_iter()
            .find(|export| export.entrypoint == payload.method)
            .ok_or(PluginError::ActionNotFound)?;

        PythonRuntime::execute_action(state, plugin_dir, entrypoint, payload)
    }
}
