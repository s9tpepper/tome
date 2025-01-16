use anathema::{
    component::{ComponentId, KeyCode, KeyEvent},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, List, Value},
    widgets::Elements,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};
use std::{fs, ops::Deref};

use arboard::Clipboard;
use serde::{Deserialize, Serialize};

use crate::{
    fs::{get_documents_dir, save_response},
    messages::confirm_actions::ConfirmAction,
    projects::{delete_endpoint, delete_project, PersistedVariable},
    theme::get_app_theme,
};
use crate::{
    projects::{
        save_project, Endpoint, HeaderState, PersistedEndpoint, PersistedProject, Project,
        DEFAULT_ENDPOINT_NAME, DEFAULT_PROJECT_NAME,
    },
    theme::AppTheme,
};
use crate::{requests::do_request, theme::get_app_theme_persisted};

use super::{
    add_header_window::AddHeaderWindow,
    app_layout::AppLayoutMessages,
    edit_header_selector::EditHeaderSelector,
    edit_header_window::EditHeaderWindow,
    floating_windows::{
        code_gen::CodeGen,
        commands::Commands,
        edit_endpoint_name::{EditEndpointName, EditEndpointNameMessages},
        edit_project_name::{EditProjectName, EditProjectNameMessages},
        FloatingWindow,
    },
    method_selector::MethodSelector,
    project_window::ProjectWindow,
    send_message,
    syntax_highlighter::get_highlight_theme,
    textarea::TextAreaMessages,
    textinput::TextInputMessages,
};
use super::{
    confirm_action_window::ConfirmActionWindow,
    floating_windows::{
        add_project_variable::{AddProjectVariable, AddProjectVariableMessages},
        body_mode_selector::BodyModeSelector,
        endpoints_selector::{EndpointsSelector, EndpointsSelectorMessages},
        file_selector::FileSelector,
        project_variables::ProjectVariables,
    },
};

pub const DASHBOARD_TEMPLATE: &str = "./src/components/templates/dashboard.aml";

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DashboardDisplay {
    RequestBody,
    RequestHeadersEditor,
    ResponseBody,
    ResponseHeaders,
}

impl anathema::state::State for DashboardDisplay {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        match self {
            DashboardDisplay::RequestBody => Some(CommonVal::Str("request_body")),
            DashboardDisplay::RequestHeadersEditor => {
                Some(CommonVal::Str("request_headers_editor"))
            }
            DashboardDisplay::ResponseBody => Some(CommonVal::Str("response_body")),
            DashboardDisplay::ResponseHeaders => Some(CommonVal::Str("response_headers")),
        }
    }
}

#[derive(anathema::state::State)]
pub struct MenuItem {
    label: Value<String>,
    color: Value<String>,
}

#[derive(anathema::state::State)]
pub struct DashboardState {
    pub main_display: Value<DashboardDisplay>,
    pub floating_window: Value<FloatingWindow>,

    pub endpoint: Value<Endpoint>,
    pub response_headers: Value<List<HeaderState>>,
    pub response: Value<String>,
    pub response_body_window_label: Value<String>,

    pub error_message: Value<String>,
    pub message: Value<String>,
    pub message_label: Value<String>,
    pub menu_items: Value<List<MenuItem>>,
    pub top_menu_items: Value<List<MenuItem>>,
    pub logs: Value<String>,

    pub new_header_name: Value<String>,
    pub new_header_value: Value<String>,

    pub edit_header_name: Value<String>,
    pub edit_header_value: Value<String>,

    pub header_being_edited: Value<Option<Value<HeaderState>>>,

    pub project: Value<Project>,
    // pub project_count: Value<u8>,
    pub endpoint_count: Value<u8>,

    pub filter_indexes: Value<List<usize>>,
    pub filter_total: Value<usize>,
    pub filter_nav_index: Value<usize>,

    pub app_bg: Value<String>,
    pub app_theme: Value<AppTheme>,
}

