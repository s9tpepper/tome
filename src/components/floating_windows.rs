use anathema::state::{CommonVal, State};

pub mod add_project_variable;
pub mod app_theme_selector;
pub mod body_mode_selector;
pub mod button_style_selector;
pub mod code_gen;
pub mod commands;
pub mod edit_endpoint_name;
pub mod edit_project_name;
pub mod endpoints_selector;
pub mod file_selector;
pub mod project_variables;
pub mod syntax_theme_selector;

#[derive(PartialEq, Eq)]
pub enum FloatingWindow {
    None,
    Method,
    AddHeader,
    Error,
    EditHeaderSelector,
    Project,
    ConfirmAction,
    Message,
    ChangeEndpointName,
    ChangeProjectName,
    EndpointsSelector,
    Commands,
    CodeGen,
    PostmanFileSelector,
    BodyModeSelector,
    AddProjectVariable,
    ViewProjectVariables,
}

impl State for FloatingWindow {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        match self {
            FloatingWindow::None => Some(CommonVal::Str("None")),
            FloatingWindow::Method => Some(CommonVal::Str("Method")),
            FloatingWindow::AddHeader => Some(CommonVal::Str("AddHeader")),
            FloatingWindow::Error => Some(CommonVal::Str("Error")),
            FloatingWindow::EditHeaderSelector => Some(CommonVal::Str("EditHeaderSelector")),
            FloatingWindow::Project => Some(CommonVal::Str("Project")),
            FloatingWindow::ConfirmAction => Some(CommonVal::Str("ConfirmAction")),
            FloatingWindow::Message => Some(CommonVal::Str("Message")),
            FloatingWindow::ChangeEndpointName => Some(CommonVal::Str("ChangeEndpointName")),
            FloatingWindow::ChangeProjectName => Some(CommonVal::Str("ChangeProjectName")),
            FloatingWindow::EndpointsSelector => Some(CommonVal::Str("EndpointsSelector")),
            FloatingWindow::Commands => Some(CommonVal::Str("Commands")),
            FloatingWindow::CodeGen => Some(CommonVal::Str("CodeGen")),
            FloatingWindow::PostmanFileSelector => Some(CommonVal::Str("PostmanFileSelector")),
            FloatingWindow::BodyModeSelector => Some(CommonVal::Str("BodyModeSelector")),
            FloatingWindow::AddProjectVariable => Some(CommonVal::Str("AddProjectVariable")),
            FloatingWindow::ViewProjectVariables => Some(CommonVal::Str("ViewProjectVariables")),
        }
    }
}
