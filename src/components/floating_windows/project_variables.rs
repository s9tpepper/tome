use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{Component, ComponentId},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{List, State, Value},
    widgets::Elements,
};
use serde::{Deserialize, Serialize};

use crate::{
    components::dashboard::{DashboardMessageHandler, DashboardState},
    messages::confirm_actions::{ConfirmAction, ConfirmDetails},
    projects::{PersistedVariable, ProjectVariable},
    theme::{get_app_theme, AppTheme},
};

use super::FloatingWindow;

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
    selected_variable: Value<String>,
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
            selected_variable: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ProjectVariablesMessages {
    SetList(Vec<PersistedVariable>),
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

    fn open_add_variable_window(&self, context: &mut Context<'_, ProjectVariablesState>) {
        context.publish("open_add_variable_window", |state| &state.cursor);
    }

    fn open_edit_variable_window(
        &self,
        state: &mut ProjectVariablesState,
        mut context: Context<'_, ProjectVariablesState>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let persisted_variable = self.variables_list.get(selected_index);

        match persisted_variable {
            Some(persisted_variable) => match serde_json::to_string(persisted_variable) {
                Ok(persisted_variable_json) => {
                    state.selected_variable.set(persisted_variable_json);
                    context.publish("rename_variable", |state| &state.selected_variable)
                }

                Err(_) => context.publish("project_variables__cancel", |state| &state.cursor),
            },
            None => context.publish("project_variables__cancel", |state| &state.cursor),
        }
    }

    fn open_delete_variable_window(
        &self,
        state: &mut ProjectVariablesState,
        mut context: Context<'_, ProjectVariablesState>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let persisted_variable = self.variables_list.get(selected_index);

        let Some(persisted_variable) = persisted_variable else {
            context.publish("project_variables__cancel", |state| &state.cursor);
            return;
        };

        match serde_json::to_string(persisted_variable) {
            Ok(variable_json) => {
                state.selected_variable.set(variable_json);
                context.publish("project_variables__delete", |state| {
                    &state.selected_variable
                });
            }

            Err(_) => context.publish("project_variables__cancel", |state| &state.cursor),
        }
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

        let list_length = self.variables_list.len();
        let last_poss = list_length.saturating_sub(1);

        let first = if first_index > last_poss {
            0
        } else {
            first_index
        };

        let last = if last_index > last_poss {
            last_poss
        } else if last_index > first {
            first
        } else {
            last_index
        };

        let display_projects = &self.variables_list[first..=last];
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
        mut context: Context<'_, DashboardState>,
        _: Elements<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
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
                state.floating_window.set(FloatingWindow::ConfirmAction);
                context.set_focus("id", "confirm_action_window");

                let value = &*value.to_common_str();
                let variable = serde_json::from_str::<PersistedVariable>(value);

                match variable {
                    Ok(variable) => {
                        let confirm_delete_endpoint = ConfirmDetails {
                            title: format!("Delete {}", variable.name.clone().unwrap_or_default()),
                            message: "Are you sure you want to delete?".into(),
                            data: variable,
                        };

                        let confirm_message =
                            ConfirmAction::ConfirmDeletePersistedVariable(confirm_delete_endpoint);

                        if let Ok(message) = serde_json::to_string(&confirm_message) {
                            let confirm_action_window_id =
                                component_ids.get("confirm_action_window");
                            if let Some(id) = confirm_action_window_id {
                                context.emit(*id, message);
                            }
                        }
                    }

                    // TODO: Fix these todo()!
                    Err(_) => {}
                }
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
        mut context: Context<'_, Self::State>,
    ) {
        match event.code {
            anathema::component::KeyCode::Char(char) => match char {
                'j' => self.move_cursor_down(state),
                'k' => self.move_cursor_up(state),
                'a' => self.open_add_variable_window(&mut context),
                'e' => self.open_edit_variable_window(state, context),
                'd' => self.open_delete_variable_window(state, context),

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
                            state.selected_variable.set(project_json);
                            context.publish("project_variables__selection", |state| {
                                &state.selected_variable
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
        _: Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: Context<'_, Self::State>,
    ) {
        let Ok(project_variables_messages) =
            serde_json::from_str::<ProjectVariablesMessages>(&message)
        else {
            return;
        };

        match project_variables_messages {
            ProjectVariablesMessages::SetList(vec) => {
                self.variables_list = vec;

                let first_index = 0;
                let last_index = 4;
                let selected_index = 0;

                self.update_project_list(first_index, last_index, selected_index, state);
            }
        }
    }
}
