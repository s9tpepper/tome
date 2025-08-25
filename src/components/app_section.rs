use anathema::{
    component::{Children, Component, Context, UserEvent},
    state::{State, Value},
};

use crate::theme::{get_app_theme, AppTheme};

#[derive(Default)]
pub struct AppSection;

impl AppSection {
    // TODO: Add a message so the theme can update since this doesn't have focus
    #[allow(unused)]
    fn update_app_theme(&self, state: &mut AppSectionState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }
}

#[derive(Default, State)]
pub struct AppSectionState {
    section_id: Value<String>,
    section_text_id: Value<String>,
    app_theme: Value<AppTheme>,
}

impl AppSectionState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        AppSectionState {
            section_id: Value::new("".into()),
            section_text_id: Value::new("".into()),
            app_theme: app_theme.into(),
        }
    }
}

impl Component for AppSection {
    type State = AppSectionState;
    type Message = ();

    fn on_tick(
        &mut self,
        state: &mut Self::State,
        children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
        _dt: std::time::Duration,
    ) {
        if state.section_id.to_ref().is_empty() {
            let Some(section_id) = context.attribute("section_id") else {
                return;
            };

            let Some(section_text_id) = context.attribute("section_text_id") else {
                return;
            };

            if let Some(id) = section_id.as_str() {
                let id = id.to_string();
                state.section_id.set(id);
            }

            if let Some(section_text_id) = section_text_id.as_str() {
                let text_id = section_text_id.to_string();
                state.section_id.set(text_id);
            }
        }

        self.on_resize(state, children, context);
    }

    fn on_event(
        &mut self,
        event: &mut UserEvent<'_>,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        if event.name() == "focus_change" {
            let focus: bool = *event.data();

            if !focus {
                context.components.by_attribute("id", "app").focus();
            }

            let section_id = state.section_id.to_ref().clone();

            match focus {
                true => children
                    .elements()
                    .by_attribute("id", section_id.as_str())
                    .each(|_element, attributes| {
                        attributes.set("foreground", "#ffff00");
                    }),
                false => children
                    .elements()
                    .by_attribute("id", section_id.as_str())
                    .each(|_element, attributes| {
                        attributes.set("foreground", "#ff0000");
                    }),
            }
        }
    }

    fn accept_focus(&self) -> bool {
        false
    }
}
