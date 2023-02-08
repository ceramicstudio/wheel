use ceramic_config::*;
use inquire::*;
use ssi::did::Document;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub async fn prompt(working_directory: &Path, admin_did: &Document) -> anyhow::Result<Config> {
    let cfg_file_path = working_directory.join("ceramic.json");
    let cfg_file_path = Text::new("Ceramic ceramic-config file location")
        .with_default(cfg_file_path.to_string_lossy().as_ref())
        .prompt()?;
    let cfg_file_path = PathBuf::from(cfg_file_path);
    let (mut cfg, existing) = if cfg_file_path.exists() {
        let data = tokio::fs::read(cfg_file_path.clone()).await?;
        let cfg = serde_json::from_slice(data.as_slice())?;
        (cfg, " Existing configuration will be overwritten")
    } else {
        (Config::default(), "")
    };

    let ans = Confirm::new(&format!("Start ceramic configuration?{}", existing))
        .with_help_message("Step through interactive prompts to configure ceramic node")
        .prompt_skippable()?;

    if let Some(true) = ans {
        configure_ceramic(&mut cfg, admin_did).await?;
    }

    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&cfg)?.as_bytes()).await?;
    f.flush().await?;

    Ok(cfg)
}

enum IpfsSelect {
    Bundled,
    Remote,
}

impl std::fmt::Display for IpfsSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundled => write!(f, "Bundled"),
            Self::Remote => write!(f, "Remote"),
        }
    }
}

fn configure_ipfs() -> anyhow::Result<Ipfs> {
    let ans = Select::new(
        "Bundled or Remote IPFS (default=Bundled)",
        vec![IpfsSelect::Bundled, IpfsSelect::Remote],
    )
    .prompt()?;

    let r = if let IpfsSelect::Remote = ans {
        let ipfs = IpfsRemote {
            host: Text::new("IPFS Hostname").prompt()?,
        };
        Ipfs::Remote(ipfs)
    } else {
        Ipfs::Bundled
    };
    Ok(r)
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

async fn configure_state_store() -> anyhow::Result<StateStore> {
    let ans = Select::new(
        "State Store (default=local)",
        vec![StateStoreSelect::Local, StateStoreSelect::S3],
    )
    .prompt()?;

    let r = if let StateStoreSelect::Local = ans {
        let location = Text::new("Directory")
            .with_default("/etc/ceramic/data")
            .prompt()?;
        let location = PathBuf::from(location);
        if !location.exists() {
            if Confirm::new("Directory does not exist, create it?").prompt()? {
                tokio::fs::create_dir_all(location.clone()).await?;
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
    Ok(r)
}

fn configure_http_api(admin_did: &Document) -> anyhow::Result<HttpApi> {
    let mut http = HttpApi::default();
    http.hostname = Text::new("Bind address")
        .with_default(&http.hostname)
        .prompt()?;
    http.port = Text::new("Bind port")
        .with_default(&http.port.to_string())
        .prompt()?
        .parse()?;
    let cors = Text::new("Cors origins, comma separated").prompt()?;
    let cors = cors.split(",").map(|s| s.trim().to_string()).collect();
    http.cors_allowed_origins = cors;
    http.admin_dids = vec![admin_did.id.clone()];
    Ok(http)
}

fn configure_network() -> anyhow::Result<Network> {
    let mut network = Network::default();
    network.name = Text::new("Network name")
        .with_default(&network.name)
        .prompt()?;
    network.pubsub_topic = Text::new("Pubsub Topic")
        .with_default(&network.pubsub_topic)
        .prompt()?;
    Ok(network)
}

fn configure_anchor() -> anyhow::Result<Anchor> {
    let mut anchor = Anchor::default();
    anchor.anchor_service_url = Text::new("Anchor Service Url")
        .with_default(&anchor.anchor_service_url)
        .prompt()?;
    anchor.ethereum_rpc_url = Text::new("Ethereum RPC Url")
        .with_default(&anchor.ethereum_rpc_url)
        .prompt()?;
    Ok(anchor)
}

fn configure_indexing() -> anyhow::Result<Indexing> {
    let mut index = Indexing::default();
    index.db = Text::new("Database Url").with_default(&index.db).prompt()?;
    Ok(index)
}

async fn configure_ceramic<'a, 'b>(
    cfg: &'a mut Config,
    admin_did: &'b Document,
) -> anyhow::Result<&'a mut Config> {
    cfg.ipfs = configure_ipfs()?;
    cfg.state_store = configure_state_store().await?;
    cfg.http_api = configure_http_api(admin_did)?;
    cfg.network = configure_network()?;
    cfg.anchor = configure_anchor()?;
    cfg.indexing = configure_indexing()?;

    Ok(cfg)
}
