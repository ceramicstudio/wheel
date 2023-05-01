mod did;
mod install;
mod prompt;

pub use crate::did::DidAndPrivateKey;
pub use ceramic_config::NetworkIdentifier;
use ceramic_config::{Anchor, CasAuth, Config};
use inquire::*;
use prompt::project::Project;
use std::path::{Path, PathBuf};
use tokio::{io::AsyncWriteExt, task::JoinHandle};

#[derive(Default)]
pub struct Versions {
    pub ceramic: Option<semver::Version>,
    pub composedb: Option<semver::Version>,
}

pub async fn interactive(
    working_directory: PathBuf,
    versions: Versions,
) -> anyhow::Result<Option<JoinHandle<()>>> {
    let network_identifier = Select::new(
        "Project Type",
        vec![
            NetworkIdentifier::InMemory,
            NetworkIdentifier::Local,
            NetworkIdentifier::Dev,
            NetworkIdentifier::Clay,
            NetworkIdentifier::Mainnet,
        ],
    )
    .with_help_message("InMemory nodes will not anchor")
    .prompt()?;

    log::info!("Starting configuration for {} project", network_identifier);

    let advanced = Confirm::new("Advanced configuration?")
        .with_default(false)
        .prompt()?;

    let project = prompt::project::configure_project(&working_directory).await?;

    let with_ceramic = Confirm::new("Include Ceramic?")
        .with_default(true)
        .prompt()?;

    let with_composedb = if with_ceramic {
        Confirm::new("Include ComposeDB?")
            .with_default(true)
            .prompt()?
    } else {
        false
    };

    let doc = prompt::did::prompt(&project.path).await?;
    let cas_auth = if NetworkIdentifier::InMemory == network_identifier {
        None
    } else {
        prompt::cas_auth::prompt(&doc, &network_identifier).await?
    };

    let cfg_file_path = project.path.join("ceramic.json");
    let cfg_file_path = Text::new("Wheel config file location")
        .with_default(cfg_file_path.to_string_lossy().as_ref())
        .prompt()?;
    let cfg_file_path = PathBuf::from(cfg_file_path);
    let mut cfg =
        get_or_create_config(&project, &network_identifier, cas_auth, &cfg_file_path).await?;

    cfg.http_api.admin_dids.push(doc.did().to_string());

    if advanced {
        prompt::prompt(&project.path, &mut cfg, &doc, prompt::advanced_config).await?;
    } else {
        match network_identifier {
            NetworkIdentifier::InMemory => {
                prompt::prompt(&project.path, &mut cfg, &doc, prompt::local_config).await?;
            }
            NetworkIdentifier::Local => {
                prompt::prompt(&project.path, &mut cfg, &doc, prompt::local_config).await?;
            }
            NetworkIdentifier::Dev => {
                prompt::prompt(&project.path, &mut cfg, &doc, prompt::remote_config).await?;
            }
            NetworkIdentifier::Clay => {
                prompt::prompt(&project.path, &mut cfg, &doc, prompt::remote_config).await?;
            }
            NetworkIdentifier::Mainnet => {
                prompt::prompt(&project.path, &mut cfg, &doc, prompt::remote_config).await?;
            }
        }
    }

    finish_setup(
        project,
        cfg,
        cfg_file_path,
        doc,
        versions,
        with_ceramic,
        with_composedb,
        false,
    )
    .await
}

pub struct QuietOptions {
    pub working_directory: PathBuf,
    pub network_identifier: NetworkIdentifier,
    pub versions: Versions,
    pub did: DidAndPrivateKey,
    pub with_ceramic: bool,
    pub with_composedb: bool,
}

pub async fn quiet(opts: QuietOptions) -> anyhow::Result<Option<JoinHandle<()>>> {
    let project = Project {
        name: "ceramic-app".to_string(),
        path: opts.working_directory,
    };

    if !project.path.exists() {
        log::info!(
            "Project directory {} does not exist, creating it",
            project.path.display()
        );
        tokio::fs::create_dir_all(&project.path).await?;
    }

    let cas_auth = Anchor::url_for_network(&opts.network_identifier).map(|url| {
        let pk = opts.did.cas_auth();
        CasAuth { url, pk: Some(pk) }
    });
    let cfg_file_path = project.path.join("ceramic.json");
    let mut cfg =
        get_or_create_config(&project, &opts.network_identifier, cas_auth, &cfg_file_path).await?;

    if let NetworkIdentifier::InMemory | NetworkIdentifier::Local = opts.network_identifier {
        let abs_path = project.path.canonicalize()?.join("ceramic-indexing");
        if !abs_path.exists() {
            tokio::fs::create_dir_all(&abs_path).await?;
        }
        let sql_path = abs_path
            .join("ceramic.sqlite")
            .to_string_lossy()
            .to_string();
        cfg.indexing.db = format!("sqlite://{}", sql_path);
    }
    cfg.http_api.admin_dids.push(opts.did.did().to_string());

    finish_setup(
        project,
        cfg,
        cfg_file_path,
        opts.did,
        opts.versions,
        opts.with_ceramic,
        opts.with_composedb,
        true,
    )
    .await
}

async fn finish_setup(
    project: Project,
    cfg: Config,
    cfg_file_path: PathBuf,
    doc: DidAndPrivateKey,
    versions: Versions,
    with_ceramic: bool,
    with_composedb: bool,
    quiet: bool,
) -> anyhow::Result<Option<JoinHandle<()>>> {
    log::info!("Saving config to {}", cfg_file_path.display());
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&cfg)?.as_bytes()).await?;
    f.flush().await?;

    let daemon_config_file = write_daemon_config(&project.path, &cfg).await?;

    let opt_child = if with_ceramic {
        let opt_child = install::ceramic_daemon::install_ceramic_daemon(
            &project.path,
            &cfg,
            &versions.ceramic,
            &daemon_config_file,
            quiet,
        )
        .await?;
        if with_composedb {
            install::compose_db::install_compose_db(&cfg, &doc, &project.path, &versions.composedb)
                .await?;
        }
        opt_child
    } else {
        None
    };

    Ok(opt_child)
}

async fn write_daemon_config(
    working_directory: impl AsRef<Path>,
    cfg: &ceramic_config::Config,
) -> anyhow::Result<PathBuf> {
    let cfg_file_path = working_directory.as_ref().join("daemon_config.json");
    log::info!("Saving daemon file to {}", cfg_file_path.display());
    let daemon_config: ceramic_config::DaemonConfig = cfg.clone().into();
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .append(false)
        .open(&cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&daemon_config)?.as_bytes())
        .await?;
    f.flush().await?;
    Ok(cfg_file_path)
}

async fn get_or_create_config(
    project: &Project,
    network_identifier: &NetworkIdentifier,
    cas_auth: Option<CasAuth>,
    cfg_file_path: impl AsRef<Path>,
) -> anyhow::Result<Config> {
    let cfg = if cfg_file_path.as_ref().exists() {
        log::info!(
            "Initializing config with previous information from {}",
            cfg_file_path.as_ref().display()
        );
        let data = tokio::fs::read(cfg_file_path.as_ref()).await?;
        let mut cfg: Config = serde_json::from_slice(data.as_slice())?;
        cfg.initialize(network_identifier, &project.name, cas_auth);
        cfg
    } else {
        let mut cfg = Config::new(network_identifier, &project.name, cas_auth);
        if network_identifier == &NetworkIdentifier::InMemory {
            cfg.indexing.db = format!("sqlite://{}/ceramic.db", project.path.display());
        }
        cfg
    };
    Ok(cfg)
}
