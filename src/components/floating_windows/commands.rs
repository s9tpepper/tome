use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{self, Children, Component, ComponentId, Context},
    runtime::Builder,
    state::{State, Value},
};

use crate::{
    compatibility::postman::export_postman,
    components::{
        dashboard::{DashboardMessageHandler, DashboardMessages, DashboardState},
        send_message,
    },
    projects::{PersistedProject, PersistedVariable},
    templates::template,
    theme::{get_app_theme, AppTheme},
};

use super::{
    add_project_variable::AddProjectVariableMessages, project_variables::ProjectVariablesMessages,
    FloatingWindow,
};

#[derive(Default)]
pub struct Commands;

impl Commands {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
    ) -> anyhow::Result<()> {
        let app_id = builder.component(
            "commands_window",
            template("floating_windows/templates/commands"),
            Commands {},
            CommandsState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert("commands_window".to_string(), app_id);

        Ok(())
    }

    #[allow(unused)]
    fn update_app_theme(&self, state: &mut CommandsState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }
}

#[derive(Default, State)]
pub struct CommandsState {
    app_theme: Value<AppTheme>,
    command: Value<char>,
}

impl CommandsState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        CommandsState {
            app_theme: app_theme.into(),
            command: ' '.into(),
        }
    }
}

impl Component for Commands {
    type State = CommandsState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        self.update_app_theme(state);
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => {
                state.command.set(char);
                context.publish("commands__selection", state.command.copy_value());
            }

            anathema::component::KeyCode::Esc => {
                context.publish("commands__cancel", state.command.copy_value());
            }

            _ => {}
        }
    }
}

impl DashboardMessageHandler for Commands {
    fn handle_message(
        event: &mut component::UserEvent<'_>,
        _ident: impl Into<String>,
        state: &mut DashboardState,
        mut context: Context<'_, '_, DashboardState>,
        _children: Children<'_, '_>,
        component_ids: Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let value = event.data::<String>().clone();
        let event_name: String = event.name().to_string();

        match event_name.as_str() {
            "commands__selection" => match value.as_str() {
                "a" => {
                    state
                        .floating_window
                        .set(FloatingWindow::AddProjectVariable);
                    context
                        .components
                        .by_attribute("id", "add_project_variable")
                        .focus();

                    let Ok(message) =
                        serde_json::to_string(&AddProjectVariableMessages::InitialFocus)
                    else {
                        return;
                    };

                    let _ = send_message(
                        "add_project_variable",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                }

                "v" => {
                    state
                        .floating_window
                        .set(FloatingWindow::ViewProjectVariables);
                    context
                        .components
                        .by_attribute("id", "project_variables")
                        .focus();

                    let variables: Vec<PersistedVariable> = state
                        .project
                        .to_ref()
                        .variable
                        .to_ref()
                        .iter()
                        .map(|vpv| {
                            let pv = vpv.to_ref();
                            let persisted_var: PersistedVariable = pv.as_ref().into();

                            persisted_var
                        })
                        .collect();

                    let project_variables_messages = ProjectVariablesMessages::SetList(variables);
                    let Ok(message) = serde_json::to_string(&project_variables_messages) else {
                        return;
                    };

                    let _ = send_message(
                        "project_variables",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                }

                "g" => {
                    state.floating_window.set(FloatingWindow::CodeGen);
                    context
                        .components
                        .by_attribute("id", "codegen_window")
                        .focus();
                }

                "i" => {
                    state
                        .floating_window
                        .set(FloatingWindow::PostmanFileSelector);
                    context
                        .components
                        .by_attribute("id", "postman_file_selector")
                        .focus();
                }

                "e" => {
                    state.floating_window.set(FloatingWindow::CodeGen);
                    context
                        .components
                        .by_attribute("id", "codegen_window")
                        .focus();

                    let project: PersistedProject = (&*state.project.to_ref()).into();

                    match export_postman(project) {
                        Ok(_) => {
                            let title = "Postman Export".to_string();
                            let message = "Postman exported successfully".to_string();
                            let dashboard_message = DashboardMessages::ShowSucces((title, message));
                            let msg = serde_json::to_string(&dashboard_message);
                            if let Ok(msg) = msg {
                                let _ =
                                    send_message("dashboard", msg, &component_ids, context.emitter);
                            }
                        }
                        Err(_) => {
                            let message = "Postman export failed".to_string();
                            let dashboard_message = DashboardMessages::ShowError(message);
                            let msg = serde_json::to_string(&dashboard_message);
                            if let Ok(msg) = msg {
                                let _ =
                                    send_message("dashboard", msg, &component_ids, context.emitter);
                            }
                        }
                    }
                }

                _ => {}
            },

            "commands__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                context.components.by_attribute("id", "app").focus();
            }

            _ => {}
        }
    }
}
