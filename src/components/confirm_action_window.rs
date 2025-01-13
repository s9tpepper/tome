use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{State, Value},
};

use crate::{
    messages::confirm_delete_project::{
        ConfirmAction, DeleteProjectDetails, DeleteProjectDetailsAnswer,
    },
    theme::{get_app_theme, AppTheme},
};

use super::{dashboard::DashboardMessages, send_message};

pub const CONFIRM_ACTION_WINDOW_TEMPLATE: &str =
    "./src/components/templates/confirm_action_window.aml";

#[derive(Default)]
pub struct ConfirmActionWindow {
    confirm_action: Option<ConfirmAction>,

    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl ConfirmActionWindow {
    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        ConfirmActionWindow {
            component_ids,
            confirm_action: None,
        }
    }

    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let id = builder.register_component(
            "confirm_action_window",
            CONFIRM_ACTION_WINDOW_TEMPLATE,
            ConfirmActionWindow::new(ids.clone()),
            ConfirmActionWindowState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("confirm_action_window"), id);

        Ok(())
    }
}

#[derive(Default, State)]
pub struct ConfirmActionWindowState {
    title: Value<String>,
    message: Value<String>,
    app_theme: Value<AppTheme>,
}

impl ConfirmActionWindowState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        ConfirmActionWindowState {
            title: "".to_string().into(),
            message: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

impl Component for ConfirmActionWindow {
    type State = ConfirmActionWindowState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        _: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'y' | 'n' => match &self.confirm_action {
                    Some(confirm_action) => {
                        let answer = match char {
                            'y' => true,
                            'n' => false,
                            _ => false,
                        };

                        let message = match confirm_action {
                            ConfirmAction::ConfirmDeletePersistedProject(
                                DeleteProjectDetails { project, .. },
                            ) => DashboardMessages::Confirmations(
                                ConfirmAction::ConfirmationDeletePersistedProject(
                                    DeleteProjectDetailsAnswer {
                                        project: project.clone(),
                                        answer,
                                    },
                                ),
                            ),

                            ConfirmAction::ConfirmDeletePersistedProjectVariable(
                                _delete_persisted_variable_details,
                            ) => todo!(),

                            _ => unreachable!(),
                        };

                        let Ok(message) = serde_json::to_string(&message) else {
                            return;
                        };

                        let Ok(ids) = self.component_ids.try_borrow() else {
                            return;
                        };

                        let _ = send_message("dashboard", message, &ids, context.emitter);
                    }
                    None => unreachable!(),
                },

                _ => {}
            },

            anathema::component::KeyCode::Esc => todo!(),

            _ => {}
        }
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        let Ok(confirm_action) = serde_json::from_str::<ConfirmAction>(message.as_str()) else {
            // TODO: Close this and send an error message
            return;
        };

        match &confirm_action {
            ConfirmAction::ConfirmDeletePersistedProject(confirm_delete_project) => {
                state.title.set(confirm_delete_project.title.clone());
                state.message.set(confirm_delete_project.message.clone());
            }

            ConfirmAction::ConfirmDeletePersistedProjectVariable(
                _delete_persisted_variable_message,
            ) => {
                todo!()
            }

            _ => {}
        }

        self.confirm_action = Some(confirm_action);
    }
}
