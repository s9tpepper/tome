use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId},
    prelude::{Context, ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, State, Value},
    widgets::Elements,
};
use log::info;

use crate::{
    app::GlobalEventHandler,
    messages::focus_messages::FocusChange,
    theme::{get_app_theme, AppTheme},
};

#[derive(Default)]
pub struct FocusableSection {
    #[allow(unused)]
    pub component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl FocusableSection {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
        ident: impl Into<String>,
        template: impl ToSourceKind,
    ) -> anyhow::Result<()> {
        let name: String = ident.into();
        let input_template = template;

        let app_id = builder.register_component(
            name.clone(),
            input_template,
            FocusableSection {
                component_ids: ids.clone(),
            },
            FocusableSectionState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(name, app_id);

        Ok(())
    }

    #[allow(unused)]
    fn update_app_theme(&self, state: &mut FocusableSectionState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);

        state
            .active_border_color
            .set(state.app_theme.to_ref().border_focused.to_ref().to_string());
        state.active_border_color.set(
            state
                .app_theme
                .to_ref()
                .border_unfocused
                .to_ref()
                .to_string(),
        );
    }
}

#[derive(Default, State)]
pub struct FocusableSectionState {
    target: Value<Option<String>>,
    active_border_color: Value<String>,
    app_theme: Value<AppTheme>,
    transient_event_value: Value<String>,
}

impl FocusableSectionState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();
        let unfocused_border_color = app_theme.border_unfocused.to_ref().to_string();
        FocusableSectionState {
            target: None.into(),
            active_border_color: unfocused_border_color.into(),
            app_theme: app_theme.into(),
            transient_event_value: "".to_string().into(),
        }
    }
}

impl Component for FocusableSection {
    type State = FocusableSectionState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        false
    }

    fn tick(
        &mut self,
        state: &mut Self::State,
        _elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
        _dt: std::time::Duration,
    ) {
        if state.target.to_ref().is_some() {
            return;
        }

        let Some(target) = context.get_external("target") else {
            return;
        };

        if let Some(target) = target.to_common() {
            state.target.set(Some(target.to_string()));
        }
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
    ) {
        let focus_message = serde_json::from_str::<FocusChange>(&message);
        match focus_message {
            Ok(focus_msg) => match focus_msg {
                FocusChange::Focused => {
                    state
                        .active_border_color
                        .set(state.app_theme.to_ref().border_focused.to_ref().to_string());
                }
                FocusChange::Unfocused => {
                    state.active_border_color.set(
                        state
                            .app_theme
                            .to_ref()
                            .border_unfocused
                            .to_ref()
                            .to_string(),
                    );
                }
            },

            Err(_) => match message.as_str() {
                "unfocus" => {
                    state.active_border_color.set(
                        state
                            .app_theme
                            .to_ref()
                            .border_unfocused
                            .to_ref()
                            .to_string(),
                    );
                }

                "theme_update" => {
                    self.update_app_theme(state);
                }

                _ => {}
            },
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: CommonVal<'_>,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        if state.target.to_ref().is_none() {
            return;
        }

        // TODO: Use FocusChange direct component message and refactor this focus_change
        // associated function out of this component
        #[allow(clippy::single_match)]
        match ident {
            "focus_change" => {
                let focus = value.to_bool();
                // dbg!(&focus);

                match focus {
                    true => {
                        state
                            .active_border_color
                            .set(state.app_theme.to_ref().border_focused.to_ref().to_string());
                    }
                    false => {
                        state.active_border_color.set(
                            state
                                .app_theme
                                .to_ref()
                                .border_unfocused
                                .to_ref()
                                .to_string(),
                        );
                    }
                }
            }

            _ => {
                info!("Re-publishing event from focusable_section: {ident}");
                state.transient_event_value.set(value.to_string());
                context.publish(ident, |state| &state.transient_event_value);
            }
        }
    }
}
