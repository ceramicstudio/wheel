use ceramic_config::*;
use inquire::*;
use ssi::did::Document;
use std::path::PathBuf;

pub async fn prompt<'a, 'b, Fn, Fut>(
    cfg: &'a mut Config,
    admin_did: &'b Document,
    mut func: Fn,
) -> anyhow::Result<()>
where
    Fut: std::future::Future<Output = anyhow::Result<()>>,
    Fn: FnMut(&'a mut Config, &'b Document) -> Fut,
{
    let ans = Confirm::new(&format!("Start ceramic configuration?"))
        .with_help_message("Step through interactive prompts to configure ceramic node")
        .prompt_skippable()?;

    if let Some(true) = ans {
        func(cfg, admin_did).await?;
    }

    Ok(())
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

pub fn configure_ipfs(cfg: &mut Config) -> anyhow::Result<()> {
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

pub fn configure_http_api(cfg: &mut Config, admin_did: &Document) -> anyhow::Result<()> {
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
    cfg.http_api.admin_dids = vec![admin_did.id.clone()];
    Ok(())
}

fn configure_network(cfg: &mut Config) -> anyhow::Result<()> {
    cfg.network.name = Text::new("Network name")
        .with_default(&cfg.network.name)
        .prompt()?;
    cfg.network.pubsub_topic = Text::new("Pubsub Topic")
        .with_default(&cfg.network.pubsub_topic)
        .prompt()?;
    Ok(())
}

fn configure_anchor(cfg: &mut Config) -> anyhow::Result<()> {
    cfg.anchor.anchor_service_url = Text::new("Anchor Service Url")
        .with_default(&cfg.anchor.anchor_service_url)
        .prompt()?;
    cfg.anchor.ethereum_rpc_url = Text::new("Ethereum RPC Url")
        .with_default(&cfg.anchor.ethereum_rpc_url)
        .prompt()?;
    Ok(())
}

pub fn configure_indexing(cfg: &mut Config) -> anyhow::Result<()> {
    cfg.indexing.db = Text::new("Database Url")
        .with_default(&cfg.indexing.db)
        .prompt()?;
    Ok(())
}

pub async fn configure<'a, 'b>(cfg: &'a mut Config, admin_did: &'b Document) -> anyhow::Result<()> {
    configure_ipfs(cfg)?;
    configure_state_store(cfg).await?;
    configure_http_api(cfg, admin_did)?;
    configure_network(cfg)?;
    configure_anchor(cfg)?;
    configure_indexing(cfg)?;

    Ok(())
}
