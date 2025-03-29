use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anathema::prelude::Context;
use directories::{ProjectDirs, UserDirs};

use crate::components::dashboard::{DashboardComponent, DashboardState};

pub fn get_project_directory<'a>(app: &'a str, path: &'a str) -> anyhow::Result<PathBuf> {
    let requested_path = ProjectDirs::from("com", "s9tpepper", app)
        .map(|project_dirs| project_dirs.data_dir().join(path));

    let path = requested_path.ok_or(anyhow::Error::msg("Could not build requested path"))?;
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    Ok(path)
}

pub fn get_app_dir(path: &str) -> anyhow::Result<PathBuf> {
    get_project_directory("Tome", path)
}

pub fn get_documents_dir() -> anyhow::Result<PathBuf> {
    let user_dirs = UserDirs::new();
    let dirs = user_dirs.ok_or(Err(anyhow::Error::msg("Could not get user directories")));

    match dirs {
        Ok(dirs) => {
            let docs_dir = dirs
                .document_dir()
                .ok_or(anyhow::Error::msg("Could not get user directories"))?;

            Ok(docs_dir.to_owned())
        }

        Err(error) => Err(error?),
    }
}

pub fn save_response(dashboard: &DashboardComponent, state: &mut DashboardState) {
    let dir = get_documents_dir();

    match dir {
        Ok(mut docs_dir) => {
            let response = state.response.to_ref().to_string();

            let endpoint_name = state.endpoint.to_ref().name.to_ref().to_string();
            let endpoint_name = endpoint_name.replace("/", "_");

            let timestamp = SystemTime::now();
            let duration = timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(1));
            let name = format!("{endpoint_name}_{}.txt", duration.as_secs());

            docs_dir.push(state.project.to_ref().name.to_ref().to_string());
            let _ = fs::create_dir_all(&docs_dir);

            match fs::exists(&docs_dir) {
                Ok(docs_exist) => {
                    if !docs_exist {
                        dashboard.show_error(
                            "Couldn't create project directory to save response",
                            state,
                        );

                        return;
                    }

                    docs_dir.push(name);

                    let save_path = docs_dir.clone();

                    match fs::write(docs_dir, response) {
                        Ok(_) => {
                            dashboard.show_message(
                                "Response Saved",
                                format!("Saved to {save_path:?}").as_str(),
                                state,
                            );
                        }
                        Err(err) => dashboard.show_error(&err.to_string(), state),
                    }
                }
                Err(_) => {
                    dashboard
                        .show_error("Couldn't create project directory to save response", state);
                }
            }
        }
        Err(error) => dashboard.show_error(&error.to_string(), state),
    }
}
