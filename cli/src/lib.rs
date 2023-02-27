mod did;
mod install;
mod prompt;

use inquire::*;
use prompt::project::Project;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

#[derive(Default)]
pub struct Versions {
    pub ceramic: Option<semver::Version>,
    pub composedb: Option<semver::Version>,
    pub app_template: Option<semver::Version>,
}

pub enum ProjectType {
    Local,
    Dev,
    Test,
    Production,
    Advanced,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Dev => write!(f, "Dev"),
            Self::Test => write!(f, "Test (Clay)"),
            Self::Production => write!(f, "Production (Mainnet)"),
            Self::Advanced => write!(f, "Advanced"),
        }
    }
}

pub async fn interactive(working_directory: PathBuf, versions: Versions) -> anyhow::Result<()> {
    let ans = Select::new(
        "Project Type",
        vec![
            ProjectType::Local,
            ProjectType::Dev,
            ProjectType::Test,
            ProjectType::Production,
            ProjectType::Advanced,
        ],
    )
    .with_help_message("Local nodes will not anchor")
    .prompt()?;

    for_project_type(working_directory, ans, versions).await
}

pub async fn for_project_type(
    working_directory: PathBuf,
    project_type: ProjectType,
    versions: Versions,
) -> anyhow::Result<()> {
    let project = prompt::project::configure_project(working_directory).await?;

    let with_composedb = Confirm::new("Include ComposeDB?")
        .with_default(false)
        .prompt()?;

    let doc = prompt::did::generate_did(&project.path).await?;

    let cfg_file_path = project.path.join("ceramic.json");
    let cfg_file_path = Text::new("Ceramic ceramic-config file location")
        .with_default(cfg_file_path.to_string_lossy().as_ref())
        .prompt()?;
    let cfg_file_path = PathBuf::from(cfg_file_path);
    let mut cfg = if cfg_file_path.exists() {
        let data = tokio::fs::read(cfg_file_path.clone()).await?;
        let cfg = serde_json::from_slice(data.as_slice())?;
        cfg
    } else {
        ceramic_config::Config::default()
    };

    cfg.http_api.admin_dids.push(doc.id.clone());

    match project_type {
        ProjectType::Local => {
            cfg.anchor = ceramic_config::Anchor::in_memory();
            cfg.network = ceramic_config::Network::in_memory();
            prompt::prompt(&mut cfg, &doc, prompt::local_config).await?;
        }
        ProjectType::Dev => {}
        ProjectType::Advanced => {
            prompt::prompt(&mut cfg, &doc, prompt::advanced_config).await?;
        }
        ProjectType::Test => {
            cfg.anchor = ceramic_config::Anchor::clay();
            cfg.network = ceramic_config::Network::clay();
            prompt::prompt(&mut cfg, &doc, prompt::remote_config).await?;
        }
        ProjectType::Production => {
            cfg.anchor = ceramic_config::Anchor::mainnet();
            cfg.network = ceramic_config::Network::mainnet();
            prompt::prompt(&mut cfg, &doc, prompt::remote_config).await?;
        }
    }

    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&cfg)?.as_bytes()).await?;
    f.flush().await?;

    install::ceramic_daemon::install_ceramic_daemon(
        &project.path,
        &cfg,
        &versions.ceramic,
        with_composedb,
    )
    .await?;
    install::compose_db::install_compose_db(&cfg, &doc, &project.path, &versions.composedb).await?;

    if Confirm::new("Install ComposeDB App Template (Next.js)?")
        .with_default(false)
        .prompt()?
    {
        install::ceramic_app_template::install_ceramic_app_template(
            &project.path,
            &versions.app_template,
        )
        .await?;
    }

    Ok(())
}

pub async fn default_for_project_type(
    working_directory: PathBuf,
    project_type: ProjectType,
    versions: Versions,
) -> anyhow::Result<()> {
    let project = Project {
        name: "ceramic-app".to_string(),
        path: working_directory,
    };

    if !project.path.exists() {
        log::info!(
            "Project directory {} does not exist, creating it",
            project.path.display()
        );
        tokio::fs::create_dir_all(&project.path).await?;
    }

    let doc = crate::did::generate_document().await?;

    let cfg_file_path = project.path.join("ceramic.json");
    let mut cfg = ceramic_config::Config::default();
    cfg.http_api.admin_dids.push(doc.id.clone());

    match project_type {
        ProjectType::Local => {
            cfg.anchor = ceramic_config::Anchor::local();
            cfg.network = ceramic_config::Network::local(&project.name);
        }
        ProjectType::Dev => {
            cfg.anchor = ceramic_config::Anchor::dev();
            cfg.network = ceramic_config::Network::dev();
        }
        ProjectType::Advanced => {
            anyhow::bail!("Advanced config not supported for a default project");
        }
        ProjectType::Test => {
            cfg.anchor = ceramic_config::Anchor::clay();
            cfg.network = ceramic_config::Network::clay();
        }
        ProjectType::Production => {
            cfg.anchor = ceramic_config::Anchor::mainnet();
            cfg.network = ceramic_config::Network::mainnet();
            cfg.indexing.enable_historical_sync = true;
        }
    }

    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&cfg)?.as_bytes()).await?;
    f.flush().await?;

    install::ceramic_daemon::install_ceramic_daemon(&project.path, &cfg, &versions.ceramic, true)
        .await?;
    install::compose_db::install_compose_db(&cfg, &doc, &project.path, &versions.composedb).await?;

    Ok(())
}
