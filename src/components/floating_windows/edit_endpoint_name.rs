use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{State, Value},
    widgets::Elements,
};
use serde::{Deserialize, Serialize};

use crate::{
    components::{dashboard::DashboardMessageHandler, send_message},
    theme::{get_app_theme, AppTheme},
};

const TEMPLATE: &str = "./src/components/floating_windows/templates/edit_endpoint_name.aml";

#[derive(Debug, Serialize, Deserialize)]
pub enum EditEndpointNameMessages {
    ClearInput,
    InputValue(String),
}

pub struct EditEndpointName {
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

                EditEndpointNameMessages::InputValue(input_value) => {
                    state.name.set(input_value.clone());

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
        _: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'e' => context.set_focus("id", "endpoint_name_input"),
                's' => context.publish("edit_endpoint_name__submit", |state| &state.name),
                'c' => context.publish("edit_endpoint_name__cancel", |state| &state.name),

                _ => {}
            },
            anathema::component::KeyCode::Esc => {
                context.publish("edit_endpoint_name__cancel", |state| &state.name)
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
                component_ids: ids.clone(),
            },
            EditEndpointNameState {
                name: String::from("").into(),
                app_theme: app_theme.into(),
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
}

#[derive(State)]
pub struct EditEndpointNameState {
    name: Value<String>,
    app_theme: Value<AppTheme>,
}