impl DashboardState {
    pub fn new(app_theme: AppTheme) -> Self {
        let project = Project::new();

        // TODO: Re-do this in a way that doesn't suck
        let color: Value<String> = app_theme.menu_color_1.to_ref().to_string().into();
        let color1: Value<String> = app_theme.menu_color_1.to_ref().to_string().into();
        let color2: Value<String> = app_theme.menu_color_2.to_ref().to_string().into();
        let color3: Value<String> = app_theme.menu_color_3.to_ref().to_string().into();
        let color4: Value<String> = app_theme.menu_color_4.to_ref().to_string().into();
        let color5: Value<String> = app_theme.menu_color_5.to_ref().to_string().into();

        DashboardState {
            // project_count: 0.into(),
            project: project.into(),
            endpoint_count: 0.into(),
            endpoint: Endpoint::new().into(),

            response: "".to_string().into(),
            message: "".to_string().into(),
            message_label: "".to_string().into(),
            response_body_window_label: "".to_string().into(),
            error_message: "".to_string().into(),
            new_header_name: "".to_string().into(),
            new_header_value: "".to_string().into(),
            edit_header_name: "".to_string().into(),
            edit_header_value: "".to_string().into(),
            floating_window: FloatingWindow::None.into(),
            // main_display: Value::<DashboardDisplay>::new(DashboardDisplay::RequestBody),
            main_display: DashboardDisplay::RequestBody.into(),
            logs: "".to_string().into(),
            menu_items: List::from_iter([
                MenuItem {
                    color: color1,
                    label: "(S)ave Project".to_string().into(),
                },
                MenuItem {
                    color: color2,
                    label: "Save Endpo(i)nt".to_string().into(),
                },
                MenuItem {
                    color: color3,
                    label: "Swap (P)roject".to_string().into(),
                },
                MenuItem {
                    color: color4,
                    label: "Swap (E)ndpoint".to_string().into(),
                },
                MenuItem {
                    color: color5,
                    label: "(O)ptions".to_string().into(),
                },
            ]),
            top_menu_items: List::from_iter([MenuItem {
                color,
                label: "(P)rojects".to_string().into(),
            }]),
            response_headers: List::from_iter(vec![]),
            header_being_edited: None.into(),
            filter_indexes: List::empty(),
            filter_total: 0.into(),
            filter_nav_index: 0.into(),
            app_bg: "#000000".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

pub struct DashboardComponent {
    pub component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    test: bool,
}

impl DashboardComponent {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let theme = get_highlight_theme(None);

        let app_theme = get_app_theme();

        let mut state = DashboardState::new(app_theme);
        let color = theme.settings.background.unwrap();

        state
            .app_bg
            .set(format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b));

        let id = builder.register_component(
            "dashboard",
            DASHBOARD_TEMPLATE,
            DashboardComponent {
                component_ids: ids.clone(),
                test: false,
            },
            state,
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("dashboard"), id);

        Ok(())
    }

    pub fn show_message(&self, title: &str, message: &str, state: &mut DashboardState) {
        state.message.set(message.to_string());
        state.message_label.set(title.to_string());
        state.floating_window.set(FloatingWindow::Message);

        // TODO: Add same auto-close behavior as show_error()
    }

    pub fn show_error(&self, message: &str, state: &mut DashboardState) {
        state.error_message.set(message.to_string());
        state.floating_window.set(FloatingWindow::Error);

        // NOTE: Can not sleep thread, as it prevents anathema from displaying
        // the window change. This needs to be in it's own thread if I want to
        // close the window after a certain amount of time
        //
        // let close_delay = Duration::from_secs(5);
        // sleep(close_delay);
        // state.floating_window.set(FloatingWindow::None);
    }

    fn save_project(&self, state: &mut DashboardState, show_message: bool) {
        let project: PersistedProject = state.project.to_ref().deref().into();

        match save_project(&project) {
            Ok(_) => {
                if show_message {
                    self.show_message("Project Save", "Saved project successfully", state)
                }
            }
            Err(error) => self.show_error(&error.to_string(), state),
        }
    }

    fn send_options_open(&self, _: &mut DashboardState, context: Context<'_, DashboardState>) {
        let component_ids = self.component_ids.try_borrow();
        if component_ids.is_err() {
            return;
        }

        let component_ids = component_ids.unwrap();
        let Some(app_id) = component_ids.get("app") else {
            return;
        };

        let Ok(msg) = serde_json::to_string(&AppLayoutMessages::OpenOptions) else {
            return;
        };

        context.emit(*app_id, msg);
    }

    fn rename_variable(
        &self,
        value: &str,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
    ) {
        let Ok(variable) = serde_json::from_str::<PersistedVariable>(value) else {
            state.floating_window.set(FloatingWindow::None);
            context.set_focus("id", "app");

            return;
        };

        let current_names: Vec<String> = state
            .project
            .to_ref()
            .variable
            .to_ref()
            .iter()
            .map(|v| v.to_ref().name.to_ref().to_string())
            .collect();

        let current_project_name = state.project.to_ref().name.to_ref().clone();
        let add_project_variable_messages = AddProjectVariableMessages::Specifically((
            current_project_name,
            variable,
            current_names,
        ));
        let Ok(message) = serde_json::to_string(&add_project_variable_messages) else {
            return;
        };

        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };

        state
            .floating_window
            .set(FloatingWindow::AddProjectVariable);
        context.set_focus("id", "add_project_variable");

        let _ = send_message("add_project_variable", message, &ids, context.emitter);
    }

