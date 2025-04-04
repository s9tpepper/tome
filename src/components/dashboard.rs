use action_confirmations::handle_confirmations;
use anathema::{
    component::{ComponentId, KeyEvent},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, List, State, Value},
    widgets::Elements,
};
use associated_functions::associated_functions;
use keyboard_events::keyboard_events;
use log::info;
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};
use std::{ops::Deref, sync::Arc};

use arboard::Clipboard;
use serde::{Deserialize, Serialize};

use crate::{
    app::GlobalEventHandler,
    fs::save_response,
    messages::confirm_actions::ConfirmAction,
    options::get_button_caps,
    projects::{Header, PersistedVariable},
    requests::do_request,
    templates::template,
    theme::{get_app_theme, update_component_theme},
};
use crate::{
    projects::{
        save_project, Endpoint, HeaderState, PersistedEndpoint, PersistedProject, Project,
        DEFAULT_ENDPOINT_NAME, DEFAULT_PROJECT_NAME,
    },
    theme::AppTheme,
};

use super::{
    app_layout::AppLayoutMessages,
    floating_windows::{
        edit_endpoint_name::EditEndpointNameMessages, edit_project_name::EditProjectNameMessages,
        FloatingWindow,
    },
    send_message,
    syntax_highlighter::get_highlight_theme,
    textarea::TextAreaMessages,
    textinput::TextInputMessages,
};
use super::{
    edit_header_selector::EditHeaderSelectorMessages,
    floating_windows::{
        add_project_variable::AddProjectVariableMessages,
        endpoints_selector::EndpointsSelectorMessages,
    },
};

mod action_confirmations;
mod associated_functions;
mod component;
mod keyboard_events;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DashboardDisplay {
    RequestBody,
    RequestHeadersEditor,
    ResponseBody,
    ResponseHeaders,
}

impl State for DashboardDisplay {
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

#[derive(State)]
pub struct MenuItem {
    label: Value<String>,
    color: Value<String>,
}

#[derive(State)]
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

    pub project: Value<Project>,
    // pub project_count: Value<u8>,
    pub endpoint_count: Value<u8>,

    pub filter_indexes: Value<List<usize>>,
    pub filter_total: Value<usize>,
    pub filter_nav_index: Value<usize>,

    pub app_bg: Value<String>,
    pub app_theme: Value<AppTheme>,
    pub button_cap_left: Value<String>,
    pub button_cap_right: Value<String>,
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

        let button_caps = get_button_caps();

        DashboardState {
            button_cap_left: button_caps.0.to_string().into(),
            button_cap_right: button_caps.1.to_string().into(),

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
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
    ) -> anyhow::Result<()> {
        let theme = get_highlight_theme(None);

        let app_theme = get_app_theme();

        let mut state = DashboardState::new(app_theme);
        let color = theme.settings.background.unwrap();

        state
            .app_bg
            .set(format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b));

        let dashboard = DashboardComponent {
            component_ids: ids.clone(),
            test: false,
        };

