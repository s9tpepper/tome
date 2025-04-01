use anathema::state::{self, List, State, Value};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufReader,
    ops::Deref,
    path::PathBuf,
};

use crate::fs::get_app_dir;

// TODO: Fix the default project row color to the correct gray
pub const DEFAULT_ROW_COLOR: &str = "#333333";

// TODO: Implement using this constant for selected rows
#[allow(unused)]
pub const SELECTED_ROW_COLOR: &str = "#FFFFFF";

pub const DEFAULT_PROJECT_NAME: &str = "Unnamed";
pub const DEFAULT_ENDPOINT_NAME: &str = "Unnamed";

#[derive(anathema::state::State)]
pub struct Project {
    pub name: Value<String>,
    pub endpoints: Value<List<Endpoint>>,
    pub row_color: Value<String>,
    pub row_fg_color: Value<String>,
    pub variable: Value<List<ProjectVariable>>,
}

#[derive(Default, Debug)]
pub enum ProjectVariableType {
    #[default]
    String,
    Boolean,
    Any,
    Number,
}

impl State for ProjectVariableType {
    fn to_common(&self) -> Option<state::CommonVal<'_>> {
        match self {
            ProjectVariableType::String => Some(state::CommonVal::Str("String")),
            ProjectVariableType::Boolean => Some(state::CommonVal::Str("Boolean")),
            ProjectVariableType::Any => Some(state::CommonVal::Str("Any")),
            ProjectVariableType::Number => Some(state::CommonVal::Str("Number")),
        }
    }
}

#[derive(State, Default, Debug)]
pub struct ProjectVariable {
    pub id: Value<String>,
    pub key: Value<String>,
    pub value: Value<String>,
    pub r#type: Value<ProjectVariableType>,
    pub name: Value<String>,
    pub system: Value<bool>,
    pub disabled: Value<bool>,
    pub private: Value<String>,

    pub row_fg_color: Value<String>,
    pub row_color: Value<String>,
}

impl From<PersistedVariable> for ProjectVariable {
    fn from(persisted_variable: PersistedVariable) -> Self {
        ProjectVariable {
            id: persisted_variable.id.unwrap_or_default().into(),
            key: persisted_variable.key.unwrap_or_default().into(),
            value: persisted_variable.value.unwrap_or_default().into(),
            r#type: match persisted_variable.r#type.unwrap_or_default() {
                VariableType::String => ProjectVariableType::String.into(),
                VariableType::Boolean => ProjectVariableType::Boolean.into(),
                VariableType::Any => ProjectVariableType::Any.into(),
                VariableType::Number => ProjectVariableType::Number.into(),
            },
            name: persisted_variable.name.unwrap_or_default().into(),
            system: persisted_variable.system.unwrap_or_default().into(),
            disabled: persisted_variable.disabled.unwrap_or_default().into(),
            private: persisted_variable.private.unwrap_or_default().into(),

            row_color: DEFAULT_ROW_COLOR.to_string().into(),
            row_fg_color: DEFAULT_ROW_COLOR.to_string().into(),
        }
    }
}

impl Project {
    pub fn new() -> Self {
        Project {
            name: String::from(DEFAULT_PROJECT_NAME).into(),
            row_color: DEFAULT_ROW_COLOR.to_string().into(),
            row_fg_color: DEFAULT_ROW_COLOR.to_string().into(),
            endpoints: List::empty(),
            variable: List::empty(),
        }
    }

    pub fn reset(&mut self) {
        self.name.set(String::from(DEFAULT_PROJECT_NAME));
        self.row_color.set(DEFAULT_ROW_COLOR.to_string());
        self.row_fg_color.set(DEFAULT_ROW_COLOR.to_string());

        self.clear_endpoints();
        self.clear_variables();
    }

    pub fn update_from_persisted(&mut self, project: &PersistedProject) {
        self.clear_endpoints();
        self.clear_variables();
        self.name.set(project.name.clone());
        self.update_endpoints(&project.endpoints);
        self.update_variables(&project.variable);
    }

    pub fn update_variables(&mut self, variables: &[PersistedVariable]) {
        variables.iter().for_each(|persisted_variable| {
            // TODO: Fix this clone I shouldn't have to do this
            let e: ProjectVariable = (persisted_variable.clone()).into();
            self.variable.insert(0, e);
        });
    }

