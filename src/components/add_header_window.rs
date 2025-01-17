use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{self, Component, ComponentId, KeyCode},
    prelude::{ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::{State, Value},
    widgets::Elements,
};

use crate::{
    projects::HeaderState,
    theme::{get_app_theme, AppTheme},
};

use super::{dashboard::DashboardMessageHandler, floating_windows::FloatingWindow};

pub const ADD_HEADER_WINDOW_TEMPLATE: &str = include_str!("./templates/add_header_window.aml");

#[derive(Default)]
pub struct AddHeaderWindow {
    #[allow(unused)]
    pub component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl AddHeaderWindow {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let name: String = "add_header_window".to_string();

        let app_id = builder.register_component(
            name.clone(),
            ADD_HEADER_WINDOW_TEMPLATE.to_template(),
            AddHeaderWindow {
                component_ids: ids.clone(),
            },
            AddHeaderWindowState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(name, app_id);

        Ok(())
    }
}

impl AddHeaderWindow {
    fn update_app_theme(&self, state: &mut AddHeaderWindowState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }
}

#[derive(Default, State)]
pub struct AddHeaderWindowState {
    name: Value<String>,
    value: Value<String>,
    app_theme: Value<AppTheme>,
}

impl AddHeaderWindowState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        AddHeaderWindowState {
            name: "".to_string().into(),
            value: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

impl DashboardMessageHandler for AddHeaderWindow {
    fn handle_message(
        value: component::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::dashboard::DashboardState,
        mut context: anathema::prelude::Context<'_, super::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        _component_ids: Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();
        match event.as_str() {
            "add_header__name_update" => state.new_header_name.set(value.to_string()),
            "add_header__value_update" => state.new_header_value.set(value.to_string()),
            "add_header__submit" => {
                let header_name = state.new_header_name.to_ref().to_string();
                let header_value = state.new_header_value.to_ref().to_string();

                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");

                if header_name.trim().is_empty() || header_value.trim().is_empty() {
                    return;
                }

                let header = HeaderState {
                    name: header_name.into(),
                    value: header_value.into(),
                };
                state.endpoint.to_mut().headers.push(header);
            }
            "add_header__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                state.new_header_name.set("".to_string());
                state.new_header_value.set("".to_string());
                context.set_focus("id", "app");
            }

            _ => {}
        }
    }
}

impl Component for AddHeaderWindow {
    type State = AddHeaderWindowState;
    type Message = String;

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
        _: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match message.as_str() {
            "open" => {
                context.set_focus("id", "header_name_input");
            }

            _ => {}
        }
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
            "header_name_update" => {
                state.name.set(value.to_string());

                context.publish("add_header__name_update", |state| &state.name)
            }

            "header_value_update" => {
                state.value.set(value.to_string());

                context.publish("add_header__value_update", |state| &state.value)
            }

            "name_input_focus" | "value_input_focus" => {
                context.set_focus("id", "add_header_window");
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
                context.publish("add_header__cancel", |state| &state.name);
            }

            KeyCode::Char(char) => {
                match char {
                    's' => context.publish("add_header__submit", |state| &state.name),

                    'c' => context.publish("add_header__cancel", |state| &state.name),

                    // Sets focus to header name text input
                    'n' => context.set_focus("id", "header_name_input"),

                    // Sets focus to header value text input
                    'v' => context.set_focus("id", "header_value_input"),

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
