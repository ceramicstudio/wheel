use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ProgramArgs {
    #[arg(long, short = "i", default = true)]
    interactive: bool,
    #[arg(long, short = "d", default = ".")]
    working_directory: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env_logger::builder().try_init();
    let args = ProgramArgs::parse();

    if args.interactive {
        wheel::interactive().await?;
    }
}
