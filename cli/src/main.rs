use clap::{Parser, Subcommand, ValueEnum};
use log::LevelFilter;
use ssi::did::Document;
use std::fmt::Formatter;
use std::io::Write;
use std::path::PathBuf;
use wheel_3box::DidAndPrivateKey;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Network {
    InMemory,
    Local,
    Dev,
    Clay,
    Mainnet,
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InMemory => write!(f, "in-memory"),
            Self::Local => write!(f, "local"),
            Self::Dev => write!(f, "dev"),
            Self::Clay => write!(f, "clay"),
            Self::Mainnet => write!(f, "mainnet"),
        }
    }
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Setup {
    CeramicOnly,
    ComposeDB,
    DemoApplication,
}

impl std::fmt::Display for Setup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CeramicOnly => write!(f, "ceramic-only"),
            Self::ComposeDB => write!(f, "compose-db"),
            Self::DemoApplication => write!(f, "demo-application"),
        }
    }
}

#[derive(Clone, Debug, Parser)]
struct DidOptions {
    #[arg(long)]
    did: String,
    #[arg(long, help = "Valid Ed25519 private key")]
    private_key: String,
}

#[derive(Subcommand, Debug)]
enum DidCommand {
    #[command(about = "Generate a new did and private key")]
    Generate,
    // Specify a did and private key
    #[command(about = "Input a did and private key")]
    Specify(DidOptions),
}

#[derive(Parser, Debug)]
struct QuietOptions {
    #[arg(long)]
    project_name: Option<String>,
    #[arg(long, short = 'n', default_value_t = Network::Clay)]
    network: Network,
    #[command(subcommand)]
    did: DidCommand,
    #[arg(long, default_value_t = Setup::ComposeDB)]
    setup: Setup,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Non-interactive setup for ceramic and compose-db")]
    Quiet(QuietOptions),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ProgramArgs {
    #[arg(long, short = 'd')]
    working_directory: Option<String>,
    #[arg(long)]
    ceramic_version: Option<String>,
    #[arg(long)]
    composedb_version: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(LevelFilter::Info)
        .filter_module("sqlx", LevelFilter::Warn)
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

    let opt_child = match args.command {
        None => {
            log::info!("Starting wheel interactive configuration");
            wheel_3box::interactive(working_directory, versions).await?
        }
        Some(Commands::Quiet(q)) => {
            let network = match q.network {
                Network::InMemory => wheel_3box::NetworkIdentifier::InMemory,
                Network::Local => wheel_3box::NetworkIdentifier::Local,
                Network::Dev => wheel_3box::NetworkIdentifier::Dev,
                Network::Clay => wheel_3box::NetworkIdentifier::Clay,
                Network::Mainnet => wheel_3box::NetworkIdentifier::Mainnet,
            };
            let did = if let DidCommand::Specify(opts) = q.did {
                DidAndPrivateKey::new(opts.private_key, Document::new(&opts.did))
            } else {
                DidAndPrivateKey::generate()?
            };
            let with_composedb = q.setup == Setup::ComposeDB;
            let with_app_template = q.setup == Setup::DemoApplication;
            wheel_3box::quiet(wheel_3box::QuietOptions {
                project_name: q.project_name,
                working_directory: working_directory,
                network_identifier: network,
                versions,
                did,
                with_ceramic: with_app_template || with_composedb || q.setup == Setup::CeramicOnly,
                with_composedb: with_app_template || with_composedb,
                with_app_template: with_app_template,
            })
            .await?
        }
    };

    log::info!("Wheel setup is complete. If running a clay or mainnet node, please check out https://github.com/ceramicstudio/simpledeploy to deploy with k8s.");

    if let Some(child) = opt_child {
        log::info!("Ceramic is now running in the background. Please use another terminal for additional commands. You can interrupt ceramic using ctrl-c.");
        child.await?;
    } else {
        log::info!("Wheel setup is complete.");
    }

    Ok(())
}
