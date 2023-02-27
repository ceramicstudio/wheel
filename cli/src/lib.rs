mod did;
mod install;
mod prompt;

use inquire::*;
use prompt::project::Project;
use std::path::PathBuf;
use tokio::{io::AsyncWriteExt, task::JoinHandle};

#[derive(Default)]
pub struct Versions {
    pub ceramic: Option<semver::Version>,
    pub composedb: Option<semver::Version>,
    pub app_template: Option<semver::Version>,
}

pub enum ProjectType {
    InMemory,
    Local,
    Dev,
    Test,
    Production,
    Advanced,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InMemory => write!(f, "InMemory"),
            Self::Local => write!(f, "Local"),
            Self::Dev => write!(f, "Dev"),
            Self::Test => write!(f, "Test (Clay)"),
            Self::Production => write!(f, "Production (Mainnet)"),
            Self::Advanced => write!(f, "Advanced"),
        }
    }
}

pub async fn interactive(
    working_directory: PathBuf,
    versions: Versions,
) -> anyhow::Result<Option<JoinHandle<()>>> {
    let ans = Select::new(
        "Project Type",
        vec![
            ProjectType::InMemory,
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
) -> anyhow::Result<Option<JoinHandle<()>>> {
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

    cfg.http_api.admin_dids.push(doc.did().to_string());

    match project_type {
        ProjectType::InMemory => {
            cfg.in_memory();
            prompt::prompt(&mut cfg, &doc, prompt::local_config).await?;
        }
        ProjectType::Local => {
            cfg.local(&project.name);
            prompt::prompt(&mut cfg, &doc, prompt::local_config).await?;
        }
        ProjectType::Dev => {
            prompt::prompt(&mut cfg, &doc, prompt::local_config).await?;
            cfg.dev();
        }
        ProjectType::Advanced => {
            prompt::prompt(&mut cfg, &doc, prompt::advanced_config).await?;
        }
        ProjectType::Test => {
            cfg.test();
            prompt::prompt(&mut cfg, &doc, prompt::remote_config).await?;
        }
        ProjectType::Production => {
            cfg.production();
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

    let opt_child = install::ceramic_daemon::install_ceramic_daemon(
        &project.path,
        &cfg,
        &versions.ceramic,
        false,
    )
    .await?;
    if with_composedb {
        install::compose_db::install_compose_db(&cfg, &doc, &project.path, &versions.composedb)
            .await?;

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
    }

    Ok(opt_child)
}

pub async fn default_for_project_type(
    working_directory: PathBuf,
    project_type: ProjectType,
    versions: Versions,
    with_composedb: bool,
) -> anyhow::Result<Option<JoinHandle<()>>> {
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

    let doc = crate::did::DidAndPrivateKey::generate()?;

    let cfg_file_path = project.path.join("ceramic.json");
    let mut cfg = ceramic_config::Config::default();

    match project_type {
        ProjectType::InMemory => {
            cfg.in_memory();
        }
        ProjectType::Local => {
            cfg.local(&project.name);
        }
        ProjectType::Dev => {
            cfg.dev();
        }
        ProjectType::Advanced => {
            anyhow::bail!("Advanced config not supported for a default project");
        }
        ProjectType::Test => {
            cfg.test();
        }
        ProjectType::Production => {
            cfg.production();
        }
    }
    cfg.http_api.admin_dids.push(doc.did().to_string());

    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&cfg)?.as_bytes()).await?;
    f.flush().await?;

    let opt_child = install::ceramic_daemon::install_ceramic_daemon(
        &project.path,
        &cfg,
        &versions.ceramic,
        true,
    )
    .await?;
    if with_composedb {
        install::compose_db::install_compose_db(&cfg, &doc, &project.path, &versions.composedb)
            .await?;
    }

    Ok(opt_child)
}
