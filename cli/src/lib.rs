mod install;
mod prompt;

use inquire::*;

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
    .with_help_message("Test and Production will use existing ceramic nodes")
    .prompt()?;

    let project = prompt::project::configure_project().await?;

    let cfg = match ans {
        ProjectType::Local => {
            let doc = prompt::did::generate_did(&project.path).await?;
            let cfg = prompt::ceramic_local_config::prompt(&project.path, &doc).await?;
            install::ceramic_daemon::install_ceramic_daemon(&project.path, &None).await?;
            install::kubo::install_kubo(&project.path, &None).await?;
            cfg
        }
        ProjectType::Advanced => {
            let doc = prompt::did::generate_did(&project.path).await?;
            let cfg = prompt::ceramic_config::prompt(&project.path, &doc).await?;
            install::ceramic_daemon::install_ceramic_daemon(&project.path, &None).await?;
            install::kubo::install_kubo(&project.path, &None).await?;
            cfg
        }
        ProjectType::Test => {
            let mut cfg = ceramic_config::Config::default();
            cfg.anchor = ceramic_config::Anchor::clay();
            cfg.network = ceramic_config::Network::clay();
            cfg
        }
        ProjectType::Production => {
            let mut cfg = ceramic_config::Config::default();
            cfg.anchor = ceramic_config::Anchor::mainnet();
            cfg.network = ceramic_config::Network::mainnet();
            cfg
        }
    };

    install::compose_db::install_compose_db(&cfg, &project.path, &None).await?;

    if Confirm::new("Install ComposeDB App Template (Next.js)?")
        .with_default(false)
        .prompt()?
    {
        install::ceramic_app_template::install_ceramic_app_template(&project.path).await?;
    }

    Ok(())
}
