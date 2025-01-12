use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{Component, ComponentId},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{List, State, Value},
    widgets::Elements,
};

use crate::{
    components::{
        dashboard::{DashboardMessageHandler, DashboardState, FloatingWindow},
        send_message,
        textarea::TextAreaMessages,
    },
    messages::confirm_delete_project::ConfirmDeleteProject,
    projects::{
        get_projects, Endpoint, PersistedProject, PersistedVariable, Project, ProjectVariable,
    },
    theme::{get_app_theme, AppTheme},
};

pub const PROJECT_VARIABLES_TEMPLATE: &str =
    "./src/components/floating_windows/templates/project_variables.aml";

#[derive(Default, State)]
pub struct ProjectVariablesState {
    cursor: Value<u8>,
    current_first_index: Value<u8>,
    current_last_index: Value<u8>,
    visible_projects: Value<u8>,
    window_list: Value<List<ProjectVariable>>,
    project_count: Value<u8>,
    // TODO: Figure out a better way to do this for deleting
    selected_project: Value<String>,
    app_theme: Value<AppTheme>,
}

impl ProjectVariablesState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        ProjectVariablesState {
            cursor: 0.into(),
            project_count: 0.into(),
            current_first_index: 0.into(),
            current_last_index: 4.into(),
            visible_projects: 5.into(),
            window_list: List::empty(),
            selected_project: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

#[derive(Default)]
pub struct ProjectVariables {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    variables_list: Vec<PersistedVariable>,
}

impl ProjectVariables {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let id = builder.register_component(
            "project_variables",
            PROJECT_VARIABLES_TEMPLATE,
            ProjectVariables::new(ids.clone()),
            ProjectVariablesState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("project_variables"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut ProjectVariablesState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        ProjectVariables {
            component_ids,
            variables_list: vec![],
        }
    }

    fn move_cursor_down(&self, state: &mut ProjectVariablesState) {
        let last_complete_list_index = self.variables_list.len().saturating_sub(1);
        let new_cursor = min(*state.cursor.to_ref() + 1, last_complete_list_index as u8);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor > last_index {
            last_index = new_cursor;
            first_index = new_cursor - (*state.visible_projects.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_project_list(
            first_index.into(),
            last_index.into(),
            new_cursor.into(),
            state,
        );
    }

    fn move_cursor_up(&self, state: &mut ProjectVariablesState) {
        let new_cursor = max(state.cursor.to_ref().saturating_sub(1), 0);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor < first_index {
            first_index = new_cursor;
            last_index = new_cursor + (*state.visible_projects.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_project_list(
            first_index.into(),
            last_index.into(),
            new_cursor.into(),
            state,
        );
    }

    fn update_project_list(
        &self,
        first_index: usize,
        last_index: usize,
        selected_index: usize,
        state: &mut ProjectVariablesState,
    ) {
        if self.variables_list.is_empty() {
            return;
        }

        let display_projects = &self.variables_list[first_index..=last_index];
        let mut new_project_list: Vec<ProjectVariable> = vec![];
        display_projects
            .iter()
            .for_each(|display_project_variable| {
                new_project_list.push(display_project_variable.clone().into());
            });

        loop {
            if state.window_list.len() > 0 {
                state.window_list.pop_front();
            } else {
                break;
            }
        }

        new_project_list
            .into_iter()
            .enumerate()
            .for_each(|(index, mut project_variable)| {
                let visible_index = selected_index.saturating_sub(first_index);
                if index == visible_index {
                    project_variable.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                    project_variable.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                } else {
                    project_variable.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                    project_variable.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                }

                state.window_list.push(project_variable);
            });
    }
}

impl DashboardMessageHandler for ProjectVariables {
    fn handle_message(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut DashboardState,
        mut context: anathema::prelude::Context<'_, DashboardState>,
        _: Elements<'_, '_>,
        _component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();

        match event.as_str() {
            "project_variables__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");
            }

            "project_variables__selection" => {
                // state.floating_window.set(FloatingWindow::None);
                // context.set_focus("id", "app");
                //
                // let value = &*value.to_common_str();
                // let project = serde_json::from_str::<PersistedProject>(value);
                //
                // match project {
                //     Ok(project) => {
                //         state.project.set((&project).into());
                //         state.endpoint_count.set(project.endpoints.len() as u8);
                //
                //         let default_endpoint: &Value<Endpoint> = &(Endpoint::new().into());
                //
                //         let project = state.project.to_ref();
                //         let endpoints = project.endpoints.to_ref();
                //         let endpoint = endpoints.get(0).unwrap_or(default_endpoint);
                //
                //         *state.endpoint.to_mut() = endpoint.to_ref().clone();
                //
                //         // Update url input in dashboard
                //         let url = endpoint.to_ref().url.to_ref().to_string();
                //         let _ =
                //             send_message("url_text_input", url, &component_ids, context.emitter);
                //
                //         let body = endpoint.to_ref().body.to_ref().to_string();
                //         let textarea_msg = TextAreaMessages::SetInput(body);
                //         if let Ok(message) = serde_json::to_string(&textarea_msg) {
                //             let _ = send_message(
                //                 "request_body_input",
                //                 message,
                //                 &component_ids,
                //                 context.emitter,
                //             );
                //         }
                //     }
                //     Err(_) => todo!(),
                // }
            }

            "project_variables__delete" => {
                // state.floating_window.set(FloatingWindow::ConfirmProject);
                //
                // let value = &*value.to_common_str();
                // let project = serde_json::from_str::<PersistedProject>(value);
                //
                // match project {
                //     Ok(project) => {
                //         let confirm_message = ConfirmDeleteProject {
                //             title: format!("Delete {}", project.name),
                //             message: "Are you sure you want to delete?".into(),
                //             project,
                //         };
                //
                //         if let Ok(message) = serde_json::to_string(&confirm_message) {
                //             let confirm_action_window_id =
                //                 component_ids.get("confirm_action_window");
                //             if let Some(id) = confirm_action_window_id {
                //                 context.emit(*id, message);
                //             }
                //         }
                //     }
                //     Err(_) => todo!(),
                // }
            }

            _ => {}
        }
    }
}

impl Component for ProjectVariables {
    type State = ProjectVariablesState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match event.code {
            anathema::component::KeyCode::Char(char) => match char {
                'j' => self.move_cursor_down(state),
                'k' => self.move_cursor_up(state),
                'd' => {
                    let selected_index = *state.cursor.to_ref() as usize;
                    let project = self.variables_list.get(selected_index);

                    match project {
                        Some(project) => match serde_json::to_string(project) {
                            Ok(project_json) => {
                                state.selected_project.set(project_json);
                                context.publish("project_variables__delete", |state| {
                                    &state.selected_project
                                })
                            }

                            Err(_) => {
                                context.publish("project_variables__cancel", |state| &state.cursor)
                            }
                        },
                        None => context.publish("project_variables__cancel", |state| &state.cursor),
                    }
                }
                _ => {}
            },

            anathema::component::KeyCode::Up => self.move_cursor_up(state),
            anathema::component::KeyCode::Down => self.move_cursor_down(state),

            anathema::component::KeyCode::Esc => {
                // NOTE: This sends cursor to satisfy publish() but is not used
                context.publish("project_variables__cancel", |state| &state.cursor)
            }

            anathema::component::KeyCode::Enter => {
                let selected_index = *state.cursor.to_ref() as usize;
                let project = self.variables_list.get(selected_index);

                match project {
                    Some(project) => match serde_json::to_string(project) {
                        Ok(project_json) => {
                            state.selected_project.set(project_json);
                            context.publish("project_variables__selection", |state| {
                                &state.selected_project
                            });
                        }
                        Err(_) => {
                            context.publish("project_variables__cancel", |state| &state.cursor)
                        }
                    },
                    None => context.publish("project_variables__cancel", |state| &state.cursor),
                }
            }

            _ => {}
        }
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);
    }

    fn message(
        &mut self,
        _: Self::Message,
        _: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        // println!("Received message in project window: {message}");

        // NOTE: The currently selected project might need to be sent from the dashboard
        // when opening the project window after choosing a project
    }
}