    fn rename_endpoint(
        &self,
        value: &str,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
    ) {
        let Ok(endpoint) = serde_json::from_str::<PersistedEndpoint>(value) else {
            state.floating_window.set(FloatingWindow::None);
            context.set_focus("id", "app");

            return;
        };

        let current_names: Vec<String> = state
            .project
            .to_ref()
            .endpoints
            .to_ref()
            .iter()
            .map(|e| e.to_ref().name.to_ref().to_string())
            .collect();

        let current_project_name = state.project.to_ref().name.to_ref().clone();
        let edit_endpoint_name_messages =
            EditEndpointNameMessages::Specifically((current_project_name, endpoint, current_names));
        let Ok(message) = serde_json::to_string(&edit_endpoint_name_messages) else {
            return;
        };

        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };

        state
            .floating_window
            .set(FloatingWindow::ChangeEndpointName);
        context.set_focus("id", "edit_endpoint_name");

        let _ = send_message("edit_endpoint_name", message, &ids, context.emitter);
    }

    fn rename_project(
        &self,
        value: &str,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
    ) {
        let Ok(project) = serde_json::from_str::<PersistedProject>(value) else {
            state.floating_window.set(FloatingWindow::None);
            context.set_focus("id", "app");

            return;
        };

        let edit_project_name_messages = EditProjectNameMessages::Specifically(project);
        let Ok(message) = serde_json::to_string(&edit_project_name_messages) else {
            return;
        };

        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };

        state.floating_window.set(FloatingWindow::ChangeProjectName);
        context.set_focus("id", "edit_project_name");

        let _ = send_message("edit_project_name", message, &ids, context.emitter);
    }

    fn new_project(&self, state: &mut DashboardState, context: &mut Context<'_, DashboardState>) {
        self.save_project(state, false);

        state.project.set(Project::new());
        state.endpoint.set(Endpoint::new());

        self.clear_url_and_request_body(context);

        state.floating_window.set(FloatingWindow::None);
        context.set_focus("id", "app");
    }

    fn new_endpoint(&self, state: &mut DashboardState, context: Context<'_, DashboardState>) {
        self.save_endpoint(state, &context, false);

        state.endpoint = Endpoint::new().into();
        self.clear_url_and_request_body(&context);
    }

    fn clear_url_and_request_body(&self, context: &Context<'_, DashboardState>) {
        if let Ok(component_ids) = self.component_ids.try_borrow() {
            let url = String::from("");
            let _ = send_message("url_text_input", url, &component_ids, context.emitter);

            let body = String::from("");
            let textarea_msg = TextAreaMessages::SetInput(body);
            if let Ok(message) = serde_json::to_string(&textarea_msg) {
                let _ = send_message(
                    "request_body_input",
                    message,
                    &component_ids,
                    context.emitter,
                );
            }
        };
    }

    fn save_endpoint(
        &self,
        state: &mut DashboardState,
        _: &Context<'_, DashboardState>,
        show_message: bool,
    ) {
        let project_name = state.project.to_ref().name.to_ref().to_string();
        let endpoint_name = state.endpoint.to_ref().name.to_ref().to_string();

        if endpoint_name == DEFAULT_ENDPOINT_NAME {
            self.show_error("Please give your endpoint a name to save", state);
            return;
        }

        if project_name == DEFAULT_PROJECT_NAME {
            self.show_error("Please give your project a name to save", state);
            return;
        }

        let mut project: PersistedProject = state.project.to_ref().deref().into();
        let existing_endpoint = project
            .endpoints
            .iter()
            .enumerate()
            .find(|(_, endpoint)| endpoint.name == endpoint_name);

        if let Some((index, _)) = existing_endpoint {
            project.endpoints.remove(index);
        }

        project
            .endpoints
            .push((&state.endpoint.to_ref().clone()).into());

        match save_project(&project) {
            Ok(_) => {
                if show_message {
                    self.show_message("Endpoint Save", "Saved endpoint successfully", state);
                }

                state.project.set((&project).into());
            }
            Err(error) => self.show_error(&error.to_string(), state),
        }
    }

    fn open_edit_project_name_window(
        &self,
        state: &mut DashboardState,
        mut context: Context<'_, DashboardState>,
    ) {
        state.floating_window.set(FloatingWindow::ChangeProjectName);

        context.set_focus("id", "edit_project_name");

        if let Ok(ids) = self.component_ids.try_borrow() {
            let mut input_value = state.project.to_ref().name.to_ref().clone();
            if input_value == DEFAULT_PROJECT_NAME {
                input_value = String::new();
            }

            let message = EditProjectNameMessages::InputValue(input_value);
            let _ = serde_json::to_string(&message).map(|msg| {
                let _ = send_message("edit_project_name", msg, &ids, context.emitter);
            });
        }
    }

    fn open_endpoints_selector(
        &self,
        state: &mut DashboardState,
        mut context: Context<'_, DashboardState>,
    ) {
        state.floating_window.set(FloatingWindow::EndpointsSelector);
        context.set_focus("id", "endpoints_selector_window");

        let persisted_endpoints: Vec<PersistedEndpoint> = state
            .project
            .to_ref()
            .endpoints
            .to_ref()
            .iter()
            .map(|endpoint| {
                let e = endpoint.to_ref();

                (&*e).into()
            })
            .collect();

        let msg = EndpointsSelectorMessages::EndpointsList(persisted_endpoints);
        #[allow(clippy::single_match)]
        match self.component_ids.try_borrow() {
            #[allow(clippy::single_match)]
            Ok(ids) => match ids.get("endpoints_selector_window") {
                Some(id) => {
                    let _ = serde_json::to_string(&msg).map(|payload| {
                        context.emit(*id, payload);
                    });
                }
                None => self.show_error("Unable to find endpoints window id", state),
            },
            Err(_) => self.show_error("Unable to find components id map", state),
        };
    }

    fn open_edit_endpoint_name_window(
        &self,
        state: &mut DashboardState,
        mut context: Context<'_, DashboardState>,
    ) {
        state
            .floating_window
            .set(FloatingWindow::ChangeEndpointName);
        context.set_focus("id", "edit_endpoint_name");

        if let Ok(ids) = self.component_ids.try_borrow() {
            let mut input_value = state.endpoint.to_ref().name.to_ref().clone();
            if input_value == DEFAULT_ENDPOINT_NAME {
                input_value = String::new();
            }

            let current_names: Vec<String> = state
                .project
                .to_ref()
                .endpoints
                .to_ref()
                .iter()
                .map(|e| e.to_ref().name.to_ref().clone())
                .collect();

            let message = EditEndpointNameMessages::InputValue((input_value, current_names));
            let _ = serde_json::to_string(&message).map(|msg| {
                let _ = send_message("edit_endpoint_name", msg, &ids, context.emitter);
            });
        }
    }

    fn yank_response(&self, state: &mut DashboardState) {
        let Ok(mut clipboard) = Clipboard::new() else {
            self.show_error("Error accessing your clipboard", state);

            return;
        };

        let operation_text = state.response.to_ref().clone();
        let set_operation = clipboard.set();
        match set_operation.text(operation_text) {
            Ok(_) => self.show_message("Clipboard", "Response copied to clipboard", state),
            Err(error) => self.show_error(&error.to_string(), state),
        }
    }

    fn open_commands_window(
        &self,
        state: &mut DashboardState,
        mut context: Context<'_, DashboardState>,
    ) {
        state.floating_window.set(FloatingWindow::Commands);
        context.set_focus("id", "commands_window");
    }

    fn open_body_mode_selector(
        &self,
        state: &mut DashboardState,
        mut context: Context<'_, DashboardState>,
    ) {
        state.floating_window.set(FloatingWindow::BodyModeSelector);
        context.set_focus("id", "body_mode_selector");
    }

    fn open_add_header_window(
        &self,
        state: &mut DashboardState,
        mut context: Context<'_, DashboardState>,
    ) {
        state.floating_window.set(FloatingWindow::AddHeader);
        context.set_focus("id", "add_header_window");

        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };

        let _ = send_message(
            "add_header_window",
            "open".to_string(),
            &ids,
            context.emitter,
        );
    }

    fn confirm_action(
        &self,
        confirm_action: ConfirmAction,
        state: &mut DashboardState,
        mut context: Context<'_, DashboardState>,
    ) {
        match confirm_action {
            ConfirmAction::ConfirmationDeletePersistedProject(delete_project_details_answer) => {
                match delete_project_details_answer.answer {
                    true => {
                        // TODO: Verify this line is correct, because this deletes the current
                        // project that's selected for the dashboard, and I believe that this
                        // should be deleting the project in delete_project_details_answer.project
                        //
                        let persisted_project: PersistedProject =
                            state.project.to_ref().as_ref().into();

                        state.project.set(Project::new());
                        state.endpoint.set(Endpoint::new());
                        self.clear_url_and_request_body(&context);

                        let project_name = persisted_project.name.clone();
                        let delete_result = delete_project(&persisted_project);
                        match delete_result {
                            Ok(_) => self.show_message(
                                "Delete Project Success",
                                &format!("Project '{project_name}' has been deleted"),
                                state,
                            ),

                            Err(_) => self.show_error(
                                &format!("There was an error deleting '{project_name}"),
                                state,
                            ),
                        }
                    }

                    false => {
                        state.floating_window.set(FloatingWindow::None);
                        context.set_focus("id", "app");
                    }
                }
            }

            ConfirmAction::ConfirmationDeletePersistedEndpoint(delete_endpoint_details_answer) => {
                match delete_endpoint_details_answer.answer {
                    true => {
                        let persisted_endpoint = delete_endpoint_details_answer.data;
                        let current_endpoint: PersistedEndpoint =
                            state.endpoint.to_ref().deref().into();

                        if persisted_endpoint == current_endpoint {
                            state.endpoint.set(Endpoint::new());
                            self.clear_url_and_request_body(&context);
                        }

                        let endpoint_name = persisted_endpoint.name.clone();
                        let mut current_project: PersistedProject =
                            state.project.to_ref().deref().into();
                        match delete_endpoint(&mut current_project, &persisted_endpoint) {
                            Ok(_) => {
                                let project: Project = (&current_project).into();
                                state.project.set(project);

                                self.show_message(
                                    "Delete Endpoint Success",
                                    &format!("The {endpoint_name} endpoint has been deleted."),
                                    state,
                                );
                            }
                            Err(_) => self.show_error(
                                &format!(
                                    "There was an error deleting the {endpoint_name} endpoint"
                                ),
                                state,
                            ),
                        }
                    }

                    false => {
                        state.floating_window.set(FloatingWindow::None);
                        context.set_focus("id", "app");
                    }
                }
            }

            ConfirmAction::ConfirmationDeletePersistedVariable(
                delete_persisted_variable_answer,
            ) => match delete_persisted_variable_answer.answer {
                true => {
                    let persisted_variable = delete_persisted_variable_answer.data;
                    let deleted_variable = persisted_variable.name.clone().unwrap_or_default();

                    let mut delete_index: i64 = -1;
                    for (index, var) in state.project.to_ref().variable.to_ref().iter().enumerate()
                    {
                        let project_persisted_variable: PersistedVariable =
                            (var.to_ref().deref()).into();

                        if project_persisted_variable == persisted_variable {
                            delete_index = index as i64;
                            break;
                        }
                    }

                    if delete_index > -1 {
                        state
                            .project
                            .to_mut()
                            .variable
                            .remove(delete_index as usize);

                        let project: PersistedProject = state.project.to_ref().deref().into();
                        match save_project(&project) {
                            Ok(_) => {
                                let title = format!("Delete '{deleted_variable}'");
                                let message = "Variable deleted successfully";
                                self.show_message(&title, message, state);
                            }
                            Err(_) => {
                                let message = format!("Error deleting '{deleted_variable}'");
                                self.show_error(&message, state);
                            }
                        }
                    }
                }

                false => {
                    state.floating_window.set(FloatingWindow::None);
                    context.set_focus("id", "app");
                }
            },

            _ => {}
        }
    }
}

