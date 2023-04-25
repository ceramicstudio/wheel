mod convert;
mod daemon;

pub use daemon::DaemonConfig;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IpfsRemote {
    pub host: String,
}

impl Default for IpfsRemote {
    fn default() -> Self {
        Self {
            host: "/ipfs".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Ipfs {
    Bundled,
    Remote(IpfsRemote),
}

impl std::fmt::Display for Ipfs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundled => write!(f, "Bundled"),
            Self::Remote(_) => write!(f, "Remote"),
        }
    }
}

impl Default for Ipfs {
    fn default() -> Self {
        Self::Bundled
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct S3StateStore {
    pub bucket: String,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HttpApi {
    pub hostname: String,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub admin_dids: Vec<String>,
}

impl Default for HttpApi {
    fn default() -> Self {
        Self {
            hostname: std::net::Ipv4Addr::LOCALHOST.to_string(),
            port: 7071,
            cors_allowed_origins: vec![],
            admin_dids: vec![],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NetworkIdentifier {
    InMemory,
    Local,
    Dev,
    Clay,
    Mainnet,
}

impl std::fmt::Display for NetworkIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InMemory => write!(f, "InMemory"),
            Self::Local => write!(f, "Local"),
            Self::Dev => write!(f, "Dev"),
            Self::Clay => write!(f, "Clay"),
            Self::Mainnet => write!(f, "Mainnet"),
        }
    }
}

impl Default for NetworkIdentifier {
    fn default() -> Self {
        Self::Clay
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Network {
    pub id: NetworkIdentifier,
    pub pubsub_topic: Option<String>,
}

impl Default for Network {
    fn default() -> Self {
        Self::dev()
    }
}

impl Network {
    pub fn in_memory() -> Self {
        Self {
            id: NetworkIdentifier::InMemory,
            pubsub_topic: None,
        }
    }
    pub fn local(name: &str) -> Self {
        Self {
            id: NetworkIdentifier::Local,
            pubsub_topic: Some(format!("/ceramic/local-topic-{}", name)),
        }
    }

    pub fn dev() -> Self {
        Self {
            id: NetworkIdentifier::Dev,
            pubsub_topic: None, //"/ceramic/dev-unstable".to_string(),
        }
    }

    pub fn clay() -> Self {
        Self {
            id: NetworkIdentifier::Clay,
            pubsub_topic: None, //"/ceramic/testnet-clay".to_string(),
        }
    }

    pub fn mainnet() -> Self {
        Self {
            id: NetworkIdentifier::Mainnet,
            pubsub_topic: None, //"/ceramic/mainnet".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Anchor {
    None,
    Ip {
        url: String,
    },
    RemoteDid {
        url: String,
        private_seed_url: String,
    },
}

impl Default for Anchor {
    fn default() -> Self {
        Self::None
    }
}

impl Anchor {
    pub fn url_for_network(id: &NetworkIdentifier) -> Option<String> {
        match id {
            NetworkIdentifier::InMemory | NetworkIdentifier::Local => None,
            NetworkIdentifier::Dev => Some("https://cas-qa.3boxlabs.com/".to_string()),
            NetworkIdentifier::Clay => Some("https://cas-clay.3boxlabs.com/".to_string()),
            NetworkIdentifier::Mainnet => Some("https://cas.3boxlabs.com/".to_string()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Indexing {
    pub db: String,
    pub allow_queries_before_historical_sync: bool,
    pub enable_historical_sync: bool,
}

impl Default for Indexing {
    fn default() -> Self {
        Self {
            db: "postgres://ceramic:password@localhost:5432/ceramic".to_string(),
            allow_queries_before_historical_sync: true,
            enable_historical_sync: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DidResolvers {
    Ethr(HashMap<String, serde_json::Value>),
}

impl Default for DidResolvers {
    fn default() -> Self {
        Self::Ethr(HashMap::default())
    }
}

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
            directory: PathBuf::from("./log/ceramic"),
        }
    }
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Metrics {
    Disabled,
    Enabled(String),
}

impl Default for Metrics {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub ipfs: Ipfs,
    pub state_store: StateStore,
    pub http_api: HttpApi,
    pub network: Network,
    pub anchor: Anchor,
    pub indexing: Indexing,
    pub did_resolvers: DidResolvers,
    pub node: Node,
    pub logger: Logger,
    pub metrics: Metrics,
}

impl Config {
    pub fn in_memory(&mut self) -> &mut Self {
        self.anchor = Anchor::None;
        self.network = Network::in_memory();
        self
    }

    pub fn local(&mut self, name: &str) -> &mut Self {
        self.anchor = Anchor::None;
        self.network = Network::local(name);
        self
    }

    pub fn dev(&mut self) -> &mut Self {
        self.network = Network::dev();
        self
    }

    pub fn test(&mut self) -> &mut Self {
        self.network = Network::clay();
        self
    }

    pub fn production(&mut self) -> &mut Self {
        self.network = Network::mainnet();
        self.indexing.enable_historical_sync = true;
        self.node.gateway = true;
        self
    }
}

impl Config {
    pub fn eth_resolver_options(&self) -> Option<String> {
        let DidResolvers::Ethr(m) = &self.did_resolvers;
        Some(serde_json::to_string(m).unwrap_or_else(|_| String::default()))
    }

    pub fn allows_sqlite(&self) -> bool {
        self.network.id != NetworkIdentifier::Mainnet
    }
}

pub fn from_file_err(file: String) -> anyhow::Result<Config> {
    let data = std::fs::read(PathBuf::from(file))?;
    Ok(serde_json::from_slice(data.as_slice())?)
}

pub fn from_string_err(json: &str) -> anyhow::Result<Config> {
    Ok(serde_json::from_str(json)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_roundtrip_default_config() {
        let js = serde_json::to_string(&Config::default()).unwrap();
        let _: Config = serde_json::from_str(&js).unwrap();
    }
}
