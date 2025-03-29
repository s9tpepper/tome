use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId, KeyCode, KeyEvent},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{State, Value},
    widgets::{components::events::KeyState, Elements},
};

use crate::{
    app::GlobalEventHandler,
    messages::confirm_actions::{ConfirmAction, ConfirmDetails, ConfirmationAnswer},
    templates::template,
    theme::{get_app_theme, AppTheme},
};

use super::{
    dashboard::{DashboardMessageHandler, DashboardMessages},
    floating_windows::FloatingWindow,
    send_message,
};

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
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
    ) -> anyhow::Result<()> {
        let id = builder.register_component(
            "confirm_action_window",
            template("templates/confirm_action_window"),
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
    #[state_ignore]
    active: bool,

    title: Value<String>,
    message: Value<String>,
    app_theme: Value<AppTheme>,
}

impl ConfirmActionWindowState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        ConfirmActionWindowState {
            active: false,
            title: "".to_string().into(),
            message: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

impl DashboardMessageHandler for ConfirmActionWindow {
    fn handle_message(
        _: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::dashboard::DashboardState,
        mut context: Context<'_, super::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        _: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();

        #[allow(clippy::single_match)]
        match event.as_str() {
            "confirm_action__cancel" => {
                state.floating_window.set(FloatingWindow::None);

                context.set_focus("id", "app");
            }

            _ => {}
        }
    }
}

impl Component for ConfirmActionWindow {
    type State = ConfirmActionWindowState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
    ) {
        state.active = true;
    }

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
    ) {
        state.active = false;
    }

    fn on_mouse(
        &mut self,
        mouse: anathema::component::MouseEvent,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        if !state.active || !mouse.lsb_up() {
            return;
        }

        let is_yes_click = RefCell::new(false);
        elements
            .at_position(mouse.pos())
            .by_attribute("id", "yes_button")
            .first(|_, _| {
                *is_yes_click.borrow_mut() = true;
            });

        if *is_yes_click.borrow() {
            self.on_key(
                KeyEvent {
                    code: KeyCode::Char('y'),
                    ctrl: false,
                    state: KeyState::Release,
                },
                state,
                elements,
                context,
            );

            return;
        }

        let is_no_click = RefCell::new(false);
        elements
            .at_position(mouse.pos())
            .by_attribute("id", "no_button")
            .first(|_, _| {
                *is_no_click.borrow_mut() = true;
            });

        if *is_no_click.borrow() {
            self.on_key(
                KeyEvent {
                    code: KeyCode::Char('n'),
                    ctrl: false,
                    state: KeyState::Release,
                },
                state,
                elements,
                context,
            );
        }
    }

    fn on_key(
        &mut self,
        key: KeyEvent,
        _: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        match key.code {
            KeyCode::Char(char) => match char {
                'y' | 'n' => match &self.confirm_action {
                    Some(confirm_action) => {
                        let answer = match char {
                            'y' => true,
                            'n' => false,
                            _ => false,
                        };

                        let message = match confirm_action {
                            ConfirmAction::ConfirmDeletePersistedProject(ConfirmDetails {
                                data: project,
                                ..
                            }) => DashboardMessages::Confirmations(
                                ConfirmAction::ConfirmationDeletePersistedProject(
                                    ConfirmationAnswer {
                                        data: project.clone(),
                                        answer,
                                    },
                                ),
                            ),

                            ConfirmAction::ConfirmDeletePersistedEndpoint(ConfirmDetails {
                                data: endpoint,
                                ..
                            }) => DashboardMessages::Confirmations(
                                ConfirmAction::ConfirmationDeletePersistedEndpoint(
                                    ConfirmationAnswer {
                                        data: endpoint.clone(),
                                        answer,
                                    },
                                ),
                            ),

                            ConfirmAction::ConfirmDeletePersistedVariable(ConfirmDetails {
                                data: variable,
                                ..
                            }) => DashboardMessages::Confirmations(
                                ConfirmAction::ConfirmationDeletePersistedVariable(
                                    ConfirmationAnswer {
                                        data: variable.clone(),
                                        answer,
                                    },
                                ),
                            ),

                            ConfirmAction::ConfirmDeleteHeader(ConfirmDetails {
                                data: variable,
                                ..
                            }) => DashboardMessages::Confirmations(
                                ConfirmAction::ConfirmationDeleteHeader(ConfirmationAnswer {
                                    data: variable.clone(),
                                    answer,
                                }),
                            ),

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

            KeyCode::Esc => {
                context.publish("confirm_action__cancel", |state| &state.title);
            }

            _ => {}
        }
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
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

            ConfirmAction::ConfirmDeletePersistedEndpoint(delete_endpoint_details) => {
                state.title.set(delete_endpoint_details.title.clone());
                state.message.set(delete_endpoint_details.message.clone());
            }

            ConfirmAction::ConfirmDeleteHeader(delete_header_details) => {
                state.title.set(delete_header_details.title.clone());
                state.message.set(delete_header_details.message.clone());
            }

            ConfirmAction::ConfirmDeletePersistedVariable(delete_persisted_variable_message) => {
                state
                    .title
                    .set(delete_persisted_variable_message.title.clone());
                state
                    .message
                    .set(delete_persisted_variable_message.message.clone());
            }

            _ => {}
        }

        self.confirm_action = Some(confirm_action);
    }
}
