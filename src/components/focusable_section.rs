use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId},
    prelude::{ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::{State, Value},
};

use crate::{
    app::GlobalEventHandler,
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
}

impl FocusableSectionState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();
        let unfocused_border_color = app_theme.border_unfocused.to_ref().to_string();
        FocusableSectionState {
            target: None.into(),
            active_border_color: unfocused_border_color.into(),
            app_theme: app_theme.into(),
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
        _elements: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
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
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        match message.as_str() {
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
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        if state.target.to_ref().is_none() {
            return;
        }

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

            _ => {}
        }
    }
}
