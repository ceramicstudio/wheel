mod install;
mod prompt;

use inquire::*;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub struct Versions {
    pub ceramic: semver::Version,
    pub kubo: semver::Version,
    pub composedb: semver::Version,
    pub app_template: semver::Version,
}

impl Default for Versions {
    fn default() -> Self {
        Self {
            ceramic: semver::Version::new(2, 20, 0),
            kubo: semver::Version::new(0, 18, 1),
            composedb: semver::Version::new(0, 3, 0),
            app_template: semver::Version::new(0, 1, 1),
        }
    }
}

enum ProjectType {
    Local,
    Test,
    Production,
    Advanced,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Test => write!(f, "Test (Clay)"),
            Self::Production => write!(f, "Production (Mainnet)"),
            Self::Advanced => write!(f, "Advanced"),
        }
    }
}

pub async fn interactive() -> anyhow::Result<()> {
    let ans = Select::new(
        "Project Type",
        vec![
            ProjectType::Local,
            ProjectType::Test,
            ProjectType::Production,
            ProjectType::Advanced,
        ],
    )
    .with_help_message("Local nodes will not anchor")
    .prompt()?;

    let project = prompt::project::configure_project().await?;

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

    match ans {
        ProjectType::Local => {
            cfg.anchor = ceramic_config::Anchor::local();
            cfg.network = ceramic_config::Network::local(&project.name);
            prompt::prompt(&mut cfg, &doc, prompt::local_config).await?;
        }
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
        .open(cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&cfg)?.as_bytes()).await?;
    f.flush().await?;

    install::kubo::install_kubo(&project.path, &None).await?;
    install::ceramic_daemon::install_ceramic_daemon(&project.path, &None, with_composedb).await?;
    install::compose_db::install_compose_db(&cfg, &doc, &project.path, &None).await?;

    if Confirm::new("Install ComposeDB App Template (Next.js)?")
        .with_default(false)
        .prompt()?
    {
        install::ceramic_app_template::install_ceramic_app_template(&project.path).await?;
    }

    Ok(())
}
