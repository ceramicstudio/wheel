use std::path::PathBuf;
use inquire::*;

pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

pub async fn configure_project() -> anyhow::Result<Project> {
    let project_name = Text::new("Project Name")
        .with_default("ceramic-test-app")
        .prompt()?;
    let current_dir = std::env::current_dir()?;
    let project_path = Text::new("Project Path")
        .with_default(current_dir.to_string_lossy().as_ref())
        .prompt()?;
    Ok(Project {
        name: project_name,
        path: PathBuf::from(project_path),
    })
}