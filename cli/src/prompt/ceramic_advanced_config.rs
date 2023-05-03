use ceramic_config::*;
use inquire::*;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};

use crate::did::DidAndPrivateKey;

enum ConfigSelect {
    Skip,
    Basic,
    Advanced,
}

impl std::fmt::Display for ConfigSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Skip => write!(f, "Skip: Use default configuration based on network"),
            Self::Basic => write!(f, "Basic: Configure limited settings based on network"),
            Self::Advanced => write!(f, "Advanced: Configure all ceramic options"),
        }
    }
}

pub async fn prompt<'a, 'b, Fn, Fut, P>(
    working_directory: P,
    cfg: &'a mut Config,
    admin_did: &'b DidAndPrivateKey,
    mut func: Fn,
) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
    Fn: FnMut(P, &'a mut Config, &'b DidAndPrivateKey) -> Fut,
{
    let ans = Select::new(
        "Configure Ceramic",
        vec![
            ConfigSelect::Skip,
            ConfigSelect::Basic,
            ConfigSelect::Advanced,
        ],
    )
    .prompt()?;

    match ans {
        ConfigSelect::Skip => {
            log::info!("Using default configuration for {}", cfg.network.id);
        }
        ConfigSelect::Basic => {
            func(working_directory, cfg, admin_did).await?;
        }
        ConfigSelect::Advanced => {
            configure(working_directory, cfg, admin_did).await?;
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
            Self::S3 => write!(f, "S3"),
            Self::Local => write!(f, "Local"),
        }
    }
}

pub async fn configure_state_store(cfg: &mut Config) -> anyhow::Result<()> {
    let ans = Select::new(
        "State Store (default=local)",
        vec![StateStoreSelect::Local, StateStoreSelect::S3],
    )
    .prompt()?;

    let r = if let StateStoreSelect::Local = ans {
        let location = Text::new("Directory")
            .with_default("/opt/ceramic/data")
            .prompt()?;
        let location = PathBuf::from(location);
        if !location.exists() {
            if Confirm::new("Directory does not exist, create it?").prompt()? {
                tokio::fs::create_dir_all(location.clone()).await?;
            } else {
                log::warn!("Not creating directory, ceramic will fail to start");
            }
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
    cfg.network.id = Select::new(
        "Network type",
        vec![
            NetworkIdentifier::InMemory,
            NetworkIdentifier::Local,
            NetworkIdentifier::Dev,
            NetworkIdentifier::Clay,
            NetworkIdentifier::Mainnet,
        ],
    )
    .with_starting_cursor(3)
    .prompt()?;
    match cfg.network.id {
        NetworkIdentifier::Local | NetworkIdentifier::Dev => {
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
        .with_default(true)
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
            Self::Sqlite => write!(f, "Sqlite (Cannot be used on Mainnet)"),
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

pub fn configure_indexing<P: AsRef<Path>>(
    working_directory: P,
    cfg: &mut Config,
) -> anyhow::Result<()> {
    let default = if cfg.indexing.db.contains("sqlite") {
        0
    } else {
        1
    };
    match Select::new(
        "Indexing Database",
        vec![IndexingSelect::Sqlite, IndexingSelect::Postgres],
    )
    .with_starting_cursor(default)
    .prompt()?
    {
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
                    let location = Text::new("Sqlite Database Location")
                        .with_help_message("Example: sqlite:///directory-for-ceramic/ceramic.db")
                        .with_default(&cfg.indexing.db)
                        .prompt()?;
                    cfg.indexing.db = format!("sqlite://{}", location);
                }
            }
        }
        IndexingSelect::Postgres => {
            cfg.indexing.db = Text::new("Postgres Database Connection String")
                .with_help_message("Example: postgres://user:password@localhost:5432/ceramic")
                .with_default(&cfg.indexing.db)
                .prompt()?;
        }
    }

    Ok(())
}

pub async fn configure<'a, 'b, P: AsRef<Path>>(
    working_directory: P,
    cfg: &'a mut Config,
    admin_did: &'b DidAndPrivateKey,
) -> anyhow::Result<()> {
    configure_ipfs(cfg)?;
    configure_state_store(cfg).await?;
    configure_http_api(cfg, admin_did)?;
    configure_network(cfg)?;
    configure_node(cfg)?;
    configure_indexing(working_directory, cfg)?;

    Ok(())
}
