use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Children, Component, ComponentId, Context, MouseEvent},
    runtime::Builder,
    state::{Maybe, State, Value},
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        dashboard::{DashboardMessageHandler, DashboardMessages},
        send_message,
    },
    projects::{rename_endpoint, PersistedEndpoint},
    templates::template,
    theme::{get_app_theme, AppTheme},
};

use super::{
    edit_project_name::{SpecificNameChange, SpecificNameUpdate},
    FloatingWindow,
};

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
        event: &mut anathema::component::UserEvent<'_>,
        _ident: impl Into<String>,
        state: &mut crate::components::dashboard::DashboardState,
        mut context: anathema::component::Context<
            '_,
            '_,
            crate::components::dashboard::DashboardState,
        >,
        _children: anathema::component::Children<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event_name: String = event.name().to_string();
        match event_name.as_str() {
            "edit_endpoint_name__specific_endpoint_rename" => {
                let specific_name_update = event.data::<SpecificNameUpdate>();

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

                state.floating_window.set(FloatingWindow::None);

                context.components.by_attribute("id", "app").focus();

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
                info!("edit_endpoint_name.rs::edit_endpoint_name__submit");

                info!("Handling edit_endpoint_name__submit event");

                let new_name = event.data::<String>().clone();
                info!("new_name: {new_name}");

                state.endpoint.to_mut().name.set(new_name);

                state.floating_window.set(FloatingWindow::None);

                context.components.by_attribute("id", "app").focus();

                if let Ok(message) = serde_json::to_string(&EditEndpointNameMessages::ClearInput) {
                    info!("Clearing input for endpoint name edits");
                    let _ = send_message(
                        "edit_endpoint_name",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                };
            }

            "edit_endpoint_name__cancel" => {
                state.floating_window.set(FloatingWindow::None);

                context.components.by_attribute("id", "app").focus();
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

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        let mut context_ref = RefCell::new(context);

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "submit_button")
            .first(|_, _| {
                // TODO: Remove this state.active after Anathema update
                if state.active && mouse.left_up() {
                    self.submit(state, &mut context_ref);
                }
            });

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "cancel_button")
            .first(|_, _| {
                // TODO: Remove this state.active after Anathema update
                if state.active && mouse.left_up() {
                    self.cancel(state, &mut context_ref);
                }
            });
    }

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        state
            .cancel_button_color
            .set(state.button_color_unfocused.clone());

        state
            .success_button_color
            .set(state.button_color_unfocused.clone());
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        self.update_app_theme(state);

        state
            .cancel_button_color
            .set(state.success_color_focused.clone());

        state
            .success_button_color
            .set(state.cancel_color_focused.clone());
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
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

                    context
                        .components
                        .by_attribute("id", "endpoint_name_input")
                        .focus();
                    state.active = true;
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
                    state.active = true;
                }
            }
        }
    }

    fn on_event(
        &mut self,
        event: &mut anathema::component::UserEvent<'_>,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match event.name() {
            "name_input_escape" => context
                .components
                .by_attribute("id", "edit_endpoint_name")
                .focus(),
            "name_input_update" => state.name.set(event.data::<String>().clone()),
            "name_input_enter" => self.submit(state, &mut context.into()),
            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'e' => context
                    .components
                    .by_attribute("id", "endpoint_name_input")
                    .focus(),
                's' => self.submit(state, &mut context.into()),
                'c' => self.cancel(state, &mut context.into()),

                _ => {}
            },

            anathema::component::KeyCode::Esc => {
                context.publish("edit_endpoint_name__cancel", |state: Self::State| {
                    state.name
                });
                state.unique_name_error.set("".to_string());
            }

            _ => {}
        }
    }
}

impl EditEndpointName {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
    ) -> anyhow::Result<()> {
        let app_theme = get_app_theme();

        let submit_bg = app_theme.overlay_submit_background.to_ref().to_string();
        let cancel_bg = app_theme.overlay_cancel_background.to_ref().to_string();
        let unfocused_bg = app_theme.border_unfocused.to_ref().to_string();

        let id = builder.component(
            "edit_endpoint_name",
            template("floating_windows/templates/edit_endpoint_name"),
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

                success_color_focused: submit_bg,
                cancel_color_focused: cancel_bg,
                button_color_unfocused: unfocused_bg.clone(),
                success_button_color: unfocused_bg.clone().into(),
                cancel_button_color: unfocused_bg.into(),

                specific_name_change: None.into(),
                active: false,
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

    fn set_name_input(
        &self,
        input_value: &str,
        mut context: Context<'_, '_, EditEndpointNameState>,
    ) {
        if let Ok(ids) = self.component_ids.try_borrow() {
            let _ = send_message(
                "edit_endpoint_name_input",
                input_value.to_string(),
                &ids,
                context.emitter,
            );
        }

        context
            .components
            .by_attribute("id", "endpoint_name_input")
            .focus();
    }

    fn rename_specific_endpoint(
        &self,
        endpoint: &PersistedEndpoint,
        state: &mut EditEndpointNameState,
        context: &mut RefCell<Context<'_, '_, EditEndpointNameState>>,
    ) {
        state.specific_name_change = Maybe::some(SpecificNameChange {
            old_name: endpoint.name.to_string().into(),
            new_name: state.name.to_ref().to_string().into(),
        })
        .into();

        let Some(project_name) = &self.persisted_project_name else {
            return;
        };

        match rename_endpoint(project_name, endpoint, &state.name.to_ref()) {
            Ok(_) => {
                context.borrow_mut().publish(
                    "edit_endpoint_name__specific_endpoint_rename",
                    |state: EditEndpointNameState| state.specific_name_change,
                );
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

                let _ = send_message("dashboard", message, &ids, context.borrow().emitter);
            }
        }
    }

    fn cancel(
        &self,
        state: &mut EditEndpointNameState,
        context: &mut RefCell<Context<'_, '_, EditEndpointNameState>>,
    ) {
        context.borrow_mut().publish(
            "edit_endpoint_name__cancel",
            |state: EditEndpointNameState| state.name,
        );
        state.unique_name_error.set("".to_string());
    }

    fn submit(
        &self,
        state: &mut EditEndpointNameState,
        context: &mut RefCell<Context<'_, '_, EditEndpointNameState>>,
    ) {
        info!("edit_endpoint_name.rs::submit()");
        let exists = state
            .current_names
            .iter()
            .find(|n| **n == state.name.to_ref().to_string());
        info!("exists: {exists:?}");

        if exists.is_some() {
            let unique_name_error = format!(
                "The name '{}' is already in use",
                state.name.to_ref().clone()
            );
            info!("unique_name_error: {unique_name_error}");

            state.unique_name_error.set(unique_name_error);
            return;
        }

        match &self.persisted_endpoint {
            Some(persisted_endpoint) => {
                info!("Found a persisted endpoint, editing existing endpoint");

                self.rename_specific_endpoint(persisted_endpoint, state, context);
            }
            None => {
                info!("Did not find a persisted endpoint, editing new endpoint");
                info!("Publishing edit_endpoint_name__submit");

                context.borrow_mut().publish(
                    "edit_endpoint_name__submit",
                    |state: EditEndpointNameState| state.name,
                );
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

    specific_name_change: Value<Maybe<SpecificNameChange>>,

    success_button_color: Value<String>,
    cancel_button_color: Value<String>,

    #[state_ignore]
    success_color_focused: String,

    #[state_ignore]
    cancel_color_focused: String,

    #[state_ignore]
    button_color_unfocused: String,

    #[state_ignore]
    active: bool,
}
