use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DaemonIpfsConfigMode {
    Bundled,
    Remote,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonIpfsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<DaemonIpfsConfigMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinning_endpoints: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DaemonStateStoreMode {
    S3,
    File,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonStateStoreConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<DaemonStateStoreMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_directory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_bucket: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_endpoint: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonHttpApiConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cors_allowed_origins: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_dids: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonNetworkConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubsub_topic: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonAnchorConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_service_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ethereum_rpc_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonIndexingConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub db: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_queries_before_historical_sync: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonResolversConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ethr_did_resolver: Option<HashMap<String, Value>>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonNodeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_override: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_cache_limit: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonLoggerConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_directory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_level: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_to_files: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonMetricsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics_exporter_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collector_host: Option<String>,
}

/// Config format compatible with existing ceramic daemon format
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DaemonConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<DaemonAnchorConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_api: Option<DaemonHttpApiConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipfs: Option<DaemonIpfsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logger: Option<DaemonLoggerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<DaemonMetricsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<DaemonNetworkConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<DaemonNodeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_store: Option<DaemonStateStoreConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub did_resolvers: Option<DaemonResolversConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing: Option<DaemonIndexingConfig>,
}
