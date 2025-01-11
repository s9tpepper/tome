use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId, Emitter, KeyCode},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    widgets::Elements,
};

const TEMPLATE: &str = "./src/components/templates/edit_input.aml";

use crate::theme::{get_app_theme, AppTheme};

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
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        ident: impl Into<String>,
        template: Option<&str>,
        input_for: Option<String>,
        listeners: Vec<String>,
    ) -> anyhow::Result<()> {
        let name: String = ident.into();
        let input_template = template.unwrap_or(TEMPLATE);

        let app_theme = get_app_theme();
        let state = InputState::new(
            &app_theme.foreground.to_ref(),
            &app_theme.background.to_ref(),
        );

        let app_id = builder.register_component(
            name.clone(),
            input_template,
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
        self._on_focus(state, elements, context);
    }

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        self._on_blur(state, elements, context);
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
