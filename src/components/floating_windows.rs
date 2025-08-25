use anathema::state::State;

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
    fn type_info(&self) -> anathema::state::Type {
        anathema::state::Type::String
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            FloatingWindow::None => Some("None"),
            FloatingWindow::Method => Some("Method"),
            FloatingWindow::AddHeader => Some("AddHeader"),
            FloatingWindow::Error => Some("Error"),
            FloatingWindow::EditHeaderSelector => Some("EditHeaderSelector"),
            FloatingWindow::Project => Some("Project"),
            FloatingWindow::ConfirmAction => Some("ConfirmAction"),
            FloatingWindow::Message => Some("Message"),
            FloatingWindow::ChangeEndpointName => Some("ChangeEndpointName"),
            FloatingWindow::ChangeProjectName => Some("ChangeProjectName"),
            FloatingWindow::EndpointsSelector => Some("EndpointsSelector"),
            FloatingWindow::Commands => Some("Commands"),
            FloatingWindow::CodeGen => Some("CodeGen"),
            FloatingWindow::PostmanFileSelector => Some("PostmanFileSelector"),
            FloatingWindow::BodyModeSelector => Some("BodyModeSelector"),
            FloatingWindow::AddProjectVariable => Some("AddProjectVariable"),
            FloatingWindow::ViewProjectVariables => Some("ViewProjectVariables"),
        }
    }
}
