use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Iterator;
use std::rc::Rc;
use std::{io::Write, str::Chars};

use anathema::component::{ComponentId, Emitter, KeyCode};
use anathema::prelude::{ToSourceKind, TuiBackend};
use anathema::runtime::RuntimeBuilder;
use anathema::{
    default_widgets::{Overflow, Text},
    geometry::Pos,
    prelude::Context,
    state::{Number, State, Value},
    widgets::{
        layout::text::{Line, Segment},
        Elements,
    },
};
use arboard::Clipboard;
use log::info;
use serde::{Deserialize, Serialize};

use crate::app::GlobalEventHandler;
use crate::theme::{get_app_theme, AppTheme};

use super::dashboard::DashboardMessages;

#[derive(Default)]
pub struct TextArea {
    #[allow(unused)]
    pub input_for: Option<String>,
    pub listeners: Vec<String>,
    pub component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    #[allow(unused)]
    pub app_theme: AppTheme,
}

#[derive(Default, anathema::state::State)]
pub struct TextAreaInputState {
    log_output: Value<String>,
    input: Value<String>,
    cursor_prefix: Value<String>,
    cursor_char: Value<String>,
    cursor_position: Value<Coordinates>,
    line_count: Value<usize>,
    text_color: Value<String>,
    fg_color: Value<String>,
    bg_color: Value<String>,
    cursor_selected_fg: Value<String>,
    cursor_selected_bg: Value<String>,
    cursor_unselected_fg: Value<String>,
    cursor_unselected_bg: Value<String>,
    focused: Value<bool>,
    width: Value<usize>,
    height: Value<usize>,
    scroll_position: Value<usize>,
}

#[derive(Default, anathema::state::State)]
struct Coordinates {
    x: Value<usize>,
    y: Value<usize>,
}

impl Coordinates {
    pub fn new(x_pos: usize, y_pos: usize) -> Self {
        Coordinates {
            x: x_pos.into(),
            y: y_pos.into(),
        }
    }
}

impl TextAreaInputState {
    pub fn new(fg_color: &str, bg_color: &str) -> Self {
        TextAreaInputState {
            input: String::from("").into(),
            cursor_prefix: String::from("").into(),
            cursor_char: String::from("").into(),
            cursor_position: Coordinates::new(0, 0).into(),
            line_count: 1.into(),
            text_color: String::from(fg_color).into(),
            fg_color: String::from(fg_color).into(),
            bg_color: String::from("").into(),
            cursor_selected_fg: String::from(bg_color).into(),
            cursor_selected_bg: String::from(fg_color).into(),
            cursor_unselected_fg: String::from(fg_color).into(),
            cursor_unselected_bg: String::from("").into(),
            focused: false.into(),
            log_output: String::from("").into(),
            width: 0.into(),
            height: 0.into(),
            scroll_position: 0.into(),
        }
    }
}

enum ScrollDirection {
    Up,
    Down,
}

// NOTE: keep around for future, maybe
// fn scroll_to_line(
//     state: &mut TextAreaInputState,
//     mut elements: Elements<'_, '_>,
//     _: Context<'_, TextAreaInputState>,
//     line: usize,
// ) {
//     elements
//         .by_attribute("id", "container")
//         .each(|el, _attributes| {
//             let overflow = el.to::<Overflow>();
//
//             state.scroll_position.set(line);
//
//             let pos = Pos {
//                 x: 0,
//                 y: line as i32,
//             };
//
//             overflow.scroll_to(pos);
//         });
// }

fn scroll(
    state: &mut TextAreaInputState,
    mut elements: Elements<'_, '_>,
    context: Context<'_, TextAreaInputState>,
    direction: ScrollDirection,
) {
    elements
        .by_attribute("id", "container")
        .each(|el, _attributes| {
            let overflow = el.to::<Overflow>();

            let scroll_amount = context.viewport.size().height / 2;
            let scroll_position = *state.scroll_position.to_ref();
            let new_scroll_position = match direction {
                ScrollDirection::Up => scroll_position.saturating_sub(scroll_amount),
                ScrollDirection::Down => scroll_position + scroll_amount,
            };

            state.scroll_position.set(new_scroll_position);

            let pos = Pos {
                x: 0,
                y: new_scroll_position as i32,
            };

            overflow.scroll_to(pos);
        });
}

