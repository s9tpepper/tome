use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId, Emitter, KeyCode},
    prelude::{Context, ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    widgets::Elements,
};
use log::info;

use crate::{
    app::GlobalEventHandler, messages::focus_messages::FocusChange, theme::{get_app_theme, get_app_theme_persisted, AppTheme}
};

use super::{
    dashboard::DashboardMessages,
    inputs::{InputReceiver, InputState},
    textinput::TextUpdate,
};

#[derive(Default)]
pub struct EditInput {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    pub listeners: Vec<String>,
    input_for: Option<String>,
    #[allow(dead_code)]
    app_theme: AppTheme,
}

impl EditInput {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
        ident: impl Into<String>,
        template: impl ToSourceKind,
        input_for: Option<String>,
        listeners: Vec<String>,
    ) -> anyhow::Result<()> {
        let name: String = ident.into();
        let app_theme = get_app_theme();
        let state = InputState::new(
            &app_theme.foreground.to_ref(),
            &app_theme.background.to_ref(),
        );

        let app_id = builder.register_component(
            name.clone(),
            template,
            EditInput {
                component_ids: ids.clone(),
                listeners,
                input_for,
                app_theme,
            },
            state,
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(name, app_id);

        Ok(())
    }

    fn send_text_update(&self, state: &mut InputState, emitter: Emitter) {
        if let Ok(ids) = self.component_ids.try_borrow() {
            let input_value = state.input.to_ref().to_string();

            // TODO: Fix this clone weirdness
            let id = self.input_for.clone().unwrap_or("".to_string());

            let input_change_message = DashboardMessages::TextInput(
                super::textinput::TextInputMessages::Update(TextUpdate {
                    id,
                    value: input_value,
                }),
            );

            if let Ok(serialized_message) = serde_json::to_string(&input_change_message) {
                for listener in &self.listeners {
                    let msg = serialized_message.clone();

                    ids.get(listener)
                        .map(|component_id| emitter.emit(*component_id, msg));
                }
            }
        }
    }

    // TODO: Remove the duplication between send_escape and send_text_update()
    fn send_escape(&self, emitter: Emitter) {
        if let Ok(ids) = self.component_ids.try_borrow() {
            // TODO: Fix this clone weirdness
            let id = self.input_for.clone().unwrap_or("".to_string());

            let input_change_message = DashboardMessages::TextInput(
                super::textinput::TextInputMessages::Escape(TextUpdate {
                    id,
                    value: "".to_string(),
                }),
            );

            if let Ok(serialized_message) = serde_json::to_string(&input_change_message) {
                for listener in &self.listeners {
                    let msg = serialized_message.clone();

                    ids.get(listener)
                        .map(|component_id| emitter.emit(*component_id, msg));
                }
            }
        }
    }

    fn send_to_listeners(&self, code: KeyCode, state: &mut InputState, emitter: Emitter) {
        if let KeyCode::Char(_) = code {}
        match code {
            KeyCode::Char(_) => self.send_text_update(state, emitter),
            KeyCode::CtrlC => self.send_text_update(state, emitter),
            KeyCode::Backspace => self.send_text_update(state, emitter),
            KeyCode::Delete => self.send_text_update(state, emitter),
            KeyCode::Esc => self.send_escape(emitter),

            _ => {}
        }
    }

    fn send_focus_to_listeners(&self, state: &mut InputState,  emitter: Emitter) {
        let message = match *state.focused.to_ref() {
            true => serde_json::to_string(&FocusChange::Focused),
            false => serde_json::to_string(&FocusChange::Unfocused),
        };

        if message.is_ok() {
            let Ok(ids) = self.component_ids.try_borrow() else {
                return;
            };

            let message = message.unwrap();
            for listener in &self.listeners {
                ids.get(listener).map(|id| emitter.emit(*id, message.clone()));
            }
        }
    }
}

