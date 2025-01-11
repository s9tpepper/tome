use anathema::{
    component::{Component, KeyCode},
    prelude::Context,
    state::{AnyState, Value},
    widgets::Elements,
};

#[derive(Default)]
pub struct TextInput;

#[derive(Default, anathema::state::State)]
pub struct InputState {
    pub input: Value<String>,
    pub cursor_prefix: Value<String>,
    pub cursor_char: Value<String>,
    pub cursor_position: Value<usize>,
    pub fg_color: Value<String>,
    pub bg_color: Value<String>,
    pub focused: Value<bool>,
    pub cursor_selected_fg: Value<String>,
    pub cursor_selected_bg: Value<String>,
    pub cursor_unselected_fg: Value<String>,
    pub cursor_unselected_bg: Value<String>,
}

impl InputState {
    pub fn new(fg_color: &str, bg_color: &str) -> Self {
        InputState {
            input: String::from("").into(),
            cursor_prefix: String::from("").into(),
            cursor_char: String::from("").into(),
            cursor_position: 0.into(),
            fg_color: String::from(fg_color).into(),
            bg_color: String::from(bg_color).into(),
            cursor_selected_fg: String::from(bg_color).into(),
            cursor_selected_bg: String::from(fg_color).into(),
            cursor_unselected_fg: String::from(fg_color).into(),
            cursor_unselected_bg: String::from("").into(),
            focused: false.into(),
        }
    }
}

impl Component for TextInput {
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

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        self._on_key(&key, state, &elements, &mut context);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        self._message(message, state, elements, context);
    }

    fn accept_focus(&self) -> bool {
        true
    }
}

impl InputReceiver for TextInput {}

pub trait InputReceiver {
    #[allow(dead_code)]
    fn _message(
        &mut self,
        message: String,
        state: &mut InputState,
        _: Elements<'_, '_>,
        mut context: Context<'_, InputState>,
    ) {
        let position = message.len();
        state.input.set(message.clone());
        state.cursor_prefix.set(message);
        state.cursor_position.set(position);
        state.cursor_char.set(" ".to_string());

        context.publish("text_change", |state| &state.input);
    }

    #[allow(dead_code)]
    fn _on_focus(
        &mut self,
        state: &mut InputState,
        _: Elements<'_, '_>,
        mut context: Context<'_, InputState>,
    ) {
        let input = state.input.to_ref();
        let Some(cursor_position) = state.cursor_position.to_number() else {
            return;
        };
        let pos = cursor_position.as_uint();

        let cursor_char = if pos == input.len() {
            ' '
        } else {
            input.chars().nth(pos).unwrap_or(' ')
        };

        state.cursor_char.set(cursor_char.to_string());
        state
            .fg_color
            .set(state.cursor_selected_fg.to_ref().to_string());
        state
            .bg_color
            .set(state.cursor_selected_bg.to_ref().to_string());
        state.focused.set(true);

        context.publish("textarea_focus", |state| &state.focused);
    }

    #[allow(dead_code)]
    fn _on_blur(
        &mut self,
        state: &mut InputState,
        _: Elements<'_, '_>,
        mut context: Context<'_, InputState>,
    ) {
        state
            .fg_color
            .set(state.cursor_unselected_fg.to_ref().to_string());
        state
            .bg_color
            .set(state.cursor_unselected_bg.to_ref().to_string());
        state.focused.set(false);

        context.publish("textarea_focus", |state| &state.focused);

        // NOTE: This is causing lots of windows to lose focus for some reason
        // context.set_focus("id", "app");
    }

    #[allow(dead_code)]
    fn _on_key(
        &mut self,
        event: &anathema::component::KeyEvent,
        state: &mut InputState,
        _: &anathema::widgets::Elements<'_, '_>,
        context: &mut anathema::prelude::Context<'_, InputState>,
    ) {
        match event.code {
            // NOTE: Unused for TextInput
            // anathema::component::KeyCode::Home => todo!(),
            // anathema::component::KeyCode::End => todo!(),
            // anathema::component::KeyCode::PageUp => todo!(),
            // anathema::component::KeyCode::PageDown => todo!(),
            // anathema::component::KeyCode::F(_) => todo!(),
            // anathema::component::KeyCode::Null => todo!(),
            // anathema::component::KeyCode::CapsLock => todo!(),
            // anathema::component::KeyCode::ScrollLock => todo!(),
            // anathema::component::KeyCode::NumLock => todo!(),
            // anathema::component::KeyCode::PrintScreen => todo!(),
            // anathema::component::KeyCode::Pause => todo!(),
            // anathema::component::KeyCode::Menu => todo!(),
            // anathema::component::KeyCode::KeypadBegin => todo!(),

            // Text Input events

            // TODO: Ask togglebit Discord if I'm supposed to get this key event
            KeyCode::Tab => {
                let char = '\u{0009}'; // Tab
                self.add_character(char, state, context);
            }

            KeyCode::Char(char) => self.add_character(char, state, context),
            KeyCode::Backspace => self.backspace(state, context),
            KeyCode::Delete => self.delete(state, context),
            KeyCode::Left => self.move_cursor_left(state),
            KeyCode::Right => self.move_cursor_right(state),
            KeyCode::Up => self.move_cursor_up(state),
            KeyCode::Down => self.move_cursor_down(state),

            KeyCode::Enter => {}
            KeyCode::Insert => {}

            // TODO: Ask togglebit Discord if I'm supposed to get this key event
            KeyCode::BackTab => todo!(),

            // TODO: Maybe implement this later, this will require implementing selections in the
            // input
            KeyCode::CtrlC => todo!(),

            // Move focus with this
            KeyCode::Esc => {
                // disable the cursor
                state.cursor_char.set("".to_string());
                state
                    .fg_color
                    .set(state.cursor_selected_fg.to_ref().to_string());
                state
                    .bg_color
                    .set(state.cursor_unselected_bg.to_ref().to_string());
                state.focused.set(false);

                context.publish("escape", |state| &state.focused);
            }

            _ => {}
        }
    }

