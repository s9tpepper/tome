use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{self, Component, ComponentId, KeyCode},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{State, Value},
    widgets::Elements,
};

use crate::{
    projects::HeaderState,
    theme::{get_app_theme, AppTheme},
};

use super::dashboard::{DashboardMessageHandler, FloatingWindow};

pub const EDIT_HEADER_WINDOW_TEMPLATE: &str = "./src/components/templates/edit_header_window.aml";

#[derive(Default)]
pub struct EditHeaderWindow {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl EditHeaderWindow {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let edit_header_window_id = builder.register_component(
            "edit_header_window",
            EDIT_HEADER_WINDOW_TEMPLATE,
            EditHeaderWindow {
                component_ids: ids.clone(),
            },
            EditHeaderWindowState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("edit_header_window"), edit_header_window_id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut EditHeaderWindowState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }
}

#[derive(Default, State)]
pub struct EditHeaderWindowState {
    name: Value<String>,
    value: Value<String>,
    app_theme: Value<AppTheme>,
}

impl EditHeaderWindowState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        EditHeaderWindowState {
            name: "".to_string().into(),
            value: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

impl DashboardMessageHandler for EditHeaderWindow {
    fn handle_message(
        value: component::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::dashboard::DashboardState,
        mut context: anathema::prelude::Context<'_, super::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        _component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();

        match event.as_str() {
            "edit_header__name_update" => state.edit_header_name.set(value.to_string()),
            "edit_header__value_update" => state.edit_header_value.set(value.to_string()),
            "edit_header__submit" => {
                let header_name = state.edit_header_name.to_ref().to_string();
                let header_value = state.edit_header_value.to_ref().to_string();

                let header = HeaderState {
                    name: header_name.into(),
                    value: header_value.into(),
                };

                state.endpoint.to_mut().headers.push(header);
                state.floating_window.set(FloatingWindow::None);

                context.set_focus("id", "app");
            }
            "edit_header__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                state.edit_header_name.set("".to_string());
                state.edit_header_value.set("".to_string());

                let header = state.header_being_edited.to_mut();
                let header = header.as_ref();
                if let Some(header) = header {
                    state.endpoint.to_mut().headers.push(HeaderState {
                        name: header.to_ref().name.to_ref().clone().into(),
                        value: header.to_ref().value.to_ref().clone().into(),
                    });
                }

                context.set_focus("id", "app");
            }

            _ => {}
        }
    }
}

impl Component for EditHeaderWindow {
    type State = EditHeaderWindowState;
    type Message = String;

    fn message(
        &mut self,
        message: Self::Message,
        _: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match message.as_str() {
            "open" => {
                context.set_focus("id", "edit_header_name_input_id");
            }

            _ => {}
        }
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match ident {
            "edit_header_name_update" => {
                state.name.set(value.to_string());

                context.publish("edit_header__name_update", |state| &state.name)
            }

            "edit_header_value_update" => {
                state.value.set(value.to_string());

                context.publish("edit_header__value_update", |state| &state.value)
            }
            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: component::KeyEvent,
        _state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            KeyCode::Esc => {
                context.publish("edit_header__cancel", |state| &state.name);
            }

            KeyCode::Char(char) => {
                match char {
                    's' => context.publish("edit_header__submit", |state| &state.name),
                    'c' => context.publish("edit_header__cancel", |state| &state.name),

                    // Sets focus to header name text input
                    'n' => context.set_focus("id", "edit_header_name_input_id"),

                    // Sets focus to header value text input
                    'v' => context.set_focus("id", "edit_header_value_input_id"),

                    _ => {}
                }
            }

            _ => {}
        }
    }

    fn accept_focus(&self) -> bool {
        true
    }
}
