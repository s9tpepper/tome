use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{self, Children, Component, ComponentId, Context},
    runtime::Builder,
    state::{State, Value},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::{
    components::{self, dashboard::DashboardMessageHandler, send_message},
    projects::{PersistedVariable, ProjectVariable},
    templates::template,
    theme::{get_app_theme, AppTheme},
};

use super::FloatingWindow;

#[derive(Debug, Serialize, Deserialize)]
pub enum AddProjectVariableMessages {
    InitialFocus,
    ClearInput,
    Specifically((String, PersistedVariable, Vec<String>)),
}

pub struct AddProjectVariable {
    persisted_project_name: Option<String>,
    persisted_variable: Option<PersistedVariable>,

    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl DashboardMessageHandler for AddProjectVariable {
    fn handle_message(
        event: &mut component::UserEvent<'_>,
        ident: impl Into<String>,
        state: &mut components::dashboard::DashboardState,
        mut context: component::Context<'_, '_, components::dashboard::DashboardState>,
        _children: component::Children<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event_name: String = ident.into();
        match event_name.as_str() {
            "add_project_variable__submit" => {
                let persisted_variable = event.data::<PersistedVariable>();

                let project_variable: ProjectVariable = persisted_variable.into();

                state.project.to_mut().variable.push(project_variable);

                state.floating_window.set(FloatingWindow::None);

                context.components.by_attribute("id", "app").focus();

                if let Ok(message) = to_string(&AddProjectVariableMessages::ClearInput) {
                    let _ = send_message(
                        "add_project_variable",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                };
            }

            "add_project_variable__cancel" => {
                state.floating_window.set(FloatingWindow::None);

                context.components.by_attribute("id", "app").focus();
            }
            _ => {}
        }
    }
}

impl Component for AddProjectVariable {
    type State = AddProjectVariableState;
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

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        if let Ok(msg) = from_str::<AddProjectVariableMessages>(&message) {
            match msg {
                AddProjectVariableMessages::ClearInput => {
                    if let Ok(ids) = self.component_ids.try_borrow() {
                        let _ = send_message(
                            "add_project_variable_name",
                            "".to_string(),
                            &ids,
                            context.emitter,
                        );

                        let _ = send_message(
                            "add_project_variable_public_value",
                            "".to_string(),
                            &ids,
                            context.emitter,
                        );

                        let _ = send_message(
                            "add_project_variable_private_value",
                            "".to_string(),
                            &ids,
                            context.emitter,
                        );
                    }
                }

                AddProjectVariableMessages::InitialFocus => {
                    context
                        .components
                        .by_attribute("id", "add_project_variable_name")
                        .focus();
                }

                AddProjectVariableMessages::Specifically((
                    project_name,
                    persisted_variable,
                    current_names,
                )) => {
                    state.current_names = current_names;
                    state.unique_name_error.set("".to_string());

                    let mut project_variable = state.variable.to_mut();

                    project_variable
                        .name
                        .set(persisted_variable.name.clone().unwrap_or_default());

                    project_variable
                        .public
                        .set(persisted_variable.value.clone().unwrap_or_default());

                    project_variable
                        .private
                        .set(persisted_variable.private.clone().unwrap_or_default());

                    self.set_input_value(
                        "add_project_variable_name",
                        &persisted_variable.name.clone().unwrap_or_default(),
                        &mut context,
                    );

                    self.set_input_value(
                        "add_project_variable_public_value",
                        &persisted_variable.value.clone().unwrap_or_default(),
                        &mut context,
                    );

                    self.set_input_value(
                        "add_project_variable_private_value",
                        &persisted_variable.private.clone().unwrap_or_default(),
                        &mut context,
                    );

                    self.persisted_project_name = Some(project_name);
                    self.persisted_variable = Some(persisted_variable);

                    context
                        .components
                        .by_attribute("id", "add_project_variable_name")
                        .focus();
                }
            }
        }
    }

    fn on_event(
        &mut self,
        event: &mut component::UserEvent<'_>,
        state: &mut Self::State,
        _children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        let value = event.data::<String>();
        match event.name() {
            "add_project_variable_name_escape" => context
                .components
                .by_attribute("id", "add_project_variable")
                .focus(),
            "add_project_variable_name_update" => {
                state.variable.to_mut().name.set(value.to_string());
            }

            "add_project_variable_public_value_escape" => context
                .components
                .by_attribute("id", "add_project_variable")
                .focus(),
            "add_project_variable_public_value_update" => {
                state.variable.to_mut().public.set(value.to_string());
            }
            "add_project_variable_private_value_escape" => context
                .components
                .by_attribute("id", "add_project_variable")
                .focus(),
            "add_project_variable_private_value_update" => {
                state.variable.to_mut().private.set(value.to_string());
            }
            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: component::KeyEvent,
        _: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match key.code {
            component::KeyCode::Char(char) => match char {
                'v' => context
                    .components
                    .by_attribute("id", "add_project_variable_name")
                    .focus(),
                'u' => context
                    .components
                    .by_attribute("id", "add_project_variable_public_value")
                    .focus(),
                'p' => context
                    .components
                    .by_attribute("id", "add_project_variable_private_value")
                    .focus(),

                's' => context.publish("add_project_variable__submit", |state: Self::State| {
                    state.variable
                }),

                'c' => context.publish("add_project_variable__cancel", |state: Self::State| {
                    state.variable
                }),

                _ => {}
            },

            component::KeyCode::Esc => context
                .publish("add_project_variable__cancel", |state: Self::State| {
                    state.cancel
                }),

            _ => {}
        }
    }
}

impl AddProjectVariable {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
    ) -> Result<()> {
        let app_theme = get_app_theme();

        let variable = Variable {
            name: String::from("").into(),
            public: String::from("").into(),
            private: String::from("").into(),
        };

        let id = builder.component(
            "add_project_variable",
            template("floating_windows/templates/add_project_variable"),
            AddProjectVariable {
                persisted_project_name: None,
                persisted_variable: None,

                component_ids: ids.clone(),
            },
            AddProjectVariableState {
                app_theme: app_theme.into(),
                variable: variable.into(),
                cancel: Cancel {}.into(),
                current_names: vec![],
                unique_name_error: "".to_string().into(),
            },
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("add_project_variable"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut AddProjectVariableState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    fn set_input_value(
        &self,
        ident: &str,
        value: &str,
        context: &mut Context<'_, '_, AddProjectVariableState>,
    ) {
        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };

        let _ = send_message(ident, value.to_string(), &ids, context.emitter);
    }
}

#[derive(Debug, State)]
pub struct Variable {
    pub name: Value<String>,
    pub public: Value<String>,
    pub private: Value<String>,
}

#[derive(State)]
struct Cancel;

#[derive(State)]
pub struct AddProjectVariableState {
    app_theme: Value<AppTheme>,
    variable: Value<Variable>,
    cancel: Value<Cancel>,
    unique_name_error: Value<String>,

    #[state_ignore]
    current_names: Vec<String>,
}