impl anathema::component::Component for TextArea {
    type State = TextAreaInputState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
    ) {
        if let Ok(deserialized_msg) = serde_json::from_str::<TextAreaMessages>(&message) {
            #[allow(clippy::single_match)]
            match deserialized_msg {
                TextAreaMessages::SetInput(value) => {
                    state.input.set(value);
                }

                TextAreaMessages::InputChange(_) => {}
            }
        }
    }

    fn resize(
        &mut self,
        state: &mut Self::State,
        _elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        if let Some(color) = context.get_external("color") {
            state.text_color.set(color.to_common().unwrap().to_string());
        };
    }

    fn tick(
        &mut self,
        state: &mut Self::State,
        mut _elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
        _dt: std::time::Duration,
    ) {
        if let Some(output) = context.get_external("output") {
            state.input.set(output.to_common().unwrap().to_string());
        };

        if let Some(color) = context.get_external("color") {
            state.text_color.set(color.to_common().unwrap().to_string());
        };
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        state
            .fg_color
            .set(state.cursor_selected_fg.to_ref().to_string());
        state
            .bg_color
            .set(state.cursor_selected_bg.to_ref().to_string());
        state.focused.set(true);

        context.publish("textarea_focus", |state| &state.focused);
    }

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        state.cursor_char.set("".to_string());
        state
            .fg_color
            .set(state.cursor_unselected_fg.to_ref().to_string());
        state
            .bg_color
            .set(state.cursor_unselected_bg.to_ref().to_string());
        state.focused.set(false);

        context.publish("textarea_focus", |state| &state.focused);
        context.set_focus("id", "app");
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
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
            anathema::component::KeyCode::Tab => {
                info!("Got tab key code");

                // NOTE: These escape sequences won't render
                // let char = '\u{0009}'; // Tab
                // let char = '\u{000b}'; // Tab
                // let char = '\t'; // Tab
                // self.add_character(char, state, &mut context, &mut elements, event);

                // NOTE: This is a HACK, the two Tab characters above do not work
                self.add_character(' ', state, &mut context, &mut elements, event);
                self.add_character(' ', state, &mut context, &mut elements, event);
            }

            anathema::component::KeyCode::Char(char) => match event.ctrl {
                true => match char {
                    'd' => scroll(state, elements, context, ScrollDirection::Down),
                    'u' => scroll(state, elements, context, ScrollDirection::Up),
                    _ => {}
                },

                false => {
                    let emitter = context.emitter.clone();
                    self.add_character(char, state, &mut context, &mut elements, event);
                    self.send_to_listeners(event.code, state, emitter);
                }
            },
            anathema::component::KeyCode::Backspace => self.backspace(state, context, elements),
            anathema::component::KeyCode::Delete => self.delete(state, context),
            anathema::component::KeyCode::Left => self.move_cursor_left(state, elements),
            anathema::component::KeyCode::Right => self.move_cursor_right(state, elements),
            anathema::component::KeyCode::Up => self.move_cursor_up(state, elements),
            anathema::component::KeyCode::Down => self.move_cursor_down(state, elements),

            // TODO: This will need to call some callback or something?
            anathema::component::KeyCode::Enter => {
                let emitter = context.emitter.clone();
                let char = '\u{000A}';
                self.add_character(char, state, &mut context, &mut elements, event);
                self.send_to_listeners(event.code, state, emitter);
            }

            // TODO: Maybe I'll implement this later
            anathema::component::KeyCode::Insert => todo!(),

            // TODO: Ask togglebit Discord if I'm supposed to get this key event
            anathema::component::KeyCode::BackTab => todo!(),

            // TODO: Maybe implement this later, this will require implementing selections in the
            // input
            anathema::component::KeyCode::CtrlC => todo!(),

            // Move focus with this
            anathema::component::KeyCode::Esc => {
                context.set_focus("id", "app");

                context.publish("textarea_focus", |state| &state.focused);
            }

            _ => {}
        }
    }
}

fn log(output: String, _file: Option<&str>) {
    info!("{output}");

    // let file_name = file.unwrap_or("logs.txt");
    // let file = std::fs::OpenOptions::new().append(true).open(file_name);
    // if let Ok(mut file) = file {
    //     let _ = file.write(output.as_bytes());
    // }
}

fn update_cursor_char(input: &mut Chars, update_index: usize) -> String {
    if let Some(cursor_char) = input.nth(update_index) {
        cursor_char.to_string()
    } else {
        " ".to_string()
    }
}

