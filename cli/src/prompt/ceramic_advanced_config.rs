use ceramic_config::*;
use inquire::*;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};

use crate::did::DidAndPrivateKey;

enum ConfigSelect {
    Defaults,
    Advanced,
}

impl std::fmt::Display for ConfigSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defaults => write!(f, "Defaults: Use default configuration based on network"),
            Self::Advanced => write!(
                f,
                "Advanced: Configure all ceramic options based on network"
            ),
        }
    }
}

pub async fn prompt(
    working_directory: impl AsRef<Path>,
    cfg: &mut Config,
    admin_did: &DidAndPrivateKey,
) -> anyhow::Result<()> {
    let ans = Select::new(
        "Configure Ceramic",
        vec![ConfigSelect::Defaults, ConfigSelect::Advanced],
    )
    .prompt()?;

    match ans {
        ConfigSelect::Defaults => {
            log::info!("Using default configuration for {}", cfg.network.id);
        }
        ConfigSelect::Advanced => {
            configure(cfg, admin_did, working_directory).await?;
        }
    }

    Ok(())
}

pub fn configure_ipfs(cfg: &mut Config) -> anyhow::Result<()> {
    let ans = Select::new(
        "Bundled or Remote IPFS (default=Bundled)",
        vec![Ipfs::Bundled, Ipfs::Remote(IpfsRemote::default())],
    )
    .prompt()?;

    let r = if let Ipfs::Remote(_) = ans {
        let ipfs = IpfsRemote {
            host: Text::new("IPFS Hostname").prompt()?,
        };
        Ipfs::Remote(ipfs)
    } else {
        Ipfs::Bundled
    };
    cfg.ipfs = r;
    Ok(())
}

enum StateStoreSelect {
    S3,
    Local,
}

impl std::fmt::Display for StateStoreSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S3 => write!(f, "S3 (Bucket must already be setup in AWS)"),
            Self::Local => write!(f, "Local"),
        }
    }
}

pub async fn configure_state_store(
    cfg: &mut Config,
    working_directory: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let ans = Select::new(
        "State Store (default=local)",
        vec![StateStoreSelect::Local, StateStoreSelect::S3],
    )
    .prompt()?;

    let r = if let StateStoreSelect::Local = ans {
        let default = working_directory.as_ref().join("ceramic-state");
        let location = Text::new("Directory")
            .with_default(&default.display().to_string())
            .prompt()?;
        let location = PathBuf::from(location);
        if !tokio::fs::try_exists(&location).await? {
            tokio::fs::create_dir_all(location.clone()).await?;
        }
        StateStore::LocalDirectory(location)
    } else {
        let bucket = Text::new("Bucket").with_default("ceramic").prompt()?;
        let endpoint = Text::new("Endpoint").prompt()?;
        StateStore::S3(S3StateStore {
            bucket: bucket,
            endpoint: endpoint,
        })
    };
    cfg.state_store = r;
    Ok(())
}

pub fn configure_http_api(cfg: &mut Config, admin_did: &DidAndPrivateKey) -> anyhow::Result<()> {
    cfg.http_api.hostname = Text::new("Bind address")
        .with_default(&cfg.http_api.hostname)
        .prompt()?;
    cfg.http_api.port = Text::new("Bind port")
        .with_default(&cfg.http_api.port.to_string())
        .prompt()?
        .parse()?;
    let cors = Text::new("Cors origins, comma separated")
        .with_default(&cfg.http_api.cors_allowed_origins.join(","))
        .prompt()?;
    let cors = cors.split(",").map(|s| s.trim().to_string()).collect();
    cfg.http_api.cors_allowed_origins = cors;
    cfg.http_api.admin_dids = vec![admin_did.did().to_string()];
    Ok(())
}

fn configure_network(cfg: &mut Config) -> anyhow::Result<()> {
    match cfg.network.id {
        NetworkIdentifier::Local => {
            let topic = cfg.network.pubsub_topic.clone().unwrap_or_else(|| {
                format!(
                    "/ceramic/local-{}",
                    std::time::Instant::now().elapsed().as_millis()
                )
            });
            let topic = Text::new("Pubsub Topic").with_default(&topic).prompt()?;
            cfg.network.pubsub_topic = Some(topic);
        }
        _ => {
            //doesn't use pubsub topic
        }
    }
    Ok(())
}

pub fn configure_node(cfg: &mut Config) -> anyhow::Result<()> {
    let gateway = Confirm::new("Run as gateway?")
        .with_help_message("Gateway nodes cannot perform mutations")
        .with_default(false)
        .prompt()?;
    cfg.node.gateway = gateway;
    Ok(())
}

enum IndexingSelect {
    Sqlite,
    Postgres,
}

impl std::fmt::Display for IndexingSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite => write!(f, "Sqlite"),
            Self::Postgres => write!(f, "Postgres"),
        }
    }
}

enum SqliteSelect {
    CurrentDirectory,
    CustomDirectory,
}

impl std::fmt::Display for SqliteSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentDirectory => write!(f, "Current Directory"),
            Self::CustomDirectory => write!(f, "Custom Directory"),
        }
    }
}

pub fn configure_indexing(
    cfg: &mut Config,
    working_directory: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let default = if cfg.indexing.db.contains("sqlite") {
        0
    } else {
        1
    };
    let indexing = if cfg.network.id == NetworkIdentifier::Mainnet {
        IndexingSelect::Postgres
    } else {
        Select::new(
            "Indexing Database",
            vec![IndexingSelect::Sqlite, IndexingSelect::Postgres],
        )
        .with_starting_cursor(default)
        .prompt()?
    };
    match indexing {
        IndexingSelect::Sqlite => {
            if !cfg.allows_sqlite() {
                anyhow::bail!("sqlite not allowed in environment {}", cfg.network.id);
            }
            let ans = Select::new(
                "Sqlite Database Location",
                vec![
                    SqliteSelect::CurrentDirectory,
                    SqliteSelect::CustomDirectory,
                ],
            )
            .with_starting_cursor(default)
            .prompt()?;
            match ans {
                SqliteSelect::CurrentDirectory => {
                    cfg.indexing.db = format!(
                        "sqlite://{}/ceramic.db",
                        working_directory.as_ref().display()
                    );
                }
                SqliteSelect::CustomDirectory => {
                    let default = if cfg.indexing.is_sqlite() {
                        cfg.indexing.db.clone()
                    } else {
                        format!("{}/ceramic.db", working_directory.as_ref().display())
                    };
                    let location = Text::new("Sqlite Database Location")
                        .with_help_message("Example: sqlite:///directory-for-ceramic/ceramic.db")
                        .with_default(&default)
                        .prompt()?;
                    cfg.indexing.db = format!("sqlite://{}", location);
                }
            }
        }
        IndexingSelect::Postgres => {
            let default = if !cfg.indexing.is_sqlite() {
                cfg.indexing.db.clone()
            } else {
                Indexing::postgres_default().to_string()
            };
            cfg.indexing.db = Text::new("Postgres Database Connection String")
                .with_help_message(&format!("Example: {}", Indexing::postgres_default()))
                .with_default(&default)
                .prompt()?;
        }
    }

    Ok(())
}

pub async fn configure(
    cfg: &mut Config,
    admin_did: &DidAndPrivateKey,
    working_directory: impl AsRef<Path>,
) -> anyhow::Result<()> {
    configure_ipfs(cfg)?;
    configure_state_store(cfg, working_directory.as_ref()).await?;
    configure_http_api(cfg, admin_did)?;
    configure_network(cfg)?;
    configure_node(cfg)?;
    configure_indexing(cfg, working_directory)?;

    Ok(())
}
