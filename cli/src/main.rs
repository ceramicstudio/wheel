use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ProgramArgs {
    #[arg(long, short = 'i', default_value_t = true)]
    interactive: bool,
    #[arg(long, short = 'd')]
    working_directory: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env_logger::builder().try_init();
    let args = ProgramArgs::parse();

    if args.interactive {
        wheel::interactive().await?;
    }

    Ok(())
}