fn get_line_lengths<'textarea>(
    lines: impl Iterator<Item = Line<impl Iterator<Item = Segment<'textarea>>>>,
) -> Vec<usize> {
    let mut line_lengths: Vec<usize> = [].to_vec();
    lines.for_each(|current_line| {
        let mut length_of_line = 0;
        current_line.entries.for_each(|entry| {
            if let Segment::Str(text) = entry {
                let chunk_length = text.len();

                length_of_line += chunk_length;
            };
        });

        line_lengths.push(length_of_line);
    });

    // Account for newline characters, but remove one because the last line doesn't
    // have a newline character
    line_lengths.push(line_lengths.len() - 1);

    line_lengths
}

#[derive(Debug, PartialEq, Eq)]
struct CursorData {
    x: usize,
    y: usize,
    cursor_index: usize,
    cursor_prefix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TextAreaMessages {
    InputChange(String),
    SetInput(String),
}

impl TextArea {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
        ident: String,
        tpl: impl ToSourceKind,
        input_for: Option<String>,
        listeners: Vec<String>,
    ) -> anyhow::Result<()> {
        let name: String = ident;

        let app_theme = get_app_theme();
        let state = TextAreaInputState::new(
            &app_theme.foreground.to_ref(),
            &app_theme.background.to_ref(),
        );

        let app_id = builder.register_component(
            name.clone(),
            tpl,
            TextArea {
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

    fn send_to_listeners(&self, code: KeyCode, state: &mut TextAreaInputState, emitter: Emitter) {
        if let KeyCode::Char(_) = code {
            // TODO: Fix the outgoing message so it is not 100% coupled to only response body
            // editing, use InputUpdate instead of InputChange, like in edit_input.rs
            if let Ok(ids) = self.component_ids.try_borrow() {
                let input_value = state.input.to_ref().to_string();
                let input_change_message =
                    DashboardMessages::TextArea(TextAreaMessages::InputChange(input_value));

                if let Ok(serialized_message) = serde_json::to_string(&input_change_message) {
                    for listener in &self.listeners {
                        let msg = serialized_message.clone();

                        ids.get(listener)
                            .map(|component_id| emitter.emit(*component_id, msg));
                    }
                }
            }
        }
    }

    fn add_character(
        &mut self,
        char: char,
        state: &mut TextAreaInputState,
        context: &mut Context<'_, TextAreaInputState>,
        elements: &mut anathema::widgets::Elements<'_, '_>,
        event: anathema::component::KeyEvent,
    ) {
        let mut input = state.input.to_mut();

        elements
            .by_attribute("id", "contents")
            .each(|el, _attributes| {
                let text = el.to::<Text>();
                let editable = context.get_external("editable");
                if let Some(editable) = editable {
                    let is_editable = editable.load_bool();
                    if !is_editable {
                        return;
                    }
                }

                if event.ctrl && char == 'c' {
                    // TODO: Add support for pasting at cursor position, replaces for now
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Ok(text) = clipboard.get_text() {
                            *input = text;

                            return;
                        };
                    }
                }

                // Update line count
                let current_line_count = text.get_line_count();
                let prev_line_count = state
                    .line_count
                    .to_number()
                    .unwrap_or(Number::Usize(0))
                    .as_uint();
                state.line_count.set(current_line_count);

                // Get coordinates before this character input
                let mut cursor_coordinates = state.cursor_position.to_mut();
                let prev_cursor_x = cursor_coordinates
                    .x
                    .to_number()
                    .unwrap_or(Number::Usize(0))
                    .as_uint();
                let prev_cursor_y = cursor_coordinates
                    .y
                    .to_number()
                    .unwrap_or(Number::Usize(0))
                    .as_uint();

                log(format!("prev_x: {prev_cursor_x}\n"), None);
                log(format!("prev_y: {prev_cursor_y}\n"), None);

                // Calculate new cursor X/Y position
                let new_cursor_x = if current_line_count > prev_line_count {
                    Number::Usize(0).as_uint()
                } else {
                    Number::Usize(prev_cursor_x + 1).as_uint()
                };
                cursor_coordinates.x.set(new_cursor_x);

                let new_cursor_y = if current_line_count > prev_line_count {
                    Number::Usize(prev_cursor_y + 1).as_uint()
                } else {
                    Number::Usize(prev_cursor_y).as_uint()
                };
                cursor_coordinates.y.set(new_cursor_y);

                log(format!("new_x: {new_cursor_x}\n"), None);
                log(format!("new_y: {new_cursor_y}\n"), None);

                // Get line lengths for all lines in input
                let lines = text.get_lines();
                let mut line_lengths: Vec<usize> = [].to_vec();

                lines.enumerate().for_each(|(index, current_line)| {
                    log(format!("Looking at line_number: {index}\n"), None);
                    log(
                        format!("Length of line {index}: {}\n", current_line.width),
                        None,
                    );

                    line_lengths.push(current_line.width.into());
                });

                // Account for newline characters, but remove one because the last line doesn't
                // have a newline character
                line_lengths.push(line_lengths.len().saturating_sub(1));

                log(format!("Length of all lines: {}\n", line_lengths.iter().sum::<usize>()), None);
                log(format!("Length of line_lengths array: {}\n", line_lengths.len()), None);

                // Sets update index to the end of the input string
                let mut update_index = line_lengths.iter().sum::<usize>();
                log(format!("Initial update_index: {update_index}\n"), None);


                if !is_at_end_of_input(&line_lengths, prev_cursor_x, prev_cursor_y) {
                    log(format!("get_update_index(): {line_lengths:?}, {prev_cursor_x}, {prev_cursor_y}\n"), None);
                    log(format!("get_update_index() line_lengths.len(): {}\n", line_lengths.len()), None);
                    update_index = get_update_index(&line_lengths, prev_cursor_x, prev_cursor_y);
                }

                log(format!("Final update_index: {update_index}\n"), None);
                log(format!("input.len(): {}\n", input.len()), None);
                log(format!(
                    "Inserting at index: {update_index} character: '{char}', current input length: {} \n", input.len()
                ), None);
                log(
                    "----------------------------------------\n".to_string(), None
                );

                // Insert new character
                // TODO: Remove this hack when I find the bug
                if update_index > input.len() {
                    update_index = input.len();
                }

                input.insert(update_index, char);

                // Update text prefix
                let prefix_text = input.chars().take(update_index + 1).collect::<String>();
                state.cursor_prefix.set(prefix_text);

                let mut chars = input.chars();
                let cursor_char = update_cursor_char(&mut chars, update_index + 1);
                state.cursor_char.set(cursor_char);
            });

        elements
            .by_attribute("id", "container")
            .each(|el, _attributes| {
                let coordinates = state.cursor_position.to_ref();
                let x = coordinates.x.to_number().unwrap_or(Number::I32(0)).as_int() as i32;
                let y = coordinates.y.to_number().unwrap_or(Number::I32(0)).as_int() as i32;

                let position = Pos { x, y };
                let overflow = el.to::<Overflow>();
                overflow.scroll_to(position);
            });

        context.publish("text_change", |state| &state.input)
    }

    fn delete(&self, _state: &mut TextAreaInputState, _context: Context<'_, TextAreaInputState>) {
        // let mut input = state.input.to_mut();
        // let Some(cursor_position) = state.cursor_position.to_number() else {
        //     return;
        // };
        //
        // let pos = cursor_position.as_uint();
        //
        // if pos == input.len() {
        //     return;
        // }
        //
        // input.remove(pos);
        //
        // if let Some(cursor_char) = input.chars().nth(pos) {
        //     state.cursor_char.set(cursor_char.to_string());
        // } else {
        //     state.cursor_char.set(' '.to_string());
        // }
        //
        // state
        //     .cursor_prefix
        //     .set(input.chars().take(pos).collect::<String>());
        //
        // context.publish("text_change", |state| &state.input)
    }

    fn backspace(
        &mut self,
        state: &mut TextAreaInputState,
        mut context: Context<'_, TextAreaInputState>,
        mut elements: anathema::widgets::Elements<'_, '_>,
    ) {
        elements
            .by_attribute("id", "contents")
            .each(|el, _attributes| {
                log(
                    "----------------------------------------\n".to_string(),
                    Some("backspace.txt"),
                );
                let text = el.to::<Text>();
                let lines = text.get_lines();

                let prev_cursor_x = *state.cursor_position.to_ref().x.to_ref();
                let prev_cursor_y = *state.cursor_position.to_ref().y.to_ref();
                let line_lengths = get_line_lengths(lines);

                let mut input = state.input.to_mut();
                let mut backspace_index = input.len().saturating_sub(1);
                log(
                    format!("backspace_index: {backspace_index}\n"),
                    Some("backspace.txt"),
                );

                if !is_at_end_of_input(&line_lengths, prev_cursor_x, prev_cursor_y) {
                    backspace_index = get_update_index(&line_lengths, prev_cursor_x, prev_cursor_y)
                        .saturating_sub(1);
                }
                log(
                    format!(">backspace_index: {backspace_index}\n"),
                    Some("backspace.txt"),
                );
                log(
                    format!("input.len(): {}\n", input.len()),
                    Some("backspace.txt"),
                );

                if backspace_index < input.len() {
                    log(
                        format!("Deleting backspace_index: {backspace_index}\n"),
                        Some("backspace.txt"),
                    );
                    input.remove(backspace_index);
                    context.publish("text_change", |state| &state.input);

                    // let mut prefix = state.cursor_prefix.to_mut();
                    // prefix.remove(backspace_index);

                    let chars = input.chars();
                    let Some(previous_line) = text.get_lines().nth(prev_cursor_y.saturating_sub(1))
                    else {
                        return;
                    };

                    let prev_cursor_index =
                        self.get_cursor_index(prev_cursor_x, prev_cursor_y, text.get_lines());
                    let new_cursor_data = get_cursor_data_left(
                        chars,
                        prev_cursor_x,
                        prev_cursor_y,
                        previous_line.width.into(),
                        prev_cursor_index,
                    );
                    let char = input
                        .chars()
                        .nth(new_cursor_data.cursor_index)
                        .unwrap_or(' ');

                    state.cursor_position.to_mut().x.set(new_cursor_data.x);
                    state.cursor_position.to_mut().y.set(new_cursor_data.y);
                    state.cursor_prefix.set(new_cursor_data.cursor_prefix);
                    state.cursor_char.set(char.to_string());
                }

                log(
                    ">>>----------------------------------------\n".to_string(),
                    Some("backspace.txt"),
                );
            });
    }

    fn get_cursor_index<'a>(
        &self,
        x: usize,
        y: usize,
        lines: impl Iterator<Item = Line<impl Iterator<Item = Segment<'a>>>>,
    ) -> usize {
        let mut previous_lines_width: usize = 0;

        lines
            .take(y)
            .for_each(|line| previous_lines_width += line.width as usize);

        previous_lines_width + y + x
    }

    fn move_cursor_left(
        &self,
        state: &mut TextAreaInputState,
        mut elements: anathema::widgets::Elements<'_, '_>,
    ) {
        // TODO: Refactor this so its not repeated when moving right
        let cursor_index = state.cursor_prefix.to_ref().len();
        let new_cursor_index = cursor_index.saturating_sub(1);
        let current_input = state.input.to_ref();
        let new_prefix = current_input.chars().take(new_cursor_index);
        state.cursor_prefix.set(new_prefix.collect::<String>());

        let input = state.input.to_ref();
        let mut chars = input.chars();
        let cursor_char = update_cursor_char(&mut chars, new_cursor_index);
        state.cursor_char.set(cursor_char);

        // Update cursor x/y position
        let mut coordinates = state.cursor_position.to_mut();
        let mut x = *coordinates.x.to_ref();
        let mut y = *coordinates.y.to_ref();

        if x > 0 {
            coordinates.x.set(x - 1);
        } else {
            elements
                .by_attribute("id", "contents")
                .each(|el, _attributes| {
                    let text = el.to::<Text>();
                    let mut lines = text.get_lines();

                    y = y.saturating_sub(1);
                    if let Some(previous_line) = lines.nth(y) {
                        x = previous_line.width as usize + 1;
                    }
                });

            coordinates.x.set(x);
            coordinates.y.set(y);
        }
    }

    fn move_cursor_right(
        &self,
        state: &mut TextAreaInputState,
        mut elements: anathema::widgets::Elements<'_, '_>,
    ) {
        elements
            .by_attribute("id", "contents")
            .each(|el, _attributes| {
                // TODO: Fix this clone
                let prefix = state.cursor_prefix.to_ref().clone();
                let input = state.input.to_ref();
                if prefix.len() == input.len() {
                    // At the end, can't move to the right
                    return;
                }

                let text = el.to::<Text>();
                let mut lines = text.get_lines();

                let mut coordinates = state.cursor_position.to_mut();
                let x = *coordinates.x.to_ref();
                let y = *coordinates.y.to_ref();

                let Some(current_line) = lines.nth(y) else {
                    return;
                };

                let current_input = state.input.to_ref();
                let last_current_line_x = current_line.width + 1;

                match x == Into::<usize>::into(last_current_line_x) {
                    true => {
                        coordinates.x.set(0);
                        coordinates.y.set(y + 1);
                    }
                    false => {
                        coordinates.x.set(x + 1);
                    }
                }

                let cursor_index = prefix.len();
                let new_cursor_index = cursor_index + 1;
                let new_prefix = current_input.chars().take(new_cursor_index);
                state.cursor_prefix.set(new_prefix.collect::<String>());

                // let input = state.input.to_ref();
                let mut chars = current_input.chars();
                let cursor_char = update_cursor_char(&mut chars, new_cursor_index);
                state.cursor_char.set(cursor_char);

                log(
                    format!(
                        "x: {}, y: {}\n",
                        *coordinates.x.to_ref(),
                        *coordinates.y.to_ref()
                    ),
                    Some("move_right.txt"),
                );
            });
    }

    fn move_cursor_up(
        &self,
        state: &mut TextAreaInputState,
        mut elements: anathema::widgets::Elements<'_, '_>,
    ) {
        elements
            .by_attribute("id", "contents")
            .each(|el, _attributes| {
                let text = el.to::<Text>();
                cursor_up(text, state);
            })
    }

    fn move_cursor_down(
        &self,
        state: &mut TextAreaInputState,
        mut elements: anathema::widgets::Elements<'_, '_>,
    ) {
        elements
            .by_attribute("id", "contents")
            .each(|el, _attributes| {
                let text = el.to::<Text>();
                cursor_down(text, state);
            })
    }
}

fn get_cursor_data_left(
    chars: Chars,
    prev_x: usize,
    prev_y: usize,
    prev_line_width: usize,
    prev_cursor_index: usize,
) -> CursorData {
    let cursor_index = prev_cursor_index.saturating_sub(1);
    let cursor_prefix = chars.take(cursor_index).collect::<String>();

    let mut x = prev_line_width + 1;
    let mut y = prev_y.saturating_sub(1);

    if prev_x > 0 {
        x = prev_x - 1;
        y = prev_y;
    }

    CursorData {
        x,
        y,
        cursor_index,
        cursor_prefix,
    }
}

#[test]
fn test_get_cursor_data_left_1() {
    // Cursor on 'g' test

    let chars = "ab\ncde\nfg".chars();

    let cursor_data = get_cursor_data_left(chars, 1, 2, 3, 8);

    assert_eq!(
        cursor_data,
        CursorData {
            x: 0,
            y: 2,
            cursor_index: 7,
            cursor_prefix: "ab\ncde\n".to_string()
        }
    )
}

#[test]
fn test_get_cursor_data_left_2() {
    // Cursor after 'g' test

    let chars = "ab\ncde\nfg".chars();

    let cursor_data = get_cursor_data_left(chars, 2, 2, 3, 9);

    assert_eq!(
        cursor_data,
        CursorData {
            x: 1,
            y: 2,
            cursor_index: 8,
            cursor_prefix: "ab\ncde\nf".to_string()
        }
    )
}

#[test]
fn test_get_cursor_data_left_3() {
    // Cursor after g on single line text
    let chars = "abcdefg".chars();

    let cursor_data = get_cursor_data_left(chars, 7, 0, 0, 7);

    assert_eq!(
        cursor_data,
        CursorData {
            x: 6,
            y: 0,
            cursor_index: 6,
            cursor_prefix: "abcdef".to_string()
        }
    )
}

#[test]
fn test_get_cursor_data_left_4() {
    // Cursor on e on single line text
    let chars = "abcdefg".chars();

    let cursor_data = get_cursor_data_left(chars, 4, 0, 0, 4);

    assert_eq!(
        cursor_data,
        CursorData {
            x: 3,
            y: 0,
            cursor_index: 3,
            cursor_prefix: "abc".to_string()
        }
    )
}

fn get_cursor_up_x_y(x: usize, y: usize, target_line_width: u16) -> Coordinates {
    let last_target_line_index = target_line_width;
    let target_x_position = if x <= last_target_line_index.into() {
        x
    } else {
        last_target_line_index.into()
    };

    let target_y_position = y - 1;

    Coordinates::new(target_x_position, target_y_position)
}

fn cursor_up(text: &mut Text, state: &mut TextAreaInputState) {
    let mut coordinates = state.cursor_position.to_mut();
    let x = *coordinates.x.to_ref();
    let y = *coordinates.y.to_ref();

    log(
        format!(
            "x: {}, y: {}\n",
            *coordinates.x.to_ref(),
            *coordinates.y.to_ref()
        ),
        Some("move_up.txt"),
    );

    if y == 0 {
        return;
    }

    let mut lines = text.get_lines();

    let Some(target_line) = lines.nth(y - 1) else {
        log("Couldnt get target line\n".to_string(), Some("move_up.txt"));
        return;
    };

    let Coordinates { x, y } = get_cursor_up_x_y(x, y, target_line.width);
    let target_x_position = *x.to_ref();
    let target_y_position = *y.to_ref();
    log(
        format!("tx: {target_x_position}, ty: {target_y_position}\n"),
        Some("move_up.txt"),
    );

    coordinates.x.set(target_x_position);
    coordinates.y.set(target_y_position);

    log(
        format!("target_y_position: {target_y_position}\n"),
        Some("move_up.txt"),
    );
    let prefix_lines = text.get_lines().take(target_y_position + 1);
    log(
        format!("prefix_lines length: {}\n", prefix_lines.count()),
        Some("move_up.txt"),
    );

    let mut cursor_index = 0;
    let prefix_lines = text.get_lines().take(target_y_position + 1);
    prefix_lines.enumerate().for_each(|(index, line)| {
        if index == target_y_position {
            cursor_index += target_x_position;
            log(
                format!(
                    "after adding x pos -> {target_x_position}: {}\n",
                    cursor_index
                ),
                Some("move_up.txt"),
            );
        } else {
            cursor_index += (line.width + 1) as usize;
            log(
                format!("after adding line width: {}\n", cursor_index),
                Some("move_up.txt"),
            );
        }
    });
    log(
        format!("cursor_index: {cursor_index}\n"),
        Some("move_up.txt"),
    );

    // TODO: Refactor this so its not repeated everywhere
    let current_input = state.input.to_ref();

    let new_prefix = current_input.chars().take(cursor_index);
    state.cursor_prefix.set(new_prefix.collect::<String>());

    let mut chars = current_input.chars();
    let cursor_char = update_cursor_char(&mut chars, cursor_index);
    state.cursor_char.set(cursor_char);
}

fn get_cursor_down_x_y(x: usize, y: usize, target_line_width: u16) -> Coordinates {
    let last_target_line_index = target_line_width;
    let target_x_position = if x <= last_target_line_index.into() {
        x
    } else {
        last_target_line_index.into()
    };

    let target_y_position = y + 1;

    Coordinates::new(target_x_position, target_y_position)
}

fn cursor_down(text: &mut Text, state: &mut TextAreaInputState) {
    let mut lines = text.get_lines();
    let mut coordinates = state.cursor_position.to_mut();
    let x = *coordinates.x.to_ref();
    let y = *coordinates.y.to_ref();

    let line_count = *state.line_count.to_ref();
    let last_line_index = line_count - 1;

    // Already at last line, can't go any lower
    if y == last_line_index {
        return;
    }

    let Some(target_line) = lines.nth(y + 1) else {
        log(
            "Couldnt get target line\n".to_string(),
            Some("move_down.txt"),
        );
        return;
    };

    let Coordinates { x, y } = get_cursor_down_x_y(x, y, target_line.width);
    let target_x_position = *x.to_ref();
    let target_y_position = *y.to_ref();
    log(
        format!("tx: {target_x_position}, ty: {target_y_position}\n"),
        Some("move_down.txt"),
    );

    coordinates.x.set(target_x_position);
    coordinates.y.set(target_y_position);

    log(
        format!("target_y_position: {target_y_position}\n"),
        Some("move_down.txt"),
    );

    let mut cursor_index = 0;
    let prefix_lines = text.get_lines().take(target_y_position + 1);
    prefix_lines.enumerate().for_each(|(index, line)| {
        if index == target_y_position {
            cursor_index += target_x_position;
            log(
                format!(
                    "after adding x pos -> {target_x_position}: {}\n",
                    cursor_index
                ),
                Some("move_down.txt"),
            );
        } else {
            cursor_index += (line.width + 1) as usize;
            log(
                format!("after adding line width: {}\n", cursor_index),
                Some("move_down.txt"),
            );
        }
    });
    log(
        format!("cursor_index: {cursor_index}\n"),
        Some("move_down.txt"),
    );

    // TODO: Refactor this so its not repeated everywhere
    let current_input = state.input.to_ref();

    let new_prefix = current_input.chars().take(cursor_index);
    state.cursor_prefix.set(new_prefix.collect::<String>());

    let mut chars = current_input.chars();
    let cursor_char = update_cursor_char(&mut chars, cursor_index);
    state.cursor_char.set(cursor_char);
}

fn is_at_end_of_input(line_lengths: &[usize], x: usize, y: usize) -> bool {
    // Subtract 2 from line_lengths because the last entry is a count of newlines
    let on_last_line = y == line_lengths.len().saturating_sub(2);
    if !on_last_line {
        return false;
    }

    // Uses nth_back(1) to skip the final entry which is a count of newlines
    let last_line_length = *line_lengths.iter().nth_back(1).unwrap_or(&usize::MIN);

    last_line_length.saturating_sub(1) == x
}

fn get_update_index(line_lengths: &[usize], x: usize, y: usize) -> usize {
    let previous_lines = line_lengths.iter().take(y);
    let newlines = previous_lines.len();
    let previous_lines_sum = previous_lines.sum::<usize>();

    previous_lines_sum + newlines + x
}

#[test]
fn test_get_update_index() {
    let lengths: Vec<usize> = vec![10, 20, 3, 5, 8];
    let update_index = get_update_index(&lengths, 1, 2);

    assert_eq!(update_index, 34);
}

#[test]
fn test_is_at_end_of_input_false() {
    let lengths: Vec<usize> = vec![10, 20, 3, 5, 8];
    let at_the_end = is_at_end_of_input(&lengths, 7, 1);

    assert!(!at_the_end);
}

#[test]
fn test_is_at_end_of_input_true() {
    let lengths: Vec<usize> = vec![10, 20, 3, 5, 8, 5];
    let at_the_end = is_at_end_of_input(&lengths, 7, 4);

    assert!(at_the_end);
}

#[test]
fn test_is_at_end_of_input_false_for_first_char() {
    let lengths: Vec<usize> = vec![0, 0];
    let at_the_end = is_at_end_of_input(&lengths, 0, 0);

    assert!(at_the_end);
}

#[test]
fn test_get_cursor_down_x_y_target_shorter() {
    let coordinates = get_cursor_down_x_y(3, 2, 2);

    let x = *coordinates.x.to_ref();
    let y = *coordinates.y.to_ref();

    assert_eq!(
        (x, y),
        (2, 3),
        "Expected x,y to equal {:?} but got {:?}",
        (2, 3),
        (x, y)
    );
}

#[test]
fn test_get_cursor_down_x_y_target_longer() {
    let coordinates = get_cursor_down_x_y(3, 5, 6);

    let x = *coordinates.x.to_ref();
    let y = *coordinates.y.to_ref();

    assert_eq!(
        (x, y),
        (3, 6),
        "Expected x,y to equal {:?} but got {:?}",
        (3, 6),
        (x, y)
    );
}

#[test]
fn test_get_cursor_down_x_y_target_same_length() {
    let coordinates = get_cursor_down_x_y(3, 5, 3);

    let x = *coordinates.x.to_ref();
    let y = *coordinates.y.to_ref();

    assert_eq!(
        (x, y),
        (3, 6),
        "Expected x,y to equal {:?} but got {:?}",
        (3, 6),
        (x, y)
    );
}

#[test]
fn test_get_cursor_up_x_y_target_shorter() {
    let coordinates = get_cursor_up_x_y(3, 2, 2);

    let x = *coordinates.x.to_ref();
    let y = *coordinates.y.to_ref();

    assert_eq!(
        (x, y),
        (2, 1),
        "Expected x,y to equal {:?} but got {:?}",
        (2, 1),
        (x, y)
    );
}

#[test]
fn test_get_cursor_up_x_y_target_longer() {
    let coordinates = get_cursor_up_x_y(3, 5, 6);

    let x = *coordinates.x.to_ref();
    let y = *coordinates.y.to_ref();

    assert_eq!(
        (x, y),
        (3, 4),
        "Expected x,y to equal {:?} but got {:?}",
        (3, 4),
        (x, y)
    );
}

#[test]
fn test_get_cursor_up_x_y_target_same_length() {
    let coordinates = get_cursor_up_x_y(3, 5, 3);

    let x = *coordinates.x.to_ref();
    let y = *coordinates.y.to_ref();

    assert_eq!(
        (x, y),
        (3, 4),
        "Expected x,y to equal {:?} but got {:?}",
        (3, 4),
        (x, y)
    );
}
