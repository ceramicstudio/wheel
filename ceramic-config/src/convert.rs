use crate::{LogLevel, NetworkIdentifier};

fn convert_log_level(level: LogLevel) -> u16 {
    match level {
        LogLevel::Error => 0,
        LogLevel::Warn => 1,
        LogLevel::Info => 2,
        LogLevel::Debug => 3,
        LogLevel::Trace => 4,
    }
}

fn convert_network_identifier(id: NetworkIdentifier) -> String {
    match id {
        NetworkIdentifier::InMemory => "inmemory".to_string(),
        NetworkIdentifier::Local => "local".to_string(),
        NetworkIdentifier::Dev => "dev-unstable".to_string(),
        NetworkIdentifier::Clay => "testnet-clay".to_string(),
        NetworkIdentifier::Mainnet => "mainnet".to_string(),
    }
}

impl Into<crate::daemon::DaemonConfig> for crate::Config {
    fn into(self) -> crate::daemon::DaemonConfig {
        let anchor = Some(crate::daemon::DaemonAnchorConfig {
            anchor_service_url: Some(self.anchor.anchor_service_url),
            ethereum_rpc_url: None,
        });
        let http = Some(crate::daemon::DaemonHttpApiConfig {
            hostname: Some(self.http_api.hostname),
            port: Some(self.http_api.port),
            admin_dids: Some(self.http_api.admin_dids),
            cors_allowed_origins: None,
        });
        let ipfs = Some(if let crate::Ipfs::Remote(r) = self.ipfs {
            crate::daemon::DaemonIpfsConfig {
                mode: Some(crate::daemon::DaemonIpfsConfigMode::Remote),
                host: Some(r.host),
                pinning_endpoints: None,
            }
        } else {
            crate::daemon::DaemonIpfsConfig {
                mode: Some(crate::daemon::DaemonIpfsConfigMode::Bundled),
                host: None,
                pinning_endpoints: None,
            }
        });
        let logger = Some(if let Some(l) = self.logger.file {
            crate::daemon::DaemonLoggerConfig {
                log_to_files: Some(l.enabled),
                log_directory: Some(l.directory.to_string_lossy().to_string()),
                log_level: Some(convert_log_level(self.logger.level)),
            }
        } else {
            crate::daemon::DaemonLoggerConfig {
                log_to_files: Some(false),
                log_directory: None,
                log_level: Some(convert_log_level(self.logger.level)),
            }
        });
        let metrics = Some(crate::daemon::DaemonMetricsConfig {
            metrics_exporter_enabled: Some(self.metrics.enabled),
            collector_host: Some(self.metrics.host),
        });
        let network = Some(crate::daemon::DaemonNetworkConfig {
            name: Some(convert_network_identifier(self.network.id)),
            pubsub_topic: self.network.pubsub_topic,
        });
        let node = Some(crate::daemon::DaemonNodeConfig {
            sync_override: None,
            gateway: Some(self.node.gateway),
            stream_cache_limit: None,
        });
        let state_store = Some(match self.state_store {
            crate::StateStore::S3(s3) => crate::daemon::DaemonStateStoreConfig {
                s3_bucket: Some(s3.bucket),
                s3_endpoint: Some(s3.endpoint),
                mode: Some(crate::daemon::DaemonStateStoreMode::S3),
                local_directory: None,
            },
            crate::StateStore::LocalDirectory(l) => crate::daemon::DaemonStateStoreConfig {
                local_directory: Some(l.to_string_lossy().to_string()),
                mode: Some(crate::daemon::DaemonStateStoreMode::File),
                s3_bucket: None,
                s3_endpoint: None,
            },
        });
        let indexing = Some(crate::daemon::DaemonIndexingConfig {
            db: Some(self.indexing.db),
            allow_queries_before_historical_sync: Some(
                self.indexing.allow_queries_before_historical_sync,
            ),
            enable_historical_sync: Some(self.indexing.enable_historical_sync),
        });
        crate::daemon::DaemonConfig {
            anchor: anchor,
            http_api: http,
            ipfs: ipfs,
            logger: logger,
            metrics: metrics,
            network: network,
            node: node,
            state_store: state_store,
            indexing: indexing,
            did_resolvers: None,
        }
    }
}