pub trait DashboardMessageHandler {
    fn handle_message(
        value: CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut DashboardState,
        context: Context<'_, DashboardState>,
        elements: Elements<'_, '_>,
        component_ids: Ref<'_, HashMap<String, ComponentId<String>>>,
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DashboardMessages {
    TextInput(TextInputMessages),
    TextArea(TextAreaMessages),
    ThemeUpdate,
    ShowSucces((String, String)),
    ShowError(String),
    Confirmations(ConfirmAction),
}

impl anathema::component::Component for DashboardComponent {
    type State = DashboardState;
    type Message = String;

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        if let Ok(dashboard_message) = serde_json::from_str::<DashboardMessages>(&message) {
            match dashboard_message {
                DashboardMessages::Confirmations(confirm_action) => {
                    self.confirm_action(confirm_action, state, context);
                }

                DashboardMessages::ShowSucces((title, message)) => {
                    self.show_message(&title, &message, state);
                }

                DashboardMessages::ShowError(message) => {
                    self.show_error(&message, state);
                }

                DashboardMessages::ThemeUpdate => {
                    // TODO: Use this message again when the state update bug is fixed in anathema
                    // println!("Changing dashboard theme");
                    // self.update_app_theme(state);

                    // let app_theme = get_app_theme_persisted();
                    // state.app_theme.set(app_theme.into());
                }

                DashboardMessages::TextInput(text_input_message) => match text_input_message {
                    // TODO: Refactor this message to not be 100% coupled to only editing the
                    // endpoint name
                    TextInputMessages::Change(value) => {
                        let mut endpoint = state.endpoint.to_mut();
                        let name_still_default = *endpoint.url.to_ref() == *endpoint.name.to_ref();

                        endpoint.url.set(value.to_string());

                        if name_still_default {
                            endpoint.name.set(value.to_string());
                        }
                    }

                    #[allow(clippy::single_match)]
                    TextInputMessages::Update(text_update) => match text_update.id.as_str() {
                        "endpoint_url_input" => {
                            state.endpoint.to_mut().url.set(text_update.value);
                        }

                        _ => {}
                    },

                    #[allow(clippy::single_match)]
                    TextInputMessages::Escape(text_update) => match text_update.id.as_str() {
                        "endpoint_url_input" => {
                            context.set_focus("id", "app");

                            if let Ok(ids) = self.component_ids.try_borrow() {
                                let _ = send_message(
                                    "url_input",
                                    "unfocus".to_string(),
                                    &ids,
                                    context.emitter,
                                );
                            }
                        }

                        _ => {}
                    },
                },

                // TODO: Refactor this message to not be 100% coupled to only editing the
                // endpoint body
                DashboardMessages::TextArea(text_area_message) => match text_area_message {
                    TextAreaMessages::InputChange(value) => {
                        state.endpoint.to_mut().body.set(value);
                    }

                    // NOTE: SetInput is only used for sending the TextArea a new value
                    TextAreaMessages::SetInput(_) => todo!(),
                },
            }
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: CommonVal<'_>,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match ident {
            // Unfocus the url input and set back to dashboard
            "url_input_focus" => {
                context.set_focus("id", "app");
            }

            "add_new_project" => {
                self.new_project(state, &mut context);
            }

            "rename_project" => {
                self.rename_project(&value.to_string(), state, &mut context);
            }

            "rename_endpoint" => {
                self.rename_endpoint(&value.to_string(), state, &mut context);
            }

            "rename_variable" => {
                self.rename_variable(&value.to_string(), state, &mut context);
            }

            "open_add_variable_window" => {
                state
                    .floating_window
                    .set(FloatingWindow::AddProjectVariable);
                context.set_focus("id", "add_project_variable");

                let Ok(message) = serde_json::to_string(&AddProjectVariableMessages::InitialFocus)
                else {
                    return;
                };

                let Ok(ids) = self.component_ids.try_borrow() else {
                    return;
                };

                let _ = send_message("add_project_variable", message, &ids, context.emitter);
            }

            _ => {}
        }

        let (component, _event) = ident.split_once("__").unwrap_or(("", ""));

        if let Ok(component_ids) = self.component_ids.try_borrow() {
            match component {
                "confirm_action" => ConfirmActionWindow::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                ),

                "project_variables" => ProjectVariables::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                ),

                "add_project_variable" => AddProjectVariable::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                ),

                "body_mode_selector" => BodyModeSelector::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                ),

