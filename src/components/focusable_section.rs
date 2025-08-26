use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    messages::focus_messages::FocusChange,
    theme::{get_app_theme, AppTheme},
};
use anathema::{
    component::{Children, Component, ComponentId, Context},
    prelude::ToSourceKind,
    runtime::Builder,
    state::{Maybe, State, Value},
};

#[derive(Default)]
pub struct FocusableSection {
    #[allow(unused)]
    pub component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl FocusableSection {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
        ident: impl Into<String>,
        template: impl ToSourceKind,
    ) -> anyhow::Result<()> {
        let name: String = ident.into();
        let input_template = template;

        let app_id = builder.component(
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
    target: Value<Maybe<String>>,
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

    fn on_tick(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
        _dt: std::time::Duration,
    ) {
        if state.target.to_ref().get_ref().is_some() {
            return;
        }

        let Some(target) = context.attribute("target") else {
            return;
        };

        if let Some(target) = target.as_str() {
            state
                .target
                .set(Maybe::<String>::from(Some(target.to_string())));
        }
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
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

    fn on_event(
        &mut self,
        event: &mut anathema::component::UserEvent<'_>,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        if state.target.to_ref().get_ref().is_none() {
            return;
        }

        // TODO: Use FocusChange direct component message and refactor this focus_change
        // associated function out of this component
        #[allow(clippy::single_match)]
        match event.name() {
            "focus_change" => {
                let focus = event.data::<bool>();
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
                let value = event.data::<String>();

                state.transient_event_value.set(value.to_string());
                context.publish(event.name(), state.transient_event_value.to_ref().clone());
            }
        }
    }
}
