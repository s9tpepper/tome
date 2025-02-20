use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{self, Component, ComponentId},
    prelude::{self, Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, State, Value},
    widgets::{self, Elements},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::{
    app::GlobalEventHandler,
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
        value: CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut components::dashboard::DashboardState,
        mut context: prelude::Context<'_, components::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();
        match event.as_str() {
            "add_project_variable__submit" => {
                let persisted_variable: PersistedVariable =
                    serde_json::from_str(&value.to_string()).expect("???");
                let project_variable: ProjectVariable = persisted_variable.into();

                state.project.to_mut().variable.push(project_variable);

                state.floating_window.set(FloatingWindow::None);

                context.set_focus("id", "app");

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

                context.set_focus("id", "app");
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
        _: Elements<'_, '_>,
        _: prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: widgets::Elements<'_, '_>,
        mut context: prelude::Context<'_, Self::State>,
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
                    context.set_focus("id", "add_project_variable_name");
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

                    context.set_focus("id", "add_project_variable_name");
                }
            }
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: CommonVal<'_>,
        state: &mut Self::State,
        _: widgets::Elements<'_, '_>,
        mut context: prelude::Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match ident {
            "add_project_variable_name_escape" => context.set_focus("id", "add_project_variable"),
            "add_project_variable_name_update" => {
                state.variable.to_mut().name.set(value.to_string());
                state.variable.to_mut().update_common();
            }

            "add_project_variable_public_value_escape" => {
                context.set_focus("id", "add_project_variable")
            }
            "add_project_variable_public_value_update" => {
                state.variable.to_mut().public.set(value.to_string());
                state.variable.to_mut().update_common();
            }
            "add_project_variable_private_value_escape" => {
                context.set_focus("id", "add_project_variable")
            }
            "add_project_variable_private_value_update" => {
                state.variable.to_mut().private.set(value.to_string());
                state.variable.to_mut().update_common();
            }
            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: component::KeyEvent,
        _: &mut Self::State,
        _: widgets::Elements<'_, '_>,
        mut context: prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            component::KeyCode::Char(char) => match char {
                'v' => context.set_focus("id", "add_project_variable_name"),
                'u' => context.set_focus("id", "add_project_variable_public_value"),
                'p' => context.set_focus("id", "add_project_variable_private_value"),

                's' => context.publish("add_project_variable__submit", |state| &state.variable),

                'c' => context.publish("add_project_variable__cancel", |state| &state.variable),

                _ => {}
            },

            component::KeyCode::Esc => {
                context.publish("add_project_variable__cancel", |state| &state.cancel)
            }

            _ => {}
        }
    }
}

impl AddProjectVariable {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
    ) -> Result<()> {
        let app_theme = get_app_theme();

        let variable = Variable {
            name: String::from("").into(),
            public: String::from("").into(),
            private: String::from("").into(),
            common: String::from(""),
        };

        let id = builder.register_component(
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
        context: &mut Context<'_, AddProjectVariableState>,
    ) {
        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };

        let _ = send_message(ident, value.to_string(), &ids, context.emitter);
    }
}

pub struct Variable {
    pub name: Value<String>,
    pub public: Value<String>,
    pub private: Value<String>,
    pub common: String,
}

impl Variable {
    pub fn update_common(&mut self) {
        let persisted_variable = PersistedVariable {
            id: None,
            key: Some(self.name.to_ref().to_string()),
            value: Some(self.public.to_ref().to_string()),
            private: Some(self.private.to_ref().to_string()),
            r#type: Some(crate::projects::VariableType::String),
            name: Some(self.name.to_ref().to_string()),
            system: Some(false),
            disabled: Some(false),
        };

        let Ok(common_val_str) = serde_json::to_string(&persisted_variable) else {
            return;
        };

        self.common = common_val_str;
    }
}

impl ::anathema::state::State for Variable {
    fn state_get(
        &self,
        path: ::anathema::state::Path<'_>,
        sub: ::anathema::state::Subscriber,
    ) -> ::core::prelude::v1::Option<::anathema::state::ValueRef> {
        let ::anathema::state::Path::Key(key) = path else {
            return ::core::prelude::v1::None;
        };
        match key {
            "name" => ::core::prelude::v1::Some(self.name.value_ref(sub)),
            "public" => ::core::prelude::v1::Some(self.public.value_ref(sub)),
            "private" => ::core::prelude::v1::Some(self.private.value_ref(sub)),
            _ => ::core::prelude::v1::None,
        }
    }

    fn state_lookup(
        &self,
        path: ::anathema::state::Path<'_>,
    ) -> ::core::prelude::v1::Option<::anathema::state::PendingValue> {
        let ::anathema::state::Path::Key(key) = path else {
            return ::core::prelude::v1::None;
        };
        match key {
            "name" => ::core::prelude::v1::Some(self.name.to_pending()),
            "public" => ::core::prelude::v1::Some(self.public.to_pending()),
            "private" => ::core::prelude::v1::Some(self.private.to_pending()),
            _ => ::core::prelude::v1::None,
        }
    }

    fn to_common(&self) -> ::core::prelude::v1::Option<::anathema::state::CommonVal<'_>> {
        Some(CommonVal::Str(&self.common))
    }
}

struct Cancel;
impl State for Cancel {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        Some(CommonVal::Str(""))
    }
}

#[derive(State)]
pub struct AddProjectVariableState {
    app_theme: Value<AppTheme>,
    variable: Value<Variable>,
    cancel: Value<Cancel>,
    unique_name_error: Value<String>,

    #[state_ignore]
    current_names: Vec<String>,
}
