use inquire::*;
use std::path::PathBuf;

pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

pub async fn configure_project(working_directory: PathBuf) -> anyhow::Result<Project> {
    let project_name = Text::new("Project Name")
        .with_default("ceramic-test-app")
        .prompt()?;
    let project_path = working_directory.join(&project_name);
    let project_path = Text::new("Project Path")
        .with_default(project_path.to_string_lossy().as_ref())
        .prompt()?;
    let project_path = PathBuf::from(project_path);
    if !project_path.exists() {
        log::info!("Project directory {} does not exist, creating it", project_path.display());
        tokio::fs::create_dir_all(&project_path).await?;
    }
    Ok(Project {
        name: project_name,
        path: project_path,
    })
}
