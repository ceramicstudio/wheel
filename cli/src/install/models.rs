use askama::Template;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Template)]
#[template(path = "basic_profile.graphql.jinja")]
struct BasicProfile {}

#[derive(Template)]
#[template(path = "following.graphql.jinja")]
struct Following<'a> {
    #[allow(dead_code)]
    model: &'a str,
}

#[derive(Template)]
#[template(path = "posts.graphql.jinja")]
struct Posts<'a> {
    #[allow(dead_code)]
    model: &'a str,
}

#[derive(Template)]
#[template(path = "posts_profile.graphql.jinja")]
struct PostsProfile<'a> {
    #[allow(dead_code)]
    profile_model: &'a str,
    #[allow(dead_code)]
    posts_model: &'a str,
}

#[derive(serde::Deserialize)]
struct CreateOutput {
    models: HashMap<String, serde_json::Value>,
}

struct PathWrapper {
    path: PathBuf,
    path_str: String,
}

impl PathWrapper {
    pub fn new(path: impl AsRef<Path>, file: &str) -> Self {
        let path = path.as_ref().join(file);
        let path_str = path.display().to_string();
        Self { path, path_str }
    }

    pub fn as_str(&self) -> &str {
        &self.path_str
    }
}

impl std::fmt::Display for PathWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl AsRef<Path> for PathWrapper {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

fn composedb_command(working_directory: impl AsRef<Path>) -> Command {
    let mut cmd = Command::new("sh");
    cmd.current_dir(working_directory.as_ref());
    cmd.args(&["composedb"]);
    cmd
}

fn create_command(working_directory: impl AsRef<Path>) -> Command {
    let mut cmd = composedb_command(working_directory);
    cmd.args(&["composite:create", "--output"]);
    cmd
}

pub async fn build_composite(
    working_directory: impl AsRef<Path>,
    app_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let app_dir = app_dir.as_ref().join("src").join("__generated__");
    log::debug!(
        "Building composite model in {}, using composedb from {}",
        app_dir.display(),
        working_directory.as_ref().display()
    );
    if !tokio::fs::try_exists(&app_dir).await? {
        tokio::fs::create_dir_all(&app_dir).await?;
    }

    let basic_profile_path = PathWrapper::new(&app_dir, "basic_profile.graphql");
    let basic_profile = BasicProfile {}.render()?;
    tokio::fs::write(&basic_profile_path, basic_profile.as_bytes()).await?;
    let basic_profile_out = PathWrapper::new(&app_dir, "basic_profile.json");
    log::debug!(
        "Creating BasicProfile model {} from schema {}",
        basic_profile_out,
        basic_profile_path
    );
    let status = create_command(&working_directory)
        .args(&[basic_profile_out.as_str(), basic_profile_path.as_str()])
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to create basic_profile model");
    }
    let create_output: CreateOutput =
        serde_json::from_slice(tokio::fs::read(&basic_profile_out).await?.as_ref())?;
    let profile_model = create_output
        .models
        .keys()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No models created"))?
        .to_string();

    let following_path = PathWrapper::new(&app_dir, "following.graphql");
    let following = Following {
        model: &profile_model,
    }
    .render()?;
    tokio::fs::write(&following_path, following.as_bytes()).await?;
    let following_out = PathWrapper::new(&app_dir, "following.json");
    log::debug!(
        "Creating Following model {} from schema {}",
        following_out,
        following_path
    );
    let status = create_command(&working_directory)
        .args(&[following_out.as_str(), following_path.as_str()])
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to create following model");
    }

    let posts_path = PathWrapper::new(&app_dir, "posts.graphql");
    let posts = Posts {
        model: &profile_model,
    }
    .render()?;
    tokio::fs::write(&posts_path, posts.as_bytes()).await?;
    let posts_out = PathWrapper::new(&app_dir, "posts.json");
    log::debug!(
        "Creating Posts model {} from schema {}",
        posts_out,
        posts_path
    );
    let status = create_command(&working_directory)
        .args(&[posts_out.as_str(), posts_path.as_str()])
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to create posts model");
    }
    let create_output: CreateOutput =
        serde_json::from_slice(tokio::fs::read(&posts_out).await?.as_ref())?;
    let posts_model = create_output
        .models
        .keys()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No models created"))?
        .to_string();

    let posts_profile_path = PathWrapper::new(&app_dir, "posts_profile.graphql");
    let posts_profile = PostsProfile {
        posts_model: &posts_model,
        profile_model: &profile_model,
    }
    .render()?;
    tokio::fs::write(&posts_profile_path, posts_profile.as_bytes()).await?;
    let posts_profile_out = PathWrapper::new(&app_dir, "posts_profile.json");
    log::debug!(
        "Creating PostsProfile model {} from schema {}",
        posts_profile_out,
        posts_profile_path
    );
    let status = create_command(&working_directory)
        .args(&[posts_profile_out.as_str(), posts_profile_path.as_str()])
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to create posts profile model");
    }

    let merged_path = PathWrapper::new(&app_dir, "merged.json");
    let status = composedb_command(&working_directory)
        .args(&[
            "composite:merge",
            basic_profile_out.as_str(),
            posts_out.as_str(),
            following_out.as_str(),
            posts_profile_out.as_str(),
            "--output",
            merged_path.as_str(),
        ])
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to merge models");
    }

    let status = composedb_command(&working_directory)
        .args(&["composite:deploy", merged_path.as_str()])
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to deploy merged model");
    }

    let runtime_path = PathWrapper::new(&app_dir, "definition.js");
    let status = composedb_command(&working_directory)
        .args(&[
            "composite:compile",
            merged_path.as_str(),
            runtime_path.as_str(),
        ])
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to deploy merged model");
    }

    Ok(())
}