    pub fn update_endpoints(&mut self, endpoints: &[PersistedEndpoint]) {
        endpoints.iter().for_each(|ep| {
            let e: Endpoint = ep.into();
            self.endpoints.insert(0, e);
        });
    }

    pub fn clear_variables(&mut self) {
        while self.variable.len() > 0 {
            self.variable.remove(0);
        }
    }

    pub fn clear_endpoints(&mut self) {
        while self.endpoints.len() > 0 {
            self.endpoints.remove(0);
        }
    }
}

#[derive(Debug, State)]
pub struct Endpoint {
    pub name: Value<String>,
    pub url: Value<String>,
    pub method: Value<String>,
    pub headers: Value<List<HeaderState>>,
    pub body: Value<String>,
    pub row_color: Value<String>,
    pub row_fg_color: Value<String>,
    pub body_mode: Value<String>,
    pub raw_type: Value<String>,
}

impl Endpoint {
    pub fn new() -> Self {
        Endpoint {
            name: String::from(DEFAULT_ENDPOINT_NAME).into(),
            url: String::from("").into(),
            method: String::from("GET").into(),
            body: String::from("").into(),
            body_mode: String::from("raw").into(),
            raw_type: String::from("text").into(),
            headers: List::from_iter(get_default_headers()),
            row_color: DEFAULT_ROW_COLOR.to_string().into(),
            row_fg_color: DEFAULT_ROW_COLOR.to_string().into(),
        }
    }

    pub fn update(&mut self, endpoint: &Endpoint) {
        self.clear_headers();
        self.add_headers(&endpoint.headers);

        self.name.set(endpoint.name.to_ref().to_string());
        self.url.set(endpoint.url.to_ref().to_string());
        self.body.set(endpoint.body.to_ref().to_string());
        self.method.set(endpoint.method.to_ref().to_string());
        self.body_mode.set(endpoint.body_mode.to_ref().to_string());
        self.raw_type.set(endpoint.raw_type.to_ref().to_string());
    }

    fn add_headers(&mut self, headers: &Value<List<HeaderState>>) {
        headers.to_ref().iter().for_each(|h| {
            let header = h.to_ref();

            // TODO: Figure out removing this clone() call
            self.headers.insert(0, header.clone());
        });
    }

    fn clear_headers(&mut self) {
        while self.headers.len() > 0 {
            self.headers.remove(0);
        }
    }

    pub fn reset(&mut self) {
        self.name.set(String::from(DEFAULT_ENDPOINT_NAME));
        self.url.set(String::from(""));
        self.method.set(String::from("GET"));
        self.body.set(String::from(""));
        self.body_mode.set(String::from("raw"));
        self.raw_type.set(String::from("text"));
        self.row_color.set(DEFAULT_ROW_COLOR.to_string());
        self.row_fg_color.set(DEFAULT_ROW_COLOR.to_string());

        while self.headers.len() > 0 {
            self.headers.remove(0);
        }

        get_default_headers().iter().for_each(|header| {
            self.headers.push(header.clone());
        });
    }

    pub fn clone(&self) -> Self {
        let headers_list = self.headers.to_ref();
        let headers = headers_list.iter().map(|header| {
            let h = header.to_ref();
            h.clone()
        });

        Endpoint {
            name: self.name.to_ref().to_string().into(),
            url: self.url.to_ref().to_string().into(),
            method: self.method.to_ref().to_string().into(),
            body: self.body.to_ref().to_string().into(),
            body_mode: self.body_mode.to_ref().to_string().into(),
            raw_type: self.raw_type.to_ref().to_string().into(),
            row_color: DEFAULT_ROW_COLOR.to_string().into(),
            row_fg_color: DEFAULT_ROW_COLOR.to_string().into(),
            headers: List::from_iter(headers),
        }
    }
}

#[derive(Debug, Default, State)]
pub struct HeaderState {
    pub name: Value<String>,
    pub value: Value<String>,
    pub row_color: Value<String>,
    pub row_fg_color: Value<String>,
}

