use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IpfsRemote {
    #[wasm_bindgen(getter_with_clone)]
    pub host: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Ipfs {
    Bundled,
    Remote(IpfsRemote),
}

impl Default for Ipfs {
    fn default() -> Self {
        Self::Bundled
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct S3StateStore {
    #[wasm_bindgen(getter_with_clone)]
    pub bucket: String,
    #[wasm_bindgen(getter_with_clone)]
    pub endpoint: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StateStore {
    S3(S3StateStore),
    LocalDirectory(PathBuf),
}

impl Default for StateStore {
    fn default() -> Self {
        Self::LocalDirectory(PathBuf::from("/etc/ceramic/data"))
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HttpApi {
    #[wasm_bindgen(getter_with_clone)]
    pub hostname: String,
    pub port: u16,
    #[wasm_bindgen(skip)]
    pub cors_allowed_origins: Vec<String>,
    #[wasm_bindgen(skip)]
    pub admin_dids: Vec<String>,
}

impl Default for HttpApi {
    fn default() -> Self {
        Self {
            hostname: std::net::Ipv4Addr::LOCALHOST.to_string(),
            port: 80,
            cors_allowed_origins: vec![],
            admin_dids: vec![],
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Network {
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub pubsub_topic: String,
}

impl Default for Network {
    fn default() -> Self {
        Self::dev()
    }
}

impl Network {
    pub fn local(name: &str) -> Self {
        Self {
            name: format!("local-{}", name),
            pubsub_topic: format!("/ceramic/local-topic-{}", name),
        }
    }

    pub fn dev() -> Self {
        Self {
            name: "dev-unstable".to_string(),
            pubsub_topic: "/ceramic/dev-unstable".to_string(),
        }
    }

    pub fn clay() -> Self {
        Self {
            name: "testnet-clay".to_string(),
            pubsub_topic: "/ceramic/testnet-clay".to_string(),
        }
    }

    pub fn mainnet() -> Self {
        Self {
            name: "mainnet".to_string(),
            pubsub_topic: "/ceramic/mainnet".to_string(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Anchor {
    #[wasm_bindgen(getter_with_clone)]
    pub anchor_service_url: String,
    #[wasm_bindgen(getter_with_clone)]
    pub ethereum_rpc_url: String,
}

impl Default for Anchor {
    fn default() -> Self {
        Self::dev()
    }
}

impl Anchor {
    pub fn local() -> Self {
        Self {
            anchor_service_url: "https://cas-qa.3boxlabs.com/".to_string(),
            ethereum_rpc_url: "http://localhost:7545".to_string(),
        }
    }

    pub fn dev() -> Self {
        Self {
            anchor_service_url: "https://cas-qa.3boxlabs.com/".to_string(),
            ethereum_rpc_url: "http://localhost:7545".to_string(),
        }
    }

    pub fn clay() -> Self {
        Self {
            anchor_service_url: "https://cas-clay.3boxlabs.com/".to_string(),
            ethereum_rpc_url: "http://localhost:7545".to_string(),
        }
    }

    pub fn mainnet() -> Self {
        Self {
            anchor_service_url: "https://cas.3boxlabs.com/".to_string(),
            ethereum_rpc_url: "http://localhost:7545".to_string(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Indexing {
    #[wasm_bindgen(getter_with_clone)]
    pub db: String,
}

impl Default for Indexing {
    fn default() -> Self {
        Self {
            db: "pg://ceramic@localhost:5432/ceramic".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DidResolvers {
    Safe(HashMap<String, serde_json::Value>),
    Ethr(HashMap<String, serde_json::Value>),
}

impl Default for DidResolvers {
    fn default() -> Self {
        Self::Ethr(HashMap::default())
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub gateway: bool,
    pub sync_override: bool,
    pub stream_cache_limit: usize,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            gateway: false,
            sync_override: false,
            stream_cache_limit: 100,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileLogger {
    pub enabled: bool,
    pub directory: PathBuf,
}

impl Default for FileLogger {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: PathBuf::from("/var/log/ceramic"),
        }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Logger {
    pub file: Option<FileLogger>,
    pub level: LogLevel,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            file: Some(FileLogger::default()),
            level: LogLevel::default(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Metrics {
    pub enabled: bool,
    #[wasm_bindgen(getter_with_clone)]
    pub host: String,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "???".to_string(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[wasm_bindgen(skip)]
    pub ipfs: Ipfs,
    #[wasm_bindgen(skip)]
    pub state_store: StateStore,
    #[wasm_bindgen(getter_with_clone)]
    pub http_api: HttpApi,
    #[wasm_bindgen(getter_with_clone)]
    pub network: Network,
    #[wasm_bindgen(getter_with_clone)]
    pub anchor: Anchor,
    #[wasm_bindgen(getter_with_clone)]
    pub indexing: Indexing,
    #[wasm_bindgen(skip)]
    pub did_resolvers: DidResolvers,
    #[wasm_bindgen(getter_with_clone)]
    pub node: Node,
    #[wasm_bindgen(skip)]
    pub logger: Logger,
    #[wasm_bindgen(getter_with_clone)]
    pub metrics: Metrics,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmFileLogger {
    pub enabled: bool,
    #[wasm_bindgen(getter_with_clone)]
    pub directory: String,
}

#[wasm_bindgen]
pub struct WasmLogger {
    #[wasm_bindgen(getter_with_clone)]
    pub file: Option<WasmFileLogger>,
    pub level: LogLevel,
}

#[wasm_bindgen]
pub struct WasmIpfs {
    #[wasm_bindgen(getter_with_clone)]
    pub remote: Option<IpfsRemote>,
}

#[wasm_bindgen]
pub struct WasmStateStore {
    #[wasm_bindgen(getter_with_clone)]
    pub local_directory: Option<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub s3: Option<S3StateStore>,
}

#[wasm_bindgen]
impl Config {
    pub fn ipfs(&self) -> WasmIpfs {
        if let Ipfs::Remote(remote) = &self.ipfs {
            WasmIpfs {
                remote: Some(remote.clone()),
            }
        } else {
            WasmIpfs { remote: None }
        }
    }

    pub fn cors_allowed_origins(&self) -> Vec<JsValue> {
        self.http_api
            .cors_allowed_origins
            .iter()
            .map(JsValue::from)
            .collect()
    }

    pub fn admin_dids(&self) -> Vec<JsValue> {
        self.http_api.admin_dids.iter().map(JsValue::from).collect()
    }

    pub fn state_store(&self) -> WasmStateStore {
        match &self.state_store {
            StateStore::LocalDirectory(dir) => WasmStateStore {
                local_directory: Some(dir.to_string_lossy().to_string()),
                s3: None,
            },
            StateStore::S3(s3) => WasmStateStore {
                local_directory: None,
                s3: Some(s3.clone()),
            },
        }
    }

    pub fn eth_resolver_options(&self) -> Option<String> {
        if let DidResolvers::Ethr(m) = &self.did_resolvers {
            Some(serde_json::to_string(m).unwrap_or_else(|_| String::default()))
        } else {
            None
        }
    }

    pub fn safe_resolver_options(&self) -> Option<String> {
        if let DidResolvers::Safe(m) = &self.did_resolvers {
            Some(serde_json::to_string(m).unwrap_or_else(|_| String::default()))
        } else {
            None
        }
    }

    pub fn logger(&self) -> WasmLogger {
        WasmLogger {
            level: self.logger.level,
            file: self.logger.file.as_ref().map(|f| WasmFileLogger {
                enabled: f.enabled,
                directory: f.directory.to_string_lossy().to_string(),
            }),
        }
    }
}

pub fn from_file_err(file: String) -> anyhow::Result<Config> {
    let data = std::fs::read(PathBuf::from(file))?;
    Ok(serde_json::from_slice(data.as_slice())?)
}

pub fn from_string_err(json: &str) -> anyhow::Result<Config> {
    Ok(serde_json::from_str(json)?)
}

#[wasm_bindgen]
pub fn from_file(file: String) -> Result<Config, JsValue> {
    from_file_err(file).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen]
pub fn from_string(json: String) -> Result<Config, JsValue> {
    from_string_err(&json).map_err(|e| JsValue::from(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_default_config() {
        let js = serde_json::to_string(&Config::default()).unwrap();
        assert_eq!(
            &js,
            r#"{"ipfs":"Bundled","state_store":{"LocalDirectory":"/etc/ceramic/data"},"http_api":{"hostname":"127.0.0.1","port":80,"cors_allowed_origins":[],"admin_dids":[]},"network":{"name":"???","pubsub_topic":"???"},"anchor":{"anchor_service_url":"???","ethereum_rpc_url":"???"},"indexing":{"db":"???"},"did_resolvers":{"Ethr":{}},"node":{"gateway":false,"sync_override":false,"stream_cache_limit":100},"logger":{"file":{"enabled":true,"directory":"/var/log/ceramic"},"level":"Info"},"metrics":{"enabled":false,"host":"???"}}"#
        );
        let _: Config = serde_json::from_str(&js).unwrap();
    }
}
