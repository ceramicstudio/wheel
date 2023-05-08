use inquire::*;
use std::path::{Path, PathBuf};

pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

pub async fn configure_project(working_directory: impl AsRef<Path>) -> anyhow::Result<Project> {
    let project_name = Text::new("Project Name")
        .with_default("ceramic-test-app")
        .prompt()?;
    let project_path = working_directory.as_ref().join(&project_name);
    let project_path = Text::new("Project Path")
        .with_default(project_path.to_string_lossy().as_ref())
        .prompt()?;
    let project_path = PathBuf::from(project_path);
    if tokio::fs::try_exists(&project_path).await? {
        if !Confirm::new("You are setting up your project in a non-empty directory. Continue?")
            .with_default(false)
            .prompt()?
        {
            anyhow::bail!("Aborting project setup");
        }
    } else {
        log::info!(
            "Project directory {} does not exist, creating it",
            project_path.display()
        );
        tokio::fs::create_dir_all(&project_path).await?;
    }
    Ok(Project {
        name: project_name,
        path: project_path.to_path_buf(),
    })
}
