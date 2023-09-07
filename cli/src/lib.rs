mod did;
mod install;
mod prompt;

pub use crate::did::DidAndPrivateKey;
pub use ceramic_config::NetworkIdentifier;
use ceramic_config::{Anchor, CasAuth, Config};
use inquire::*;
use prompt::project::Project;
use ssi::did::Document;
use std::path::{Path, PathBuf};
use tokio::{io::AsyncWriteExt, task::JoinHandle};

#[derive(Default)]
pub struct Versions {
    pub ceramic: Option<semver::Version>,
    pub composedb: Option<semver::Version>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DefaultChoice {
    Keep,
    Change,
    Exit,
}

impl std::fmt::Display for DefaultChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keep => write!(f, "Keep"),
            Self::Change => write!(f, "Change"),
            Self::Exit => write!(f, "Exit"),
        }
    }
}

pub async fn interactive_default(
    working_directory: PathBuf,
    versions: Versions,
) -> anyhow::Result<Option<JoinHandle<()>>> {
    let network_identifier = NetworkIdentifier::InMemory;
    let project = Project {
        name: "ceramic-app".to_string(),
        path: working_directory.join("ceramic-app"),
    };
    let did_sk_path = project.path.join("admin.sk");

    log::info!(r#"Welcome to Wheel! By default, wheel will have the following configuration:
    - Project Name: {}
    - Project Directory: {}
    - Network: {}
    - Ceramic Included
    - ComposeDB Included
    - ComposeDB Sample Application Included
    - DID Generated Secret Key Path: {}
"#, project.name, project.path.display(), network_identifier, did_sk_path.display());

    let default_choice = Select::new(
        "Would you like to keep or change this configuration?",
        vec![
            DefaultChoice::Keep,
            DefaultChoice::Change,
            DefaultChoice::Exit,
        ],
    )
        .with_help_message(r#"If you choose to change configuration, you will be walked through a set of interactive prompts."#)
        .prompt()?;

    match default_choice {
        DefaultChoice::Exit => {
            log::info!("Exiting wheel");
            std::process::exit(0);
        }
        DefaultChoice::Change => {
            interactive(working_directory, versions).await
        }
        DefaultChoice::Keep => {
            if !tokio::fs::try_exists(&project.path).await? {
                log::info!(
                    "Project directory {} does not exist, creating it",
                    project.path.display()
                );
                tokio::fs::create_dir_all(&project.path).await?;
            }
            let doc = DidAndPrivateKey::generate(Some(did_sk_path)).await?;
            let cfg_file_path = project.path.join("ceramic.json");
            let mut cfg = get_or_create_config(
                &project,
                &network_identifier,
                None,
                &project.path,
                &cfg_file_path,
            )
            .await?;

            cfg.http_api.admin_dids.push(doc.did().to_string());

            finish_setup(
                project,
                cfg,
                cfg_file_path,
                doc,
                versions,
                true,
                true,
                true,
                false,
            ).await
        }
    }

}

pub async fn interactive(
    working_directory: PathBuf,
    versions: Versions,
) -> anyhow::Result<Option<JoinHandle<()>>> {
    let network_identifier = Select::new(
        "Project Type",
        vec![
            NetworkIdentifier::InMemory,
            NetworkIdentifier::Dev,
            NetworkIdentifier::Clay,
            NetworkIdentifier::Mainnet,
        ],
    )
    .with_help_message(r#"InMemory is recommended when trying out Ceramic and ComposeDB (but nodes will not anchor).
Other network types will require to setup up authentication with CAS (Ceramic Anchoring Service).
Selection is used to setup project defaults"#)
    .prompt()?;

    log::info!("Starting configuration for {} project", network_identifier);

    let project = prompt::project::configure_project(&working_directory).await?;

    let with_ceramic = Confirm::new("Include Ceramic?")
        .with_help_message("Installs Ceramic and allows Ceramic to be run as a daemon")
        .with_default(true)
        .prompt()?;

    let with_composedb = if with_ceramic {
        Confirm::new("Include ComposeDB?")
            .with_help_message("Installs ComposeDB and allows ComposeDB cli to be run")
            .with_default(true)
            .prompt()?
    } else {
        false
    };

    let with_app_template = if with_composedb {
        Confirm::new("Include ComposeDB Sample Application?")
            .with_help_message("Installs a sample application using ComposeDB")
            .with_default(false)
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
        .with_default(&cfg_file_path.display().to_string())
        .prompt()?;
    let cfg_file_path = PathBuf::from(cfg_file_path);
    let mut cfg = get_or_create_config(
        &project,
        &network_identifier,
        cas_auth,
        &project.path,
        &cfg_file_path,
    )
    .await?;

    cfg.http_api.admin_dids.push(doc.did().to_string());

    if with_app_template {
        if NetworkIdentifier::InMemory == network_identifier {
            cfg.http_api.cors_allowed_origins = vec![".*".to_string()];
        }
    }

    match network_identifier {
        NetworkIdentifier::InMemory => {
            prompt::prompt(&project.path, &mut cfg, &doc).await?;
        }
        NetworkIdentifier::Local => {
            // TODO: prompt for local config
        }
        NetworkIdentifier::Dev => {
            prompt::prompt(&project.path, &mut cfg, &doc).await?;
        }
        NetworkIdentifier::Clay => {
            prompt::prompt(&project.path, &mut cfg, &doc).await?;
        }
        NetworkIdentifier::Mainnet => {
            prompt::prompt(&project.path, &mut cfg, &doc).await?;
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
        with_app_template,
        false,
    )
    .await
}

pub struct DidOptions {
    pub did: String,
    pub private_key: String,
}

pub struct QuietOptions {
    pub project_name: Option<String>,
    pub working_directory: PathBuf,
    pub network_identifier: NetworkIdentifier,
    pub versions: Versions,
    pub did: Option<DidOptions>,
    pub with_ceramic: bool,
    pub with_composedb: bool,
    pub with_app_template: bool,
}

pub async fn quiet(opts: QuietOptions) -> anyhow::Result<Option<JoinHandle<()>>> {
    let project_name = opts
        .project_name
        .unwrap_or_else(|| "ceramic-app".to_string());
    let project_path = opts.working_directory.join(&project_name);
    let project = Project {
        name: project_name,
        path: project_path,
    };

    if !project.path.exists() {
        log::info!(
            "Project directory {} does not exist, creating it",
            project.path.display()
        );
        tokio::fs::create_dir_all(&project.path).await?;
    }

    let did = if let Some(opts) = opts.did {
        DidAndPrivateKey::new(opts.private_key, Document::new(&opts.did))
    } else {
        DidAndPrivateKey::generate(Some(project.path.join("admin.sk"))).await?
    };
    let cas_auth = Anchor::url_for_network(&opts.network_identifier).map(|url| {
        let pk = did.cas_auth();
        CasAuth { url, pk: Some(pk) }
    });
    let cfg_file_path = project.path.join("ceramic.json");
    let mut cfg = get_or_create_config(
        &project,
        &opts.network_identifier,
        cas_auth,
        &project.path,
        &cfg_file_path,
    )
    .await?;

    cfg.http_api.admin_dids.push(did.did().to_string());

    finish_setup(
        project,
        cfg,
        cfg_file_path,
        did,
        opts.versions,
        opts.with_ceramic,
        opts.with_composedb,
        opts.with_app_template,
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
    with_app_template: bool,
    quiet: bool,
) -> anyhow::Result<Option<JoinHandle<()>>> {
    log::info!("Saving config to {}", cfg_file_path.display());
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .truncate(true)
        .open(&cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string_pretty(&cfg)?.as_bytes())
        .await?;
    f.flush().await?;

    let daemon_config_file = write_daemon_config(&project.path, &cfg).await?;

    let opt_child = if with_ceramic {
        install::ceramic_daemon::install_ceramic_daemon(
            &project.path,
            &cfg,
            &versions.ceramic,
            &daemon_config_file,
            quiet,
        )
        .await?
    } else {
        None
    };

    if with_composedb {
        install::compose_db::install_compose_db(&cfg, &doc, &project.path, &versions.composedb)
            .await?;
    }

    if with_app_template {
        if opt_child.is_none() {
            anyhow::bail!("Cannot install app template without ceramic daemon");
        }
        install::ceramic_app_template::install_ceramic_app_template(
            &project.path,
            &project.name,
            &daemon_config_file,
        )
        .await?;
    }

    log::info!(
        "Project {} created at {} for network {}",
        project.name,
        project.path.display(),
        cfg.network.id
    );

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
    f.write_all(serde_json::to_string_pretty(&daemon_config)?.as_bytes())
        .await?;
    f.flush().await?;
    Ok(cfg_file_path)
}

async fn get_or_create_config(
    project: &Project,
    network_identifier: &NetworkIdentifier,
    cas_auth: Option<CasAuth>,
    working_directory: impl AsRef<Path>,
    cfg_file_path: impl AsRef<Path>,
) -> anyhow::Result<Config> {
    let cfg = if tokio::fs::try_exists(cfg_file_path.as_ref()).await? {
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
        if let NetworkIdentifier::InMemory
        | NetworkIdentifier::Local
        | NetworkIdentifier::Dev
        | NetworkIdentifier::Clay = cfg.network.id
        {
            let db_path = working_directory
                .as_ref()
                .canonicalize()?
                .join("ceramic-indexing");
            if !db_path.exists() {
                tokio::fs::create_dir_all(&db_path).await?;
            }
            let db_path = db_path.join("ceramic.db");
            cfg.indexing.db = format!("sqlite://{}", db_path.display());
        }
        cfg
    };
    Ok(cfg)
}
