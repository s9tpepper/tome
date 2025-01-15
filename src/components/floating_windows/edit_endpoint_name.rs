use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{State, Value},
    widgets::Elements,
};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        dashboard::{DashboardMessageHandler, DashboardMessages},
        send_message,
    },
    projects::{rename_endpoint, PersistedEndpoint},
    theme::{get_app_theme, AppTheme},
};

use super::edit_project_name::{SpecificNameChange, SpecificNameUpdate};

const TEMPLATE: &str = "./src/components/floating_windows/templates/edit_endpoint_name.aml";

#[derive(Debug, Serialize, Deserialize)]
pub enum EditEndpointNameMessages {
    ClearInput,
    InputValue((String, Vec<String>)),
    Specifically((String, PersistedEndpoint, Vec<String>)),
}

pub struct EditEndpointName {
    persisted_project_name: Option<String>,
    persisted_endpoint: Option<PersistedEndpoint>,

    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl DashboardMessageHandler for EditEndpointName {
    fn handle_message(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut crate::components::dashboard::DashboardState,
        mut context: anathema::prelude::Context<'_, crate::components::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();
        match event.as_str() {
            "edit_endpoint_name__specific_endpoint_rename" => {
                let Ok(specific_name_update) =
                    serde_json::from_str::<SpecificNameUpdate>(&value.to_string())
                else {
                    let error_message =
                        "There was an error while processing the name update".to_string();
                    let dashboard_messages = DashboardMessages::ShowError(error_message);

                    let Ok(message) = serde_json::to_string(&dashboard_messages) else {
                        return;
                    };

                    let _ = send_message("dashboard", message, &component_ids, context.emitter);
                    return;
                };

                if *state.endpoint.to_ref().name.to_ref() == specific_name_update.old_name {
                    state
                        .endpoint
                        .to_mut()
                        .name
                        .set(specific_name_update.new_name.clone());
                }

                state
                    .project
                    .to_mut()
                    .endpoints
                    .to_mut()
                    .iter_mut()
                    .for_each(|endpoint| {
                        let mut e = endpoint.to_mut();

                        if e.name.to_ref().to_string() == specific_name_update.old_name {
                            e.name.set(specific_name_update.new_name.clone());
                        }
                    });

                state
                    .floating_window
                    .set(crate::components::dashboard::FloatingWindow::None);

                context.set_focus("id", "app");

                if let Ok(message) = serde_json::to_string(&EditEndpointNameMessages::ClearInput) {
                    let _ = send_message(
                        "edit_project_name",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                };
            }

            "edit_endpoint_name__submit" => {
                let new_name = value.to_string();
                state.endpoint.to_mut().name.set(new_name);

                state
                    .floating_window
                    .set(crate::components::dashboard::FloatingWindow::None);

                context.set_focus("id", "app");

                if let Ok(message) = serde_json::to_string(&EditEndpointNameMessages::ClearInput) {
                    let _ = send_message(
                        "edit_endpoint_name",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                };
            }

            "edit_endpoint_name__cancel" => {
                state
                    .floating_window
                    .set(crate::components::dashboard::FloatingWindow::None);

                context.set_focus("id", "app");
            }
            _ => {}
        }
    }
}

impl Component for EditEndpointName {
    type State = EditEndpointNameState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if let Ok(msg) = serde_json::from_str::<EditEndpointNameMessages>(&message) {
            match msg {
                EditEndpointNameMessages::ClearInput => {
                    state.unique_name_error.set("".to_string());
                    state.name.set("".to_string());

                    if let Ok(ids) = self.component_ids.try_borrow() {
                        let _ = send_message(
                            "edit_endpoint_name_input",
                            "".to_string(),
                            &ids,
                            context.emitter,
                        );
                    }
                }

                EditEndpointNameMessages::InputValue((input_value, current_names)) => {
                    state.unique_name_error.set("".to_string());
                    state.name.set(input_value.clone());
                    state.current_names = current_names;

                    if let Ok(ids) = self.component_ids.try_borrow() {
                        let _ = send_message(
                            "edit_endpoint_name_input",
                            input_value,
                            &ids,
                            context.emitter,
                        );
                    }

                    context.set_focus("id", "endpoint_name_input");
                }

                EditEndpointNameMessages::Specifically((
                    project_name,
                    persisted_endpoint,
                    current_names,
                )) => {
                    state.current_names = current_names;
                    state.unique_name_error.set("".to_string());
                    let input_value = &persisted_endpoint.name;
                    state.name.set(input_value.to_string());

                    self.set_name_input(input_value, context);

                    self.persisted_project_name = Some(project_name);
                    self.persisted_endpoint = Some(persisted_endpoint);
                }
            }
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match ident {
            "name_input_escape" => context.set_focus("id", "edit_endpoint_name"),
            "name_input_update" => state.name.set(value.to_string()),
            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'e' => context.set_focus("id", "endpoint_name_input"),

                's' => {
                    let exists = state
                        .current_names
                        .iter()
                        .find(|n| **n == state.name.to_ref().to_string());

                    if exists.is_some() {
                        let unique_name_error = format!(
                            "The name '{}' is already in use",
                            state.name.to_ref().clone()
                        );

                        state.unique_name_error.set(unique_name_error);
                        return;
                    }

                    match &self.persisted_endpoint {
                        Some(persisted_endpoint) => {
                            self.rename_specific_endpoint(persisted_endpoint, state, context);
                        }
                        None => {
                            context.publish("edit_endpoint_name__submit", |state| &state.name);
                        }
                    }
                }

                'c' => {
                    context.publish("edit_endpoint_name__cancel", |state| &state.name);
                    state.unique_name_error.set("".to_string());
                }

                _ => {}
            },

            anathema::component::KeyCode::Esc => {
                context.publish("edit_endpoint_name__cancel", |state| &state.name);
                state.unique_name_error.set("".to_string());
            }

            _ => {}
        }
    }
}

impl EditEndpointName {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let app_theme = get_app_theme();
        let id = builder.register_component(
            "edit_endpoint_name",
            TEMPLATE,
            EditEndpointName {
                persisted_project_name: None,
                persisted_endpoint: None,
                component_ids: ids.clone(),
            },
            EditEndpointNameState {
                name: String::from("").into(),
                app_theme: app_theme.into(),
                current_names: vec![],
                unique_name_error: String::from("").into(),

                specific_name_change: None.into(),
            },
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("edit_endpoint_name"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut EditEndpointNameState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    fn set_name_input(&self, input_value: &str, mut context: Context<'_, EditEndpointNameState>) {
        if let Ok(ids) = self.component_ids.try_borrow() {
            let _ = send_message(
                "edit_endpoint_name_input",
                input_value.to_string(),
                &ids,
                context.emitter,
            );
        }

        context.set_focus("id", "endpoint_name_input");
    }

    fn rename_specific_endpoint(
        &self,
        endpoint: &PersistedEndpoint,
        state: &mut EditEndpointNameState,
        mut context: Context<'_, EditEndpointNameState>,
    ) {
        let specific_name_update = SpecificNameUpdate {
            old_name: endpoint.name.to_string(),
            new_name: state.name.to_ref().to_string(),
        };

        let Ok(common) = serde_json::to_string(&specific_name_update) else {
            let error_message = "There was an error with the name update".to_string();
            let dashboard_messages = DashboardMessages::ShowError(error_message);

            let Ok(message) = serde_json::to_string(&dashboard_messages) else {
                return;
            };

            let Ok(ids) = self.component_ids.try_borrow() else {
                return;
            };

            let _ = send_message("dashboard", message, &ids, context.emitter);
            return;
        };

        state.specific_name_change = Some(SpecificNameChange {
            old_name: endpoint.name.to_string().into(),
            new_name: state.name.to_ref().to_string().into(),
            common,
        })
        .into();

        let Some(project_name) = &self.persisted_project_name else {
            return;
        };

        match rename_endpoint(project_name, endpoint, &state.name.to_ref()) {
            Ok(_) => {
                context.publish("edit_endpoint_name__specific_endpoint_rename", |state| {
                    &state.specific_name_change
                });
            }
            Err(_) => {
                let error_message = "There was an error renaming the endpoint".to_string();
                let dashboard_messages = DashboardMessages::ShowError(error_message);

                let Ok(message) = serde_json::to_string(&dashboard_messages) else {
                    return;
                };

                let Ok(ids) = self.component_ids.try_borrow() else {
                    return;
                };

                let _ = send_message("dashboard", message, &ids, context.emitter);
            }
        }
    }
}

#[derive(State)]
pub struct EditEndpointNameState {
    name: Value<String>,
    app_theme: Value<AppTheme>,
    unique_name_error: Value<String>,

    #[state_ignore]
    current_names: Vec<String>,

    specific_name_change: Value<Option<SpecificNameChange>>,
}