                "file_selector" => {
                    FileSelector::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                "commands" => {
                    Commands::handle_message(value, ident, state, context, elements, component_ids);
                }

                "codegen" => {
                    CodeGen::handle_message(value, ident, state, context, elements, component_ids);
                }

                "add_header" => {
                    AddHeaderWindow::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                "edit_header" => {
                    EditHeaderWindow::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                "edit_header_selector" => {
                    EditHeaderSelector::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                "method_selector" => {
                    MethodSelector::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                "project_window" => {
                    ProjectWindow::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                "edit_endpoint_name" => {
                    EditEndpointName::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                "edit_project_name" => {
                    EditProjectName::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                "endpoints_selector" => {
                    EndpointsSelector::handle_message(
                        value,
                        ident,
                        state,
                        context,
                        elements,
                        component_ids,
                    );
                }

                _ => {}
            }
        } else {
            println!("Could not find id for {ident}");
        }
    }

    fn on_key(
        &mut self,
        event: KeyEvent,
        state: &mut Self::State,
        elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match event.code {
            KeyCode::Char(char) => {
                let main_display = *state.main_display.to_ref();

                match char {
                    'c' => self.open_commands_window(state, context),
                    's' => self.save_project(state, true),
                    'n' => self.open_edit_endpoint_name_window(state, context),
                    'j' => self.open_edit_project_name_window(state, context),
                    'i' => self.save_endpoint(state, &context, true),
                    // 'f' => context.set_focus("id", "response_body_input"),
                    'o' => self.send_options_open(state, context),
                    't' => self.new_endpoint(state, context),
                    'w' => self.new_project(state, &mut context),

                    'v' => match main_display {
                        DashboardDisplay::RequestBody => {}
                        DashboardDisplay::RequestHeadersEditor => {}
                        DashboardDisplay::ResponseBody => save_response(self, state, context),
                        DashboardDisplay::ResponseHeaders => {}
                    },

                    // Set focus to the request url text input
                    'u' => {
                        if !event.ctrl {
                            context.set_focus("id", "url_input");
                        }
                    }

                    // Quit app
                    'q' => quit::with_code(0),

                    // Make the request
                    'r' => {
                        let response = do_request(state, context, elements, self);
                        match response {
                            Ok(_) => {}
                            Err(err) => {
                                self.show_error(&err.to_string(), state);
                            }
                        }
                    }

                    // Show request body editor window
                    'b' => match main_display {
                        DashboardDisplay::RequestBody => context.set_focus("id", "textarea"),
                        DashboardDisplay::RequestHeadersEditor => {
                            state.main_display.set(DashboardDisplay::RequestBody);
                        }
                        DashboardDisplay::ResponseBody => {
                            // NOTE: Maybe revert this, needs testing to check focus UX
                            // state.main_display.set(DashboardDisplay::RequestBody);
                            context.set_focus("id", "response_renderer");
                        }
                        DashboardDisplay::ResponseHeaders => {
                            state.main_display.set(DashboardDisplay::ResponseBody)
                        }
                    },

                    // Show request headers editor window
                    'd' => {
                        if !event.ctrl {
                            state
                                .main_display
                                .set(DashboardDisplay::RequestHeadersEditor);
                        }
                    }

                    // Open Endpoints selector
                    'e' => {
                        self.open_endpoints_selector(state, context);
                    }

                    // Show projects window
                    'p' => {
                        if let Ok(component_ids) = self.component_ids.try_borrow() {
                            state.floating_window.set(FloatingWindow::Project);
                            context.set_focus("id", "project_selector");

                            let _ = component_ids.get("project_selector").map(|id| {
                                context.emit(*id, "projects".to_string());
                            });
                        }
                    }

                    // Show response headers display
                    'h' => match main_display {
                        DashboardDisplay::RequestBody => {}
                        DashboardDisplay::RequestHeadersEditor => {
                            state
                                .floating_window
                                .set(FloatingWindow::EditHeaderSelector);
                            context.set_focus("id", "edit_header_selector");
                        }
                        DashboardDisplay::ResponseBody => {
                            state.main_display.set(DashboardDisplay::ResponseHeaders)
                        }
                        DashboardDisplay::ResponseHeaders => {}
                    },

                    // Open Request Method selection window
                    'm' => {
                        state.floating_window.set(FloatingWindow::Method);
                        context.set_focus("id", "method_selector");
                    }

                    'a' => match main_display {
                        DashboardDisplay::RequestBody => {}
                        DashboardDisplay::RequestHeadersEditor => {
                            // Open header window
                            self.open_add_header_window(state, context);
                        }
                        DashboardDisplay::ResponseBody => {}
                        DashboardDisplay::ResponseHeaders => {}
                    },

                    'y' => match main_display {
                        DashboardDisplay::RequestBody => {
                            self.open_body_mode_selector(state, context)
                        }
                        DashboardDisplay::RequestHeadersEditor => {}
                        DashboardDisplay::ResponseBody => {
                            // Copy response body to clipboard
                            self.yank_response(state)
                        }
                        DashboardDisplay::ResponseHeaders => {}
                    },

                    _ => {}
                }
            }

            KeyCode::Esc => {
                context.set_focus("id", "app");

                if *state.floating_window.to_ref() != FloatingWindow::None {
                    state.floating_window.set(FloatingWindow::None);
                // NOTE: Test the UX of the focus movement with this Esc binding
                } else if *state.main_display.to_ref() != DashboardDisplay::RequestBody {
                    state.main_display.set(DashboardDisplay::RequestBody);
                }
            }

            KeyCode::Enter => {
                // TODO: Do something with the Enter button
            }

            _ => {}
        }
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        update_theme(state);

        if self.test {
            return;
        }

        // TODO: REMOVE THIS - ONLY FOR TESTING
        if let Ok(ids) = self.component_ids.try_borrow() {
            let _ = send_message(
                "url_text_input",
                "https://jsonplaceholder.typicode.com/todos".to_string(),
                &ids,
                context.emitter,
            );

            self.test = true;
        }
    }

    fn accept_focus(&self) -> bool {
        true
    }
}

fn update_theme(state: &mut DashboardState) {
    let app_theme = get_app_theme_persisted();

    // TODO: Figure out why this breaks the styling and messaging of the dashboard components
    // println!("{app_theme:?}");
    // state.app_theme.set(app_theme.into());
    // state.app_theme.set(app_theme.into());

    let mut at = state.app_theme.to_mut();
    at.background.set(app_theme.background);
    at.foreground.set(app_theme.foreground);
    at.project_name_background
        .set(app_theme.project_name_background);
    at.project_name_foreground
        .set(app_theme.project_name_foreground);
    at.border_focused.set(app_theme.border_focused);
    at.border_unfocused.set(app_theme.border_unfocused);
    at.overlay_heading.set(app_theme.overlay_heading);
    at.overlay_background.set(app_theme.overlay_background);
    at.overlay_foreground.set(app_theme.overlay_foreground);
    at.overlay_submit_background
        .set(app_theme.overlay_submit_background);
    at.overlay_submit_foreground
        .set(app_theme.overlay_submit_foreground);

    at.overlay_cancel_background
        .set(app_theme.overlay_cancel_background);
    at.overlay_cancel_foreground
        .set(app_theme.overlay_cancel_foreground);
    at.menu_color_1.set(app_theme.menu_color_1);
    at.menu_color_2.set(app_theme.menu_color_2);
    at.menu_color_3.set(app_theme.menu_color_3);
    at.menu_color_4.set(app_theme.menu_color_4);
    at.menu_color_5.set(app_theme.menu_color_5);

    at.endpoint_name_background
        .set(app_theme.endpoint_name_background);
    at.endpoint_name_foreground
        .set(app_theme.endpoint_name_foreground);
    at.menu_opt_background.set(app_theme.menu_opt_background);
    at.menu_opt_foreground.set(app_theme.menu_opt_foreground);
    at.top_bar_background.set(app_theme.top_bar_background);
    at.top_bar_foreground.set(app_theme.top_bar_foreground);
    at.bottom_bar_background
        .set(app_theme.bottom_bar_background);
    at.bottom_bar_foreground
        .set(app_theme.bottom_bar_foreground);

    // *state.app_theme.to_mut() = app_theme.into();
}
