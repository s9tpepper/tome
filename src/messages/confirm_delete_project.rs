use serde::{Deserialize, Serialize};

use crate::projects::{PersistedProject, PersistedVariable};

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfirmAction {
    ConfirmDeletePersistedProject(DeleteProjectDetails),
    ConfirmationDeletePersistedProject(DeleteProjectDetailsAnswer),

    ConfirmDeletePersistedProjectVariable(DeletePersistedVariableDetails),
    ConfirmationDeletePersistedProjectVariable(DeletePersistedVariableAnswer),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteProjectDetails {
    pub project: PersistedProject,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteProjectDetailsAnswer {
    pub project: PersistedProject,
    pub answer: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePersistedVariableDetails {
    pub project: PersistedVariable,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePersistedVariableAnswer {
    pub project: PersistedVariable,
    pub answer: bool,
}
