use std::{fmt::Display, str::FromStr};

use anathema::{
    component::{Component, KeyCode},
    state::{State, Value},
    widgets::Elements,
};

use crate::{
    components::dashboard::{DashboardMessageHandler, DashboardState, FloatingWindow},
    theme::{get_app_theme, AppTheme},
};

pub const BODY_MODE_SELECTOR_TEMPLATE: &str =
    "./src/components/floating_windows/templates/body_mode_selector.aml";

#[derive(Default)]
pub struct BodyModeSelector;

impl BodyModeSelector {
    fn update_app_theme(&self, state: &mut BodyModeSelectorState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }
}

#[derive(Default, State)]
pub struct BodyModeSelectorState {
    selection: Value<String>,
    app_theme: Value<AppTheme>,
}

impl BodyModeSelectorState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        BodyModeSelectorState {
            selection: "None".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

impl DashboardMessageHandler for BodyModeSelector {
    fn handle_message(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut DashboardState,
        mut context: anathema::prelude::Context<'_, DashboardState>,
        _elements: Elements<'_, '_>,
        _component_ids: std::cell::Ref<
            '_,
            std::collections::HashMap<String, anathema::component::ComponentId<String>>,
        >,
    ) {
        let event: String = ident.into();

        match event.as_str() {
            "body_mode_selector__cancel" => {
                state.floating_window.set(FloatingWindow::None);
            }

            "body_mode_selector__selection" => {
                let value = &*value.to_common_str();

                match value {
                    "Text" | "JavaScript" | "Json" | "Html" | "Xml" => {
                        state.endpoint.to_mut().body_mode.set("raw".to_string());
                        state.endpoint.to_mut().raw_type.set(value.to_string());
                    }

                    _ => {
                        state.endpoint.to_mut().raw_type.set("".to_string());
                        state.endpoint.to_mut().body_mode.set(value.to_string());
                    }
                }

                context.set_focus("id", "app");
            }

            _ => {}
        }
    }
}

impl Component for BodyModeSelector {
    type State = BodyModeSelectorState;
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
                match char.to_string().as_ref() {
                    "t" => state.selection.set("Text".to_string()),
                    "j" => state.selection.set("JavaScript".to_string()),
                    "s" => state.selection.set("Json".to_string()),
                    "h" => state.selection.set("Html".to_string()),
                    "x" => state.selection.set("Xml".to_string()),
                    "f" => state.selection.set("FormData".to_string()),
                    "u" => state.selection.set("UrlEncoded".to_string()),
                    "b" => state.selection.set("Binary".to_string()),
                    "g" => state.selection.set("GraphQL".to_string()),
                    "n" => state.selection.set("None".to_string()),

                    _ => {
                        // NOTE: Prevents other keys from closing the window
                        return;
                    }
                };

                context.publish("body_mode_selector__selection", |state| &state.selection);
                context.publish("body_mode_selector__cancel", |state| &state.selection);
                context.set_focus("id", "app")
            }

            KeyCode::Esc => {
                context.publish("body_mode_selector__cancel", |state| &state.selection);
                context.set_focus("id", "app")
            }

            _ => (),
        };
    }
}

#[derive(Default)]
pub enum BodyMode {
    #[default]
    None,
    Text,
    JavaScript,
    Json,
    Html,
    Xml,
    FormData,
    UrlEncoded,
    Binary,
    GraphQL,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BodyModeParseError;

impl FromStr for BodyMode {
    type Err = BodyModeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" => Ok(BodyMode::None),
            "Text" => Ok(BodyMode::Text),
            "JavaScript" => Ok(BodyMode::JavaScript),
            "Json" => Ok(BodyMode::Json),
            "Html" => Ok(BodyMode::Html),
            "Xml" => Ok(BodyMode::Xml),
            "FormData" => Ok(BodyMode::FormData),
            "UrlEncoded" => Ok(BodyMode::UrlEncoded),
            "Binary" => Ok(BodyMode::Binary),
            "GraphQL" => Ok(BodyMode::GraphQL),

            _ => Ok(BodyMode::None),
        }
    }
}

impl Display for BodyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyMode::None => write!(f, "None"),
            BodyMode::Text => write!(f, "Text"),
            BodyMode::JavaScript => write!(f, "JavaScript"),
            BodyMode::Json => write!(f, "Json"),
            BodyMode::Html => write!(f, "Html"),
            BodyMode::Xml => write!(f, "Xml"),
            BodyMode::FormData => write!(f, "FormData"),
            BodyMode::UrlEncoded => write!(f, "UrlEncoded"),
            BodyMode::Binary => write!(f, "Binary"),
            BodyMode::GraphQL => write!(f, "GraphQL"),
        }
    }
}
