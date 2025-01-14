use serde::{Deserialize, Serialize};

use crate::projects::{PersistedEndpoint, PersistedProject, PersistedVariable};

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfirmAction {
    ConfirmDeletePersistedVariable(DeleteVariableDetails),
    ConfirmationDeletePersistedVariable(DeleteVariableDetailsAnswer),

    ConfirmDeletePersistedProject(DeleteProjectDetails),
    ConfirmationDeletePersistedProject(DeleteProjectDetailsAnswer),

    ConfirmDeletePersistedEndpoint(DeleteEndpointDetails),
    ConfirmationDeletePersistedEndpoint(DeleteEndpointDetailsAnswer),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteVariableDetails {
    pub variable: PersistedVariable,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteVariableDetailsAnswer {
    pub variable: PersistedVariable,
    pub answer: bool,
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
