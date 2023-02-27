use clap::{Parser, ValueEnum};
use log::LevelFilter;
use std::path::PathBuf;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Network {
    Local,
    Dev,
    Clay,
    Mainnet,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ProgramArgs {
    #[arg(long, short = 'n')]
    network: Option<Network>,
    #[arg(long, short = 'd')]
    working_directory: Option<String>,
    #[arg(long)]
    ceramic_version: Option<String>,
    #[arg(long)]
    composedb_version: Option<String>,
    #[arg(long, short = 'y', default_value_t = false)]
    no_interactive: bool,
    #[arg(long, default_value_t = true)]
    with_compose_db: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init();
    let args = ProgramArgs::parse();
    let current_directory = std::env::current_dir()?;
    let working_directory = args
        .working_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| current_directory);
    let mut versions = wheel_3box::Versions::default();
    if let Some(v) = args.ceramic_version {
        versions.ceramic = Some(v.parse()?);
    }
    if let Some(v) = args.composedb_version {
        versions.composedb = Some(v.parse()?);
    }

    let opt_child = if let Some(network) = args.network {
        let project_type = match network {
            Network::Local => wheel_3box::ProjectType::Local,
            Network::Dev => wheel_3box::ProjectType::Dev,
            Network::Clay => wheel_3box::ProjectType::Test,
            Network::Mainnet => wheel_3box::ProjectType::Production,
        };
        if args.no_interactive {
            wheel_3box::default_for_project_type(
                working_directory,
                project_type,
                versions,
                args.with_compose_db,
            )
            .await?
        } else {
            wheel_3box::for_project_type(working_directory, project_type, versions).await?
        }
    } else {
        log::info!("No network specified, starting interactive configuration");
        wheel_3box::interactive(working_directory, versions).await?
    };

    if let Some(child) = opt_child {
        log::info!("Ceramic is now running in the background. Please use another terminal for additional commands. You can interrupt ceramic using ctrl-c.");
        child.await?;
    }

    Ok(())
}