        let id = builder.register_component(
            "dashboard",
            template("templates/dashboard"),
            dashboard,
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
        info!("{message}");

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

    fn send_options_open(&self, context: &mut Context<'_, DashboardState>) {
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

    fn focus_url_input(&self, context: &mut RefCell<Context<'_, DashboardState>>, ctrl: bool) {
        if !ctrl {
            context.borrow_mut().set_focus("id", "url_input");
        }
    }

    fn show_request_headers(
        &self,
        event: Option<KeyEvent>,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
    ) {
        let ctrl_pressed = event.map_or_else(|| false, |event| event.ctrl);

        info!("show_request_headers():: ctrl_pressed: {ctrl_pressed}");

        if ctrl_pressed {
            return;
        }

        state
            .main_display
            .set(DashboardDisplay::RequestHeadersEditor);

        context.set_focus("id", "request_headers_editor");

        info!(
            "Updated state.main_display to {:?}",
            *state.main_display.to_ref()
        );
    }

    fn send_request(
        &mut self,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
        elements: &Elements<'_, '_>,
    ) {
        let response = do_request(state, context, elements, self);
        match response {
            Ok(_) => {}
            Err(err) => {
                self.show_error(&err.to_string(), state);
            }
        }

        let main_display = *state.main_display.to_ref();
        info!("send_request(): main_display: {main_display:?}");
    }

    fn send_save_response(&self, state: &mut DashboardState) {
        info!("send_save_response()");

        let main_display = *state.main_display.to_ref();
        match main_display {
            DashboardDisplay::RequestBody => {}
            DashboardDisplay::RequestHeadersEditor => {}
            DashboardDisplay::ResponseBody => save_response(self, state),
            DashboardDisplay::ResponseHeaders => {}
        }
    }

    fn handle_y_press(
        &self,
        state: &mut DashboardState,
        context: &mut RefCell<Context<'_, DashboardState>>,
    ) {
        let main_display = *state.main_display.to_ref();
        info!("handle_y_press():: main_display: {main_display:?}");

        match main_display {
            DashboardDisplay::RequestBody => self.open_body_mode_selector(state, context),
            DashboardDisplay::RequestHeadersEditor => {}
            DashboardDisplay::ResponseBody => {
                // Copy response body to clipboard
                self.yank_response(state)
            }
            DashboardDisplay::ResponseHeaders => {}
        }
    }

    fn go_to_headers(&self, state: &mut DashboardState, context: &mut Context<'_, DashboardState>) {
        let main_display = *state.main_display.to_ref();

        info!("go_to_headers() :: main_display {main_display:?}");
        match main_display {
            DashboardDisplay::RequestBody => {}
            DashboardDisplay::RequestHeadersEditor => self.open_edit_header_window(state, context),
            DashboardDisplay::ResponseBody => {
                state.main_display.set(DashboardDisplay::ResponseHeaders)
            }
            DashboardDisplay::ResponseHeaders => {}
        }
    }

    fn go_back(&self, state: &mut DashboardState, context: &mut Context<'_, DashboardState>) {
        let main_display = *state.main_display.to_ref();
        match main_display {
            DashboardDisplay::RequestBody => context.set_focus("id", "textarea"),
            DashboardDisplay::RequestHeadersEditor => {
                state.main_display.set(DashboardDisplay::RequestBody);
            }
            DashboardDisplay::ResponseBody => {
                state.main_display.set(DashboardDisplay::RequestBody);
                context.set_focus("id", "app");
            }
            DashboardDisplay::ResponseHeaders => {
                state.main_display.set(DashboardDisplay::ResponseBody)
            }
        }
    }

    fn new_project(&self, state: &mut DashboardState, context: &mut Context<'_, DashboardState>) {
        self.save_project(state, false);

        state.project.to_mut().reset();
        state.endpoint.to_mut().reset();

        self.clear_url_and_request_body(context);

        state.floating_window.set(FloatingWindow::None);
        context.set_focus("id", "app");
    }

    fn new_endpoint(&self, state: &mut DashboardState, context: &mut Context<'_, DashboardState>) {
        self.save_endpoint(state, false);

        state.endpoint = Endpoint::new().into();
        self.clear_url_and_request_body(context);
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

    fn save_endpoint(&self, state: &mut DashboardState, show_message: bool) {
        info!("dashboard.rs::save_endpoint()");
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

                info!("save_endpoint():: saved project data: {project:?}");

                let mut state_project = state.project.to_mut();
                state_project.update_from_persisted(&project);
            }
            Err(error) => self.show_error(&error.to_string(), state),
        }
    }

    fn open_edit_project_name_window(
        &self,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
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

    fn open_edit_header_window(
        &self,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
    ) {
        state
            .floating_window
            .set(FloatingWindow::EditHeaderSelector);
        context.set_focus("id", "edit_header_selector");

        let headers: Vec<Header> = state
            .endpoint
            .to_ref()
            .headers
            .to_ref()
            .iter()
            .map(|header_state| (&*header_state.to_ref()).into())
            .collect();

        let edit_header_selector_messages = EditHeaderSelectorMessages::HeadersList(headers);

        let Ok(message) = serde_json::to_string(&edit_header_selector_messages) else {
            return;
        };

        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };

        let _ = send_message("edit_header_selector", message, &ids, context.emitter);
    }

    fn open_projects_window(
        &self,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
    ) {
        if let Ok(component_ids) = self.component_ids.try_borrow() {
            state.floating_window.set(FloatingWindow::Project);
            context.set_focus("id", "project_selector");

            let _ = component_ids.get("project_selector").map(|id| {
                context.emit(*id, "projects".to_string());
            });
        }
    }

    fn open_endpoints_selector(
        &self,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
    ) {
        info!("dashboard.rs :: open_endpoints_selector()");

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

        info!("persisted_endpoints: {persisted_endpoints:?}");

        let msg = EndpointsSelectorMessages::EndpointsList(persisted_endpoints);
        info!("msg: {msg:?}");

        #[allow(clippy::single_match)]
        match self.component_ids.try_borrow() {
            #[allow(clippy::single_match)]
            Ok(ids) => match ids.get("endpoints_selector_window") {
                Some(id) => {
                    info!("Sending message to id: {id:?}");

                    let message_result = serde_json::to_string(&msg).map(|payload| {
                        info!("Sent endpoints_selector_window a message: {payload:?}");
                        context.emit(*id, payload);
                    });

                    info!("send endoints_selector_window message result: {message_result:?}");
                }
                None => self.show_error("Unable to find endpoints window id", state),
            },
            Err(_) => self.show_error("Unable to find components id map", state),
        };
    }

    fn open_edit_endpoint_name_window(
        &self,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
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
        context: &mut Context<'_, DashboardState>,
    ) {
        state.floating_window.set(FloatingWindow::Commands);
        context.set_focus("id", "commands_window");
    }

    fn open_body_mode_selector(
        &self,
        state: &mut DashboardState,
        context: &mut RefCell<Context<'_, DashboardState>>,
    ) {
        state.floating_window.set(FloatingWindow::BodyModeSelector);
        context.borrow_mut().set_focus("id", "body_mode_selector");
    }

    fn open_add_header_window(
        &self,
        state: &mut DashboardState,
        context: &mut Context<'_, DashboardState>,
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
        context: Context<'_, DashboardState>,
    ) {
        handle_confirmations(self, confirm_action, state, context);
    }

    fn add_test_url(&mut self, context: Context<'_, DashboardState>) {
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum KeebState {
    Press,
    Repeat,
    Release,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct KeebEvent {
    pub character: char,
    pub ctrl: bool,
    pub state: KeebState,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DashboardMessages {
    TextInput(TextInputMessages),
    TextArea(TextAreaMessages),
    ThemeUpdate,
    ButtonStyleUpdate,
    ShowSucces((String, String)),
    ShowError(String),
    Confirmations(ConfirmAction),
    BackToRequest,
    KeyboardEvent(KeebEvent),
}

fn update_theme(state: &mut DashboardState) {
    // TODO: Figure out why this breaks the styling and messaging of the dashboard components
    // state.app_theme.set(app_theme.into());

    update_component_theme(&mut state.app_theme);
}
