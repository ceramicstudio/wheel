use crate::{Anchor, LogLevel, Metrics, NetworkIdentifier};

fn convert_log_level(level: LogLevel) -> u16 {
    match level {
        LogLevel::Error => 0,
        LogLevel::Warn => 1,
        LogLevel::Info => 2,
        LogLevel::Debug => 3,
        LogLevel::Trace => 4,
    }
}

pub fn convert_network_identifier(id: &NetworkIdentifier) -> &'static str {
    match id {
        NetworkIdentifier::InMemory => "inmemory",
        NetworkIdentifier::Local => "local",
        NetworkIdentifier::Dev => "dev-unstable",
        NetworkIdentifier::Clay => "testnet-clay",
        NetworkIdentifier::Mainnet => "mainnet",
    }
}

impl Into<crate::daemon::DaemonConfig> for crate::Config {
    fn into(self) -> crate::daemon::DaemonConfig {
        let mut node = crate::daemon::DaemonNodeConfig {
            sync_override: None,
            gateway: Some(self.node.gateway),
            stream_cache_limit: None,
            private_seed_url: None,
        };
        let anchor = match self.anchor {
            Anchor::None => {
                //TODO: This is a hack to get around the fact that the anchor service must be present
                Some(crate::daemon::DaemonAnchorConfig {
                    anchor_service_url: None,
                    auth_method: None,
                    ethereum_rpc_url: None,
                })
            }
            Anchor::Ip { url } => {
                log::warn!("Anchor using {} with IP Authentication. Please see https://composedb.js.org/docs/0.4.x/guides/composedb-server/access-mainnet#updating-to-did-based-authentication to use IP authentication", url);
                Some(crate::daemon::DaemonAnchorConfig {
                    anchor_service_url: Some(url),
                    auth_method: None,
                    ethereum_rpc_url: None,
                })
            }
            Anchor::RemoteDid {
                url,
                private_seed_url,
            } => {
                log::info!("Anchor using {} with DID Authentication. Please see https://composedb.js.org/docs/0.4.x/guides/composedb-server/access-mainnet#updating-to-did-based-authentication for more information", url);
                node.private_seed_url = Some(private_seed_url);
                Some(crate::daemon::DaemonAnchorConfig {
                    anchor_service_url: Some(url),
                    auth_method: Some("did".to_string()),
                    ethereum_rpc_url: None,
                })
            }
        };
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
        let metrics = if let Metrics::Enabled(url) = self.metrics {
            crate::daemon::DaemonMetricsConfig {
                metrics_exporter_enabled: true,
                collector_host: Some(url),
            }
        } else {
            crate::daemon::DaemonMetricsConfig {
                metrics_exporter_enabled: false,
                collector_host: None,
            }
        };
        let network = Some(crate::daemon::DaemonNetworkConfig {
            name: Some(convert_network_identifier(&self.network.id).to_string()),
            pubsub_topic: self.network.pubsub_topic,
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
            metrics: Some(metrics),
            network: network,
            node: Some(node),
            state_store: state_store,
            indexing: indexing,
            did_resolvers: None,
        }
    }
}