    fn add_character(
        &mut self,
        char: char,
        state: &mut InputState,
        context: &mut Context<'_, InputState>,
    ) {
        let mut input = state.input.to_mut();
        let Some(cursor_position) = state.cursor_position.to_number() else {
            return;
        };

        // NOTE: Input when cursor is at the far right
        let pos = cursor_position.as_uint();
        input.insert(pos, char);

        let new_position = pos + 1;
        state.cursor_position.set(new_position);

        if pos == input.len() {
            state.cursor_prefix.set(input.to_string());
        } else {
            state
                .cursor_prefix
                .set(input.chars().take(new_position).collect::<String>())
        };

        let cursor_char = if pos == input.len() {
            ' '
        } else {
            input.chars().nth(new_position).unwrap_or(' ')
        };

        state.cursor_char.set(cursor_char.to_string());

        context.publish("text_change", |state| &state.input)
    }

    fn delete(&self, state: &mut InputState, context: &mut Context<'_, InputState>) {
        let mut input = state.input.to_mut();
        let Some(cursor_position) = state.cursor_position.to_number() else {
            return;
        };

        let pos = cursor_position.as_uint();

        if pos == input.len() {
            return;
        }

        input.remove(pos);

        if let Some(cursor_char) = input.chars().nth(pos) {
            state.cursor_char.set(cursor_char.to_string());
        } else {
            state.cursor_char.set(' '.to_string());
        }

        state
            .cursor_prefix
            .set(input.chars().take(pos).collect::<String>());

        context.publish("text_change", |state| &state.input)
    }

    fn backspace(&mut self, state: &mut InputState, context: &mut Context<'_, InputState>) {
        let mut input = state.input.to_mut();
        let Some(cursor_position) = state.cursor_position.to_number() else {
            return;
        };

        let pos = cursor_position.as_uint();

        if pos == 0 {
            return;
        }

        let new_pos = pos - 1;
        input.remove(new_pos);

        state.cursor_position.set(new_pos);

        if let Some(cursor_char) = input.chars().nth(new_pos) {
            state.cursor_char.set(cursor_char.to_string());
        } else {
            state.cursor_char.set(' '.to_string());
        }

        state
            .cursor_prefix
            .set(input.chars().take(new_pos).collect::<String>());

        context.publish("text_change", |state| &state.input)
    }

    fn move_cursor_left(&self, state: &mut InputState) {
        let input = state.input.to_mut();
        let Some(cursor_position) = state.cursor_position.to_number() else {
            return;
        };

        let pos = cursor_position.as_uint();
        if pos == 0 {
            return;
        }

        let new_pos = pos - 1;
        if let Some(new_char) = input.get(0..new_pos) {
            state.cursor_position.set(new_pos);
            state.cursor_prefix.set(new_char.to_string());

            if let Some(cursor_char) = input.to_string().chars().nth(new_pos) {
                state.cursor_char.set(cursor_char.to_string());
            }
        }
    }

    fn move_cursor_right(&self, state: &mut InputState) {
        let input = state.input.to_mut();
        let Some(cursor_position) = state.cursor_position.to_number() else {
            return;
        };

        let pos = cursor_position.as_uint();
        if pos == input.len() {
            return;
        }

        let new_pos = pos + 1;
        if let Some(new_char) = input.get(0..new_pos) {
            state.cursor_position.set(new_pos);
            state.cursor_prefix.set(new_char.to_string());

            if new_pos == input.len() {
                state.cursor_char.set(" ".to_string());
            } else if let Some(cursor_char) = input.to_string().chars().nth(new_pos) {
                state.cursor_char.set(cursor_char.to_string());
            }
        }
    }

    fn move_cursor_up(&self, state: &mut InputState) {
        let input = state.input.to_mut();

        state.cursor_position.set(input.len());
        state.cursor_char.set(' '.to_string());
        state.cursor_prefix.set(input.to_string());
    }

    fn move_cursor_down(&self, state: &mut InputState) {
        let input = state.input.to_mut();

        state.cursor_position.set(0);
        state
            .cursor_char
            .set(input.chars().nth(0).unwrap_or(' ').to_string());
        state.cursor_prefix.set("".to_string());
    }
}
