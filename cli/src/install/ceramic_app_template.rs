use crate::DidAndPrivateKey;
use std::path::Path;
use tokio::io::AsyncWriteExt;

use crate::install::npm::npm_install;

const REPO: &'static str = "https://github.com/ceramicstudio/ComposeDbExampleApp";

pub async fn install_ceramic_app_template(
    key: &DidAndPrivateKey,
    working_directory: &Path,
    project_name: &str,
    template_branch: &Option<String>,
    daemon_config_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
    log::info!("Setting up application template from {}", REPO);
    let zip_path = format!(
        "/archive/refs/heads/{}.zip",
        template_branch
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("main")
    );
    let data = reqwest::get(format!("{}{}", REPO, zip_path))
        .await?
        .bytes()
        .await?;

    let output_dir = working_directory.join(format!("{}-app", project_name));
    let b_output_dir = working_directory.to_path_buf();

    let unzip_dir = working_directory.join("ComposeDbExampleApp-main");
    if tokio::fs::try_exists(&unzip_dir).await? {
        tokio::fs::remove_dir_all(&unzip_dir).await?;
    }
    if tokio::fs::try_exists(&output_dir).await? {
        tokio::fs::remove_dir_all(&output_dir).await?;
    }
    tokio::task::spawn_blocking(move || {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(data.as_ref()))?;
        z.extract(&b_output_dir)
    })
    .await??;

    tokio::fs::rename(&unzip_dir, &output_dir).await?;

    npm_install(&output_dir, &None, false).await?;

    let readme = output_dir.join("README.md");
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .append(false)
        .truncate(true)
        .create(true)
        .open(&readme)
        .await?;
    f.write_all(r#"
## Getting Started

First, run the development server:

```bash
npm run nextDev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

## Learn More

To learn more about Ceramic please visit the following links

- [Ceramic Documentation](https://developers.ceramic.network/learn/welcome/) - Learn more about the Ceramic Ecosystem.
- [ComposeDB](https://composedb.js.org/) - Details on how to use and develop with ComposeDB!

You can check out [Create Ceramic App repo](https://github.com/ceramicstudio/create-ceramic-app) and provide us with your feedback or contributions!
"#.as_bytes()
    )
        .await?;
    f.flush().await?;

    let demo_config_file = output_dir.join("composedb.config.json");
    tokio::fs::copy(&daemon_config_file, &demo_config_file).await?;

    let seed_file = output_dir.join("admin_seed.txt");
    tokio::fs::write(&seed_file, key.pk().as_bytes()).await?;

    log::info!("Building composites");

    tokio::process::Command::new("npm")
        .current_dir(&output_dir)
        .arg("run")
        .arg("dev")
        .status()
        .await?;

    log::info!(
        r#"Application demo is available at {}. To run the demo application

cd {}
npm run nextDev

See the README at {} for more information"#,
        output_dir.display(),
        output_dir.display(),
        readme.display()
    );

    Ok(())
}