impl HeaderState {
    pub fn clone(&self) -> Self {
        HeaderState {
            name: self.name.to_ref().to_string().into(),
            value: self.value.to_ref().to_string().into(),
            row_color: "".to_string().into(),
            row_fg_color: "".to_string().into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedProject {
    pub name: String,
    pub endpoints: Vec<PersistedEndpoint>,
    pub variable: Vec<PersistedVariable>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum VariableType {
    #[default]
    String,
    Boolean,
    Any,
    Number,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PersistedVariable {
    pub id: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub private: Option<String>,
    pub r#type: Option<VariableType>,
    pub name: Option<String>,
    pub system: Option<bool>,
    pub disabled: Option<bool>,
}

impl From<&ProjectVariable> for PersistedVariable {
    fn from(project_variable: &ProjectVariable) -> Self {
        PersistedVariable {
            id: Some(project_variable.id.to_ref().to_string()),
            key: Some(project_variable.key.to_ref().to_string()),
            value: Some(project_variable.value.to_ref().to_string()),
            private: Some(project_variable.private.to_ref().to_string()),
            r#type: Some(match *project_variable.r#type.to_ref() {
                ProjectVariableType::String => VariableType::String,
                ProjectVariableType::Boolean => VariableType::Boolean,
                ProjectVariableType::Any => VariableType::Any,
                ProjectVariableType::Number => VariableType::Number,
            }),
            name: Some(project_variable.name.to_ref().to_string()),
            system: Some(*project_variable.system.to_ref()),
            disabled: Some(*project_variable.disabled.to_ref()),
        }
    }
}

impl From<ProjectVariable> for PersistedVariable {
    fn from(project_variable: ProjectVariable) -> Self {
        PersistedVariable {
            id: Some(project_variable.id.to_ref().to_string()),
            key: Some(project_variable.key.to_ref().to_string()),
            value: Some(project_variable.value.to_ref().to_string()),
            private: Some(project_variable.private.to_ref().to_string()),
            r#type: Some(match *project_variable.r#type.to_ref() {
                ProjectVariableType::String => VariableType::String,
                ProjectVariableType::Boolean => VariableType::Boolean,
                ProjectVariableType::Any => VariableType::Any,
                ProjectVariableType::Number => VariableType::Number,
            }),
            name: Some(project_variable.name.to_ref().to_string()),
            system: Some(*project_variable.system.to_ref()),
            disabled: Some(*project_variable.disabled.to_ref()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedEndpoint {
    pub name: String,
    pub url: String,
    pub method: String,
    pub headers: Vec<Header>,
    pub body: String,
    pub body_mode: String,
    pub raw_type: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

fn get_default_headers() -> Vec<HeaderState> {
    vec![
        HeaderState {
            name: "user-agent".to_string().into(),
            value: "tome-tui".to_string().into(),
            row_color: "".to_string().into(),
            row_fg_color: "".to_string().into(),
        },
        HeaderState {
            name: "content-type".to_string().into(),
            value: "application/json".to_string().into(),
            row_color: "".to_string().into(),
            row_fg_color: "".to_string().into(),
        },
    ]
}

fn get_project(project_path: &PathBuf) -> anyhow::Result<PersistedProject> {
    let file = File::open(project_path)?;
    let reader = BufReader::new(file);

    let persisted_project: PersistedProject = serde_json::from_reader(reader)?;

    Ok(persisted_project)
}

pub fn rename_endpoint(
    project_name: &str,
    endpoint: &PersistedEndpoint,
    new_name: &str,
) -> anyhow::Result<()> {
    let dir_result = get_app_dir("projects");
    if dir_result.is_err() {
        return Err(anyhow::Error::msg("Unable to access projects directory"));
    }

    let mut project_dir = dir_result.unwrap();
    project_dir.push(format!("{}.project", project_name));

    let mut persisted_project = get_project(&project_dir)?;
    delete_project(&persisted_project)?;

    let mut endpoints: Vec<PersistedEndpoint> = persisted_project
        .endpoints
        .iter()
        .filter(|e| e.name != endpoint.name)
        .map(|e| (*e).clone())
        .collect();

    let mut new_endpoint = endpoint.clone();
    new_endpoint.name = new_name.to_string();

    endpoints.push(new_endpoint);
    persisted_project.endpoints = endpoints;

    save_project(&persisted_project)
}

pub fn rename_project(project: &PersistedProject, new_name: &str) -> anyhow::Result<()> {
    let dir_result = get_app_dir("projects");
    if dir_result.is_err() {
        return Err(anyhow::Error::msg("Unable to access projects directory"));
    }

    let mut old_project_dir = dir_result.unwrap();
    let mut new_project_dir = old_project_dir.clone();

    old_project_dir.push(format!("{}.project", project.name));
    new_project_dir.push(format!("{}.project", new_name));

    let mut persisted_project = get_project(&old_project_dir)?;
    delete_project(&persisted_project)?;

    persisted_project.name = new_name.to_string();

    save_project(&persisted_project)?;

    Ok(())
}

pub fn delete_endpoint(
    project: &mut PersistedProject,
    endpoint: &PersistedEndpoint,
) -> anyhow::Result<()> {
    delete_project(project)?;

    let endpoints: Vec<PersistedEndpoint> = project
        .endpoints
        .iter()
        .filter(|pe| **pe != *endpoint)
        .cloned()
        .collect();

    project.endpoints = endpoints;

    save_project(project)?;

    Ok(())
}

pub fn delete_project(project: &PersistedProject) -> anyhow::Result<()> {
    let dir_result = get_app_dir("projects");
    if dir_result.is_err() {
        return Err(anyhow::Error::msg("Unable to access projects directory"));
    }

    let mut project_dir = dir_result.unwrap();
    project_dir.push(format!("{}.project", project.name));

    let remove_result = fs::remove_file(project_dir);
    if remove_result.is_err() {
        let write_error = remove_result.unwrap_err();

        return Err(anyhow::Error::msg(write_error.to_string()));
    }

    Ok(())
}

pub fn save_project(project: &PersistedProject) -> anyhow::Result<()> {
    if project.name.trim() == "" {
        return Err(anyhow::Error::msg("Project must have name"));
    }

    let serialization_result = serde_json::to_string(&project);

    if serialization_result.is_err() {
        return Err(anyhow::Error::msg("Unable to serialize project"));
    }

    let dir_result = get_app_dir("projects");
    if dir_result.is_err() {
        return Err(anyhow::Error::msg("Unable to access projects directory"));
    }

    let mut project_dir = dir_result.unwrap();
    let serialized_project = serialization_result.unwrap();
    project_dir.push(format!("{}.project", project.name));

    let write_result = fs::write(project_dir, serialized_project);
    if write_result.is_err() {
        let write_error = write_result.unwrap_err();

        return Err(anyhow::Error::msg(write_error.to_string()));
    }

    Ok(())
}

pub fn get_projects() -> anyhow::Result<Vec<PersistedProject>> {
    let dir_result = get_app_dir("projects");
    if dir_result.is_err() {
        return Err(anyhow::Error::msg("Unable to access projects directory"));
    }

    let project_dir = dir_result.unwrap();

    let read_dir = fs::read_dir(project_dir)?;

    Ok(read_dir
        .flatten()
        .flat_map(|entry| fs::read_to_string(entry.path()))
        .flat_map(|content| serde_json::from_str::<PersistedProject>(&content))
        .collect::<Vec<PersistedProject>>())
}

#[allow(unused)]
pub fn get_project_list() -> anyhow::Result<Value<List<Project>>> {
    match get_projects() {
        Ok(projects) => Ok(List::<Project>::from_iter(
            projects.iter().map(|project| project.into()),
        )),
        Err(error) => Err(error),
    }
}

impl From<&Endpoint> for PersistedEndpoint {
    fn from(endpoint: &Endpoint) -> Self {
        let mut headers: Vec<Header> = vec![];

        endpoint.headers.to_ref().iter().for_each(|header_state| {
            let h_state = header_state.to_ref();
            headers.push(h_state.deref().into());
        });

        PersistedEndpoint {
            name: endpoint.name.to_ref().to_string(),
            url: endpoint.url.to_ref().to_string(),
            method: endpoint.method.to_ref().to_string(),
            body: endpoint.body.to_ref().to_string(),
            body_mode: endpoint.body_mode.to_ref().to_string(),
            raw_type: endpoint.raw_type.to_ref().to_string(),
            headers,
        }
    }
}

impl From<&Project> for PersistedProject {
    fn from(project: &Project) -> Self {
        let mut endpoints: Vec<PersistedEndpoint> = vec![];
        project
            .endpoints
            .to_ref()
            .iter()
            .for_each(|endpoint_value| {
                let endpoint = endpoint_value.to_ref();
                endpoints.push(endpoint.deref().into());
            });

        let name = project.name.to_ref().clone();
        let variable = project
            .variable
            .to_ref()
            .iter()
            .map(|pv| PersistedVariable {
                id: Some(pv.to_ref().id.to_ref().to_string()),
                key: Some(pv.to_ref().key.to_ref().to_string()),
                value: Some(pv.to_ref().value.to_ref().to_string()),
                r#type: Some(match *pv.to_ref().r#type.to_ref() {
                    ProjectVariableType::String => VariableType::String,
                    ProjectVariableType::Boolean => VariableType::Boolean,
                    ProjectVariableType::Any => VariableType::Any,
                    ProjectVariableType::Number => VariableType::Number,
                }),
                name: Some(pv.to_ref().name.to_ref().to_string()),
                system: Some(*pv.to_ref().system.to_ref()),
                disabled: Some(*pv.to_ref().disabled.to_ref()),
                private: Some(pv.to_ref().private.to_ref().to_string()),
            })
            .collect();

        PersistedProject {
            name,
            endpoints,
            variable,
        }
    }
}

impl From<&HeaderState> for Header {
    fn from(header_state: &HeaderState) -> Self {
        Header {
            name: header_state.name.to_ref().to_string(),
            value: header_state.value.to_ref().to_string(),
        }
    }
}

impl From<&PersistedProject> for Project {
    fn from(persisted_project: &PersistedProject) -> Self {
        let endpoints: Value<List<Endpoint>> = List::from_iter(
            persisted_project
                .endpoints
                .iter()
                .map(|persisted_endpoint| persisted_endpoint.into()),
        );

        let variable = persisted_project
            .variable
            .iter()
            .map(|pv| ProjectVariable {
                id: pv.id.clone().unwrap_or_default().into(),
                key: pv.key.clone().unwrap_or_default().into(),
                value: pv.value.clone().unwrap_or_default().into(),
                r#type: match &pv.r#type {
                    Some(vt) => match vt {
                        VariableType::String => ProjectVariableType::String.into(),
                        VariableType::Boolean => ProjectVariableType::Boolean.into(),
                        VariableType::Any => ProjectVariableType::Any.into(),
                        VariableType::Number => ProjectVariableType::Number.into(),
                    },
                    None => ProjectVariableType::Any.into(),
                },
                name: pv.name.clone().unwrap_or_default().into(),
                system: pv.system.unwrap_or_default().into(),
                disabled: pv.disabled.unwrap_or_default().into(),
                private: pv.private.clone().unwrap_or_default().into(),
                row_color: "".to_string().into(),
                row_fg_color: "".to_string().into(),
            })
            .collect();

        Project {
            name: persisted_project.name.clone().into(),
            row_color: DEFAULT_ROW_COLOR.to_string().into(),
            row_fg_color: DEFAULT_ROW_COLOR.to_string().into(),
            endpoints,
            variable,
        }
    }
}

impl From<&PersistedEndpoint> for Endpoint {
    fn from(persisted_endpoint: &PersistedEndpoint) -> Self {
        let headers: Value<List<HeaderState>> = List::from_iter(
            persisted_endpoint
                .headers
                .iter()
                .map(|header| header.into()),
        );

        Endpoint {
            name: persisted_endpoint.name.clone().into(),
            body: persisted_endpoint.body.clone().into(),
            body_mode: persisted_endpoint.body_mode.clone().into(),
            raw_type: persisted_endpoint.raw_type.clone().into(),
            url: persisted_endpoint.url.clone().into(),
            method: persisted_endpoint.method.clone().into(),
            row_color: DEFAULT_ROW_COLOR.to_string().into(),
            row_fg_color: DEFAULT_ROW_COLOR.to_string().into(),
            headers,
        }
    }
}

impl From<&Header> for HeaderState {
    fn from(header: &Header) -> Self {
        HeaderState {
            name: header.name.clone().into(),
            value: header.value.clone().into(),
            row_color: "".to_string().into(),
            row_fg_color: "".to_string().into(),
        }
    }
}
