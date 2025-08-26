use std::{fmt::Display, str::FromStr};

use anathema::{
    component::{Children, Component, Context, KeyCode},
    state::{State, Value},
};

use crate::{
    components::dashboard::{DashboardMessageHandler, DashboardState},
    theme::{get_app_theme, AppTheme},
};

use super::FloatingWindow;

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
        event: &mut anathema::component::UserEvent<'_>,
        _ident: impl Into<String>,
        state: &mut DashboardState,
        mut context: anathema::component::Context<'_, '_, DashboardState>,
        _children: anathema::component::Children<'_, '_>,
        _component_ids: std::cell::Ref<
            '_,
            std::collections::HashMap<String, anathema::component::ComponentId<String>>,
        >,
    ) {
        let event_name: String = event.name().to_string();

        match event_name.as_str() {
            "body_mode_selector__cancel" => {
                state.floating_window.set(FloatingWindow::None);
            }

            "body_mode_selector__selection" => {
                let value = event.data::<String>();

                match value.as_str() {
                    "Text" | "JavaScript" | "Json" | "Html" | "Xml" => {
                        state.endpoint.to_mut().body_mode.set("raw".to_string());
                        state.endpoint.to_mut().raw_type.set(value.to_string());
                    }

                    _ => {
                        state.endpoint.to_mut().raw_type.set("".to_string());
                        state.endpoint.to_mut().body_mode.set(value.to_string());
                    }
                }

                context.components.by_attribute("id", "app").focus();
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

    fn on_mount(
        &mut self,
        _: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        context.components.by_name("body_mode_selector").focus();
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        mut _children: Children<'_, '_>,
        mut _context: Context<'_, '_, Self::State>,
    ) {
        self.update_app_theme(state);

        // TODO: Highlight current selection
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut Self::State,
        _children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
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

                context.publish(
                    "body_mode_selector__selection",
                    state.selection.to_ref().clone(),
                );
                context.publish(
                    "body_mode_selector__cancel",
                    state.selection.to_ref().clone(),
                );
                context.components.by_attribute("id", "app").focus()
            }

            KeyCode::Esc => {
                context.publish(
                    "body_mode_selector__cancel",
                    state.selection.to_ref().clone(),
                );
                context.components.by_attribute("id", "app").focus()
            }

            _ => (),
        };
    }
}

#[allow(unused)]
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
