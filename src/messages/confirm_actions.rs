use serde::{Deserialize, Serialize};

use crate::projects::{Header, PersistedEndpoint, PersistedProject, PersistedVariable};

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfirmAction {
    ConfirmDeletePersistedVariable(ConfirmDetails<PersistedVariable>),
    ConfirmationDeletePersistedVariable(ConfirmationAnswer<PersistedVariable>),

    ConfirmDeletePersistedProject(ConfirmDetails<PersistedProject>),
    ConfirmationDeletePersistedProject(ConfirmationAnswer<PersistedProject>),

    ConfirmDeletePersistedEndpoint(ConfirmDetails<PersistedEndpoint>),
    ConfirmationDeletePersistedEndpoint(ConfirmationAnswer<PersistedEndpoint>),

    ConfirmDeleteHeader(ConfirmDetails<Header>),
    ConfirmationDeleteHeader(ConfirmationAnswer<Header>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmDetails<T> {
    pub data: T,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmationAnswer<T> {
    pub data: T,
    pub answer: bool,
}
