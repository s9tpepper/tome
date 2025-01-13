use serde::{Deserialize, Serialize};

use crate::projects::{PersistedEndpoint, PersistedProject, PersistedVariable};

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfirmAction {
    ConfirmDeletePersistedProject(DeleteProjectDetails),
    ConfirmationDeletePersistedProject(DeleteProjectDetailsAnswer),

    ConfirmDeletePersistedEndpoint(DeleteEndpointDetails),
    ConfirmationDeletePersistedEndpoint(DeleteEndpointDetailsAnswer),

    ConfirmDeletePersistedProjectVariable(DeletePersistedVariableDetails),
    ConfirmationDeletePersistedProjectVariable(DeletePersistedVariableAnswer),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteEndpointDetails {
    pub endpoint: PersistedEndpoint,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteEndpointDetailsAnswer {
    pub endpoint: PersistedEndpoint,
    pub answer: bool,
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