impl InputReceiver for EditInput {}
impl Component for EditInput {
    type State = InputState;
    type Message = String;

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let emitter = context.emitter.clone();
        self._on_focus(state, elements, context);
        update_theme(self, state);
        self.send_focus_to_listeners(state, emitter);
    }

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let emitter = context.emitter.clone();
        self._on_blur(state, elements, context);
        self.send_focus_to_listeners(state, emitter);
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        self._on_key(&key, state, &elements, &mut context);

        let emitter = context.emitter.clone();
        self.send_to_listeners(key.code, state, emitter);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let emitter = context.emitter.clone();
        self._message(message, state, elements, context);
        self.send_to_listeners(KeyCode::Char(' '), state, emitter);
    }

    fn accept_focus(&self) -> bool {
        true
    }
}

fn update_theme(edit_input: &mut EditInput, state: &mut InputState) {
    info!("updating theme for edit_input()");

    let app_theme = get_app_theme_persisted();

    // TODO: Figure out why this breaks the styling and messaging of the dashboard components
    // println!("{app_theme:?}");
    // state.app_theme.set(app_theme.into());
    // state.app_theme.set(app_theme.into());

    // let mut at = state.app_theme;
    edit_input.app_theme.background.set(app_theme.background);
    edit_input.app_theme.foreground.set(app_theme.foreground);
    edit_input
        .app_theme
        .project_name_background
        .set(app_theme.project_name_background);
    edit_input
        .app_theme
        .project_name_foreground
        .set(app_theme.project_name_foreground);
    edit_input
        .app_theme
        .border_focused
        .set(app_theme.border_focused);
    edit_input
        .app_theme
        .border_unfocused
        .set(app_theme.border_unfocused);
    edit_input
        .app_theme
        .overlay_heading
        .set(app_theme.overlay_heading);
    edit_input
        .app_theme
        .overlay_background
        .set(app_theme.overlay_background);
    edit_input
        .app_theme
        .overlay_foreground
        .set(app_theme.overlay_foreground);
    edit_input
        .app_theme
        .overlay_submit_background
        .set(app_theme.overlay_submit_background);
    edit_input
        .app_theme
        .overlay_submit_foreground
        .set(app_theme.overlay_submit_foreground);

    edit_input
        .app_theme
        .overlay_cancel_background
        .set(app_theme.overlay_cancel_background);
    edit_input
        .app_theme
        .overlay_cancel_foreground
        .set(app_theme.overlay_cancel_foreground);
    edit_input
        .app_theme
        .menu_color_1
        .set(app_theme.menu_color_1);
    edit_input
        .app_theme
        .menu_color_2
        .set(app_theme.menu_color_2);
    edit_input
        .app_theme
        .menu_color_3
        .set(app_theme.menu_color_3);
    edit_input
        .app_theme
        .menu_color_4
        .set(app_theme.menu_color_4);
    edit_input
        .app_theme
        .menu_color_5
        .set(app_theme.menu_color_5);

    edit_input
        .app_theme
        .endpoint_name_background
        .set(app_theme.endpoint_name_background);
    edit_input
        .app_theme
        .endpoint_name_foreground
        .set(app_theme.endpoint_name_foreground);
    edit_input
        .app_theme
        .menu_opt_background
        .set(app_theme.menu_opt_background);
    edit_input
        .app_theme
        .menu_opt_foreground
        .set(app_theme.menu_opt_foreground);
    edit_input
        .app_theme
        .top_bar_background
        .set(app_theme.top_bar_background);
    edit_input
        .app_theme
        .top_bar_foreground
        .set(app_theme.top_bar_foreground);
    edit_input
        .app_theme
        .bottom_bar_background
        .set(app_theme.bottom_bar_background);
    edit_input
        .app_theme
        .bottom_bar_foreground
        .set(app_theme.bottom_bar_foreground);

    state.border_color_focused = edit_input.app_theme.border_focused.to_ref().to_string();
    state.border_color_unfocused = edit_input.app_theme.border_unfocused.to_ref().to_string();

    state
        .border_color
        .set(edit_input.app_theme.border_focused.to_ref().to_string());

    // *state.app_theme.to_mut() = app_theme.into();
}
