use std::{fmt::Display, str::FromStr};

use anathema::{
    component::{Component, KeyCode},
    state::{State, Value},
    widgets::Elements,
};

use crate::theme::{get_app_theme, AppTheme};

use super::{dashboard::DashboardMessageHandler, floating_windows::FloatingWindow};

#[derive(Default)]
pub struct MethodSelector;

impl MethodSelector {
    fn update_app_theme(&self, state: &mut MethodSelectorState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }
}

#[derive(Default, State)]
pub struct MethodSelectorState {
    selection: Value<String>,
    app_theme: Value<AppTheme>,
}

impl MethodSelectorState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        MethodSelectorState {
            selection: "GET".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

impl DashboardMessageHandler for MethodSelector {
    fn handle_message(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::dashboard::DashboardState,
        mut context: anathema::prelude::Context<'_, super::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        _component_ids: std::cell::Ref<
            '_,
            std::collections::HashMap<String, anathema::component::ComponentId<String>>,
        >,
    ) {
        let event: String = ident.into();

        match event.as_str() {
            "method_selector__cancel" => {
                state.floating_window.set(FloatingWindow::None);
            }

            "method_selector__new" => {
                let value = &*value.to_common_str();

                state.endpoint.to_mut().method.set(value.to_string());

                // Trigger a resize on the text input by setting focus and then resetting it to app
                context.set_focus("id", "url_input");
                context.set_focus("id", "app");
            }

            _ => {}
        }
    }
}

impl Component for MethodSelector {
    type State = MethodSelectorState;
    type Message = ();

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        mut _elements: anathema::widgets::Elements<'_, '_>,
        mut _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);

        // TODO: Highlight current selection
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match event.code {
            KeyCode::Char(char) => {
                match char.to_uppercase().to_string().as_ref() {
                    "G" => state.selection.set("GET".to_string()),
                    "P" => state.selection.set("POST".to_string()),
                    "U" => state.selection.set("PUT".to_string()),
                    "D" => state.selection.set("DELETE".to_string()),
                    "H" => state.selection.set("HEAD".to_string()),
                    "O" => state.selection.set("OPTIONS".to_string()),
                    "C" => state.selection.set("CUSTOM".to_string()),
                    _ => {
                        // NOTE: Prevents other keys from closing the window
                        return;
                    }
                };

                context.publish("method_selector__new", |state| &state.selection);
                context.publish("method_selector__cancel", |state| &state.selection);
                context.set_focus("id", "app")
            }

            KeyCode::Esc => {
                context.publish("method_selector__cancel", |state| &state.selection);
                context.set_focus("id", "app")
            }

            _ => (),
        };
    }
}

#[derive(Default)]
enum Method {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Connect,
    Custom,
}

#[derive(Debug, PartialEq, Eq)]
struct MethodParseError;

impl FromStr for Method {
    type Err = MethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            "OPTIONS" => Ok(Method::Options),
            "CONNECT" => Ok(Method::Connect),
            _ => Ok(Method::Custom),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Patch => write!(f, "PATCH"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Connect => write!(f, "CONNECT"),
            Method::Custom => write!(f, "CUSTOM"),
        }
    }
}
