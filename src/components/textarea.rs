#![allow(clippy::too_many_arguments)]

use std::{cell::RefCell, cmp::min, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId, Emitter, KeyCode, KeyEvent},
    default_widgets::Overflow,
    geometry::{Pos, Size},
    prelude::{Context, ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::{State, Value},
    widgets::{Element, Elements},
};
use arboard::Clipboard;
use log::info;
use rstest::rstest;
use serde::{Deserialize, Serialize};

use crate::{
    app::{GlobalEventHandler, TAB_NAV},
    theme::{get_app_theme, AppTheme},
};

use super::dashboard::DashboardMessages;

#[derive(State, Default)]
pub struct TextAreaState {
    pub foreground: Value<String>,
    pub background: Value<String>,
    pub display_input: Value<String>,
    pub cursor_prefix: Value<String>,
    pub cursor_char: Value<char>,
    pub focused: Value<bool>,

    #[state_ignore]
    pub input: String,

    #[state_ignore]
    pub cursor_pos: CursorPosition,

    #[state_ignore]
    pub ident: String,
}

impl TextAreaState {
    #[allow(dead_code)]
    pub fn new(input: String, prefix: String, x: usize, y: usize, fg: &str, bg: &str) -> Self {
        TextAreaState {
            display_input: input.into(),
            cursor_prefix: prefix.into(),
            cursor_pos: CursorPosition { x, y },
            focused: false.into(),
            foreground: fg.to_string().into(),
            background: bg.to_string().into(),
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct CursorPosition {
    x: usize,
    y: usize,
}

pub struct TextArea {
    #[allow(unused)]
    pub input_for: Option<String>,

    #[allow(unused)]
    pub listeners: Vec<String>,

    #[allow(unused)]
    pub component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,

    #[allow(unused)]
    pub app_theme: AppTheme,
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
        let app_theme = get_app_theme();

        let name: String = ident.clone();

        let state = TextAreaState {
            foreground: app_theme.foreground.to_ref().to_string().into(),
            background: app_theme.background.to_ref().to_string().into(),

            display_input: "".to_string().into(),
            cursor_char: ' '.into(),
            cursor_prefix: "".to_string().into(),

            input: "".to_string(),
            cursor_pos: CursorPosition::default(),
            focused: false.into(),
            ident,
        };

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

    fn send_to_listeners(&self, code: KeyCode, state: &mut TextAreaState, emitter: Emitter) {
        if let KeyCode::Char(_) = code {
            // TODO: Fix the outgoing message so it is not 100% coupled to only response body
            // editing, use InputUpdate instead of InputChange, like in edit_input.rs
            if let Ok(ids) = self.component_ids.try_borrow() {
                let input_value = state.input.clone();
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
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TextAreaMessages {
    InputChange(String),
    SetInput(String),
}

impl Component for TextArea {
    type State = TextAreaState;
    type Message = String;

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        state.focused.set(false);
        context.publish("textarea_focus", |state| &state.focused);

        // TODO: Try to refactor these unsafe blocks out
        #[allow(static_mut_refs)]
        let tab_nav = unsafe { TAB_NAV.get_mut() };
        *tab_nav = true;
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        state.focused.set(true);

        // TODO: Try to refactor these unsafe blocks out
        #[allow(static_mut_refs)]
        let tab_nav = unsafe { TAB_NAV.get_mut() };
        *tab_nav = false;

        context.publish("textarea_focus", |state| &state.focused);

        // NOTE: Trying to trigger a redraw here, the focus change is not showing up immediately
        // until a redraw happens, usually by typing in the textarea
        // elements.by_attribute("id", "container")
        //     .first(|element, _| {
        //         let overflow = element.to::<Overflow>();
        //         overflow.scroll_down();
        //     });
    }

    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        match key.code {
            KeyCode::Char(c) => {
                let emitter = context.emitter.clone();
                handle_typing(c, state, &mut elements, &mut context);
                self.send_to_listeners(key.code, state, emitter);
            }

            KeyCode::Enter => {
                let emitter = context.emitter.clone();
                handle_typing('\n', state, &mut elements, &mut context);
                self.send_to_listeners(key.code, state, emitter);
            }

            KeyCode::Backspace => backspace(state, context, &mut elements),
            KeyCode::Delete => delete(state, context, &mut elements),

            // TODO: Get back to handling tab characters after updating to the
            // latest version of Anathema to use runtime_builder.with_global_events_handler()
            // to toggle focus management on/off at runtime
            KeyCode::Tab => {
                handle_typing(' ', state, &mut elements, &mut context);
                handle_typing(' ', state, &mut elements, &mut context);
            },
            KeyCode::BackTab => todo!(),

            // TODO: Get back to handling Ctrl-C copy command after updating to the
            // latest version of Anathema to use runtime_builder.with_global_events_handler()
            // to toggle focus management on/off at runtime
            KeyCode::CtrlC => copy(state),

            KeyCode::Left => move_left(state),
            KeyCode::Right => move_right(state),
            KeyCode::Up => handle_move_up(state, &mut elements),
            KeyCode::Down => handle_move_down(state, &mut elements),

            KeyCode::Esc => escape(state, context),

            _ => {} // anathema::component::KeyCode::Insert => todo!(),
                    // anathema::component::KeyCode::F(_) => todo!(),
                    // anathema::component::KeyCode::Null => todo!(),
                    // anathema::component::KeyCode::CapsLock => todo!(),
                    // anathema::component::KeyCode::ScrollLock => todo!(),
                    // anathema::component::KeyCode::NumLock => todo!(),
                    // anathema::component::KeyCode::PrintScreen => todo!(),
                    // anathema::component::KeyCode::Pause => todo!(),
                    // anathema::component::KeyCode::Menu => todo!(),
                    // anathema::component::KeyCode::KeypadBegin => todo!(),
                    // Maybe?
                    // KeyCode::Home => todo!(),
                    // KeyCode::End => todo!(),
                    // KeyCode::PageUp => todo!(),
                    // KeyCode::PageDown => todo!(),
        }
    }
}

fn copy(state: &mut TextAreaState) {
    let Ok(mut clipboard) = Clipboard::new() else {
        return;
    };

    let set_operation = clipboard.set();
    let _ = set_operation.text(state.display_input.to_ref().to_string());
}

// TODO: Add tests for the backspace function
fn backspace(
    state: &mut TextAreaState,
    mut context: Context<'_, TextAreaState>,
    elements: &mut Elements<'_, '_>,
) {
    let mut block = false;
    elements
        .by_attribute("id", "container")
        .first(|_, attributes| {
            if let Some(editable) = attributes.get::<bool>("editable") {
                if !editable {
                    block = true;
                }
            }
        });

    if block {
        return;
    }

    let CursorPosition { x, y } = state.cursor_pos;
    if x == 0 && y == 0 {
        return;
    }

    let cursor_pos = &state.cursor_pos;
    let index = get_insert_position(&mut state.input, cursor_pos);

    let mut indices = state.input.char_indices();
    let Some((remove_index, _)) = indices.nth(index - 1) else {
        return;
    };

    state.input.remove(remove_index);
    state.display_input.set(state.input.clone());

    if x > 0 {
        state.cursor_pos.x -= 1;
    } else if x == 0 && y > 0 {
        state.cursor_pos.y -= 1;

        let new_input = state.display_input.to_ref();
        let new_line = new_input.lines().nth(state.cursor_pos.y).unwrap_or("");
        state.cursor_pos.x = new_line.len();
    }

    update_cursor_after_move(state);

    let event_name = format!("{}_textchange", state.ident);
    context.publish(&event_name, |state| &state.display_input);
}

fn escape(state: &mut TextAreaState, mut context: Context<'_, TextAreaState>) {
    context.set_focus("id", "app");

    let event_name = format!("{}_escape", state.ident);
    context.publish(&event_name, |state| &state.cursor_char);
    context.publish("textarea_focus", |state| &state.focused);
}

// TODO: Add tests for the delete function
fn delete(
    state: &mut TextAreaState,
    mut context: Context<'_, TextAreaState>,
    elements: &mut Elements<'_, '_>,
) {
    let mut block = false;
    elements
        .by_attribute("id", "container")
        .first(|_, attributes| {
            if let Some(editable) = attributes.get::<bool>("editable") {
                if !editable {
                    block = true;
                }
            }
        });

    if block {
        return;
    }

    let CursorPosition { x, y } = state.cursor_pos;

    let lines = state.input.lines().count();
    let last_line = state.input.lines().last().unwrap_or("");

    if x == last_line.len() - 1 && y == lines - 1 {
        return;
    }

    let cursor_pos = &state.cursor_pos;

    let current_input_index = get_insert_position(&mut state.input, cursor_pos);
    let Some((delete_index, _)) = state.input.char_indices().nth(current_input_index) else {
        return;
    };

    state.input.remove(delete_index);
    state.display_input.set(state.input.clone());

    update_cursor_after_move(state);

    let event_name = format!("{}_textchange", state.ident);
    context.publish(&event_name, |state| &state.display_input);
}

fn move_down(state: &mut TextAreaState, size: Size) {
    let CursorPosition { x, y } = state.cursor_pos;
    let input = state.display_input.to_ref().to_string();

    let current_line = input.lines().nth(state.cursor_pos.y).unwrap_or("");

    let starting_index = input
        .lines()
        .take(state.cursor_pos.y)
        .fold(0, |ndx, line| ndx + line.len().saturating_sub(1));
    let ending_index = starting_index + current_line.len().saturating_sub(1);

    let is_wider_than_width = current_line.len() > size.width;
    let is_cursor_not_on_last_subline =
        state.cursor_pos.x < current_line.len().saturating_sub(size.width);

    let total_sublines = ((current_line.len() / size.width) as f32).ceil() as usize;
    let last_full_line_start = starting_index + (total_sublines.saturating_sub(1) * size.width);
    let last_full_line_end = last_full_line_start + size.width;

    if is_wider_than_width && is_cursor_not_on_last_subline {
        state.cursor_pos.x += size.width;
        return;
    } else if is_wider_than_width
        && state.cursor_pos.x >= last_full_line_start
        && state.cursor_pos.x <= last_full_line_end
    {
        let last_line_pos = state.cursor_pos.x + size.width;
        state.cursor_pos.x = min(last_line_pos, ending_index);
        return;
    }

    let total_lines = state.input.lines().count();
    if y == total_lines.saturating_sub(1) {
        return;
    }

    state.cursor_pos.y += 1;

    let input = state.display_input.to_ref().to_string();
    let new_line_contents = input.lines().nth(state.cursor_pos.y).unwrap_or("");
    if x > new_line_contents.len() {
        state.cursor_pos.x = new_line_contents.len();
    }

    render_display(state);
}

fn handle_move_down(state: &mut TextAreaState, elements: &mut Elements<'_, '_>) {
    elements
        .by_attribute("id", "container")
        .first(|element, _| {
            let size = element.size();

            move_down(state, size);
            scroll_into_view(element, state);
        });

    update_cursor_after_move(state);
}

fn move_up(state: &mut TextAreaState, size: Size) {
    let CursorPosition { x, y } = state.cursor_pos;

    let input = state.display_input.to_ref().to_string();
    let current_line = input.lines().nth(state.cursor_pos.y).unwrap_or("");
    let is_wider_than_width = current_line.len() > size.width;
    let is_cursor_on_subline = state.cursor_pos.x > size.width;

    if is_wider_than_width && is_cursor_on_subline {
        state.cursor_pos.x -= size.width;
        return;
    }

    if y == 0 {
        return;
    }

    // Otherwise, if current line isn't wider than available width
    // then we move up a line
    state.cursor_pos.y -= 1;

    let new_line_contents = input.lines().nth(state.cursor_pos.y).unwrap_or("");
    if x > new_line_contents.len() {
        state.cursor_pos.x = new_line_contents.len();
    }
    let line_width = new_line_contents.len();

    if line_width > size.width {
        let sub_lines = (line_width / size.width) as i16;
        let shift = sub_lines as usize * size.width;

        state.cursor_pos.x += shift;
    }

    render_display(state);
}

fn handle_move_up(state: &mut TextAreaState, elements: &mut Elements<'_, '_>) {
    elements
        .by_attribute("id", "container")
        .first(|element, _| {
            let size = element.size();

            move_up(state, size);

            scroll_into_view(element, state);
        });

    update_cursor_after_move(state);
}

fn move_right(state: &mut TextAreaState) {
    let CursorPosition { x, y } = state.cursor_pos;

    let input = state.display_input.to_ref().to_string();
    let total_lines = input.lines().count();
    let current_line = input.lines().nth(state.cursor_pos.y).unwrap_or(" ");

    let is_at_end_of_line = x == current_line.len();
    let is_at_last_line = y == total_lines.saturating_sub(1);

    if is_at_end_of_line && is_at_last_line {
        return;
    } else if is_at_end_of_line && !is_at_last_line {
        state.cursor_pos.x = 0;
        state.cursor_pos.y += 1;
    } else {
        state.cursor_pos.x += 1;
    }

    update_cursor_after_move(state);
}

fn move_left(state: &mut TextAreaState) {
    let CursorPosition { x, y } = state.cursor_pos;

    if x == 0 && y == 0 {
        return;
    } else if x == 0 {
        let input = state.display_input.to_ref();
        let prev_line_ndx = y.saturating_sub(1);
        let Some(previous_line) = input.lines().nth(prev_line_ndx) else {
            return;
        };

        state.cursor_pos.x = previous_line.len();
        state.cursor_pos.y = state.cursor_pos.y.saturating_sub(1);
    } else {
        state.cursor_pos.x = state.cursor_pos.x.saturating_sub(1);
    }

    update_cursor_after_move(state);
}

fn update_cursor_after_move(state: &mut TextAreaState) {
    let input = state.display_input.to_ref().to_string();
    let cursor_input_pos = get_insert_position(&mut input.to_string(), &state.cursor_pos);

    let indices = input.char_indices();

    let prefix_end = cursor_input_pos;
    let cursor_char_index = cursor_input_pos;

    let prefix: String = indices.take(prefix_end).map(|(_, c)| c).collect();

    let mut indices = input.char_indices();

    let mut cursor_char: char = indices
        .nth(cursor_char_index)
        .map(|(_, c)| c)
        .unwrap_or(' ');

    // Replace newline with space when the cursor moves to the end of a line
    if cursor_char == '\n' {
        cursor_char = ' ';
    }

    state.cursor_prefix.set(prefix);
    state.cursor_char.set(cursor_char);
}

fn handle_typing(
    c: char,
    state: &mut TextAreaState,
    elements: &mut Elements<'_, '_>,
    context: &mut Context<'_, TextAreaState>,
) {
    elements
        .by_attribute("id", "container")
        .first(|element, attributes| {
            if let Some(editable) = attributes.get::<bool>("editable") {
                if !editable {
                    return;
                }
            }

            add_character(c, state);
            render_display(state);
            scroll_into_view(element, state);

            let event_name = format!("{}_textchange", state.ident);
            context.publish(&event_name, |state| &state.display_input);
        });
}

// TODO: add tests to render_display
fn render_display(state: &mut TextAreaState) {
    state.display_input.set(state.input.clone());

    let prefix = get_cursor_prefix(state);
    state.cursor_prefix.set(prefix);
}

fn scroll_into_view(element: &mut Element<'_>, state: &mut TextAreaState) {
    let overflow = element.to::<Overflow>();
    let CursorPosition { x, y } = state.cursor_pos;
    let pos = Pos::new(x as i32, y as i32);
    overflow.scroll_to(pos);
}

fn get_cursor_prefix(state: &mut TextAreaState) -> String {
    let index = get_insert_position(&mut state.input, &state.cursor_pos);

    let mut prefix = state.input.clone();
    let Some((split_index, _)) = prefix.char_indices().nth(index) else {
        return state.input.clone();
    };

    let _ = prefix.split_off(split_index);

    prefix
}

fn add_character(c: char, state: &mut TextAreaState) {
    let TextAreaState { cursor_pos, .. } = state;

    // let mut current_input = state.input.clone();
    let insert_position = get_insert_position(&mut state.input, cursor_pos);
    do_insert(&mut state.input, insert_position, c);

    if c == '\n' {
        cursor_pos.x = 0;
        cursor_pos.y += 1;
    } else {
        cursor_pos.x += 1;
    }

    // state.display_input.set(current_input);
    // update_cursor_display(prefix, state);

    // let adjusted = adjust_lines(&current_input, width);
    // Update state
    // state.input.set(adjusted);
    // update_cursor_after_move(state);
}

#[allow(unused)]
fn adjust_lines(input: &str, width: usize) -> String {
    // Adjust lines that have gotten wider than the available area
    let mut carry_over: Option<String> = None;
    let lines = input.lines();
    let mut adjusted: String = String::new();
    for line in lines {
        let mut this_line = String::new();

        if let Some(carry) = &carry_over {
            this_line.push_str(carry);
            carry_over = None;
        }

        this_line.push_str(line);

        if this_line.len() >= width - 1 {
            let line_copy = this_line.clone();

            let Some((split_index, _)) = line_copy.char_indices().nth(width - 1) else {
                continue;
            };

            let (left, right) = line_copy.split_at(split_index);
            this_line = left.to_string();
            this_line.push('\n');

            let right = right.to_string();
            carry_over = Some(right.replace('\n', ""));
        }

        adjusted.push_str(&this_line);
    }

    if let Some(carry) = carry_over {
        adjusted.push_str(&carry);
    }

    adjusted
}

#[rstest]
#[ignore]
fn test_adjust_lines() {
    let input = "Here is a line\n and more text";
    let width = 5;

    let adjusted = adjust_lines(input, width);

    assert_eq!(adjusted, "Here\n is \na li\nne\n and\n mor\ne te\nxt");
}

// TODO: Determine if I can get rid of this after adjusting textarea
// to update newline characters if an edit makes a line longer than
// the available horizontal area
#[allow(unused)]
fn update_cursor_display(prefix: String, state: &mut TextAreaState) {
    state.cursor_prefix.set(prefix.clone());

    let is_at_end = is_cursor_at_end(
        &state.cursor_pos,
        &mut state.display_input.to_ref().to_string(),
    );

    if is_at_end {
        state.cursor_char.set(' ');
    } else {
        let cursor_char = prefix.chars().last().unwrap();
        state.cursor_char.set(cursor_char);
    }
}

fn is_cursor_at_end(cursor: &CursorPosition, input: &mut str) -> bool {
    let Some((last_index, _)) = input.char_indices().last() else {
        return true;
    };

    let curr_pos = get_insert_position(input, cursor);

    let last_cursor_index = last_index + 1;
    info!(
        "input: {input}, last_index: {last_index}, curr_pos: {curr_pos}, last_cursor_index: {last_cursor_index}"
    );

    last_cursor_index == curr_pos
}

/// Returns the index of the next character insertion in the input string
fn get_insert_position(input: &mut str, cursor_pos: &CursorPosition) -> usize {
    let CursorPosition { x, y } = cursor_pos;
    let lines = input.split('\n').take(*y);

    x + y + lines.fold(0, |sum, line| sum + line.len())
}

fn do_insert(current_input: &mut String, insert_position: usize, c: char) {
    let mut indices = current_input.char_indices();
    match indices.nth(insert_position) {
        Some((index, _)) => {
            current_input.insert(index, c);

            // let mut prefix = current_input.clone();
            // let _ = prefix.split_off(index + 1);
            // prefix
        }

        // current_input is empty
        None => {
            if current_input.is_empty() {
                current_input.push(c);
                // current_input.clone()
            } else {
                info!("insert_position: {insert_position}, c: '{c}'");
                current_input.insert(insert_position, c);
                // let mut prefix = current_input.clone();
                // let _ = prefix.split_off(insert_position + 1);
                //
                // prefix
            }
        }
    }
}

#[rstest]
#[ignore]
fn test_copy() {
    let mut state = TextAreaState {
        display_input: "Some stuff".to_string().into(),
        ..Default::default()
    };

    copy(&mut state);

    let Ok(mut clipboard) = Clipboard::new() else {
        panic!("Could not get clipboard access");
    };

    let get_op = clipboard.get();
    let text = get_op.text().unwrap_or("failed".to_string());

    assert_eq!(text, "Some stuff");
}

#[rstest]
#[case(
    TextAreaState {
        cursor_pos: CursorPosition { x: 2, y: 0 },
        display_input: "ab".to_string().into(),
        input: "ab".to_string(),
        cursor_prefix: "".to_string().into(),
        cursor_char: ' '.into(),
        ..Default::default()
    },
    'x',
    TestMoveCursorResult {
        char_result: ' ',
        prefix_result: "",
        x_result: 3,
        y_result: 0,
    }
)]
#[case(
    TextAreaState {
        cursor_pos: CursorPosition { x: 0, y: 0 },
        display_input: "".to_string().into(),
        input: "".to_string(),
        cursor_prefix: "".to_string().into(),
        cursor_char: ' '.into(),
        ..Default::default()
    },
    'a',
    TestMoveCursorResult {
        char_result: ' ',
        prefix_result: "",
        x_result: 1,
        y_result: 0,
    }
)]
#[case(
    TextAreaState {
        cursor_pos: CursorPosition { x: 0, y: 0 },
        display_input: "ab\n".to_string().into(),
        input: "ab\n".to_string(),
        cursor_prefix: "".to_string().into(),
        cursor_char: 'a'.into(),
        ..Default::default()
    },
    'x',
    TestMoveCursorResult {
        char_result: 'a',
        prefix_result: "",
        x_result: 1,
        y_result: 0,
    }
)]
fn test_add_character(
    #[case] mut state: TextAreaState,
    #[case] new_char: char,
    #[case] expected: TestMoveCursorResult,
) {
    add_character(new_char, &mut state);

    let actual = TestMoveCursorResult {
        char_result: *state.cursor_char.to_ref(),
        prefix_result: &state.cursor_prefix.to_ref(),
        x_result: state.cursor_pos.x,
        y_result: state.cursor_pos.y,
    };

    dbg!(&*state.input);

    assert_eq!(actual, expected);
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
struct TestMoveCursorResult<'a> {
    char_result: char,
    prefix_result: &'a str,
    x_result: usize,
    y_result: usize,
}

#[rstest]
#[case(CursorPosition {x: 3, y: 0}, " ", "abc\n", "abc\ndef", ' ', "abc\ndef", 3, 1)]
#[case(CursorPosition {x: 6, y: 0}, " ", "abcdef\n", "abcdef\nghi", ' ', "abcdef\nghi", 3, 1)]
#[case(CursorPosition {x: 6, y: 1}, " ", "abcdef\nghi", "abcdef\nghi", ' ', "abcdef\nghi", 6, 1)]
fn test_move_down(
    #[case] cursor_pos: CursorPosition,
    #[case] cursor_char: char,
    #[case] cursor_prefix: &str,
    #[case] input: &str,

    #[case] char_result: char,
    #[case] prefix_result: &str,
    #[case] x_result: usize,
    #[case] y_result: usize,
) {
    let mut text_area_state = TextAreaState {
        cursor_pos,
        cursor_char: cursor_char.into(),
        cursor_prefix: cursor_prefix.to_string().into(),
        display_input: input.to_string().into(),
        input: input.to_string(),
        background: "#000000".to_string().into(),
        foreground: "#ffffff".to_string().into(),
        ..Default::default()
    };

    let size = Size::new(100, 100);

    move_down(&mut text_area_state, size);

    let actual = TestMoveCursorResult {
        char_result: *text_area_state.cursor_char.to_ref(),
        prefix_result: &text_area_state.cursor_prefix.to_ref(),
        x_result: text_area_state.cursor_pos.x,
        y_result: text_area_state.cursor_pos.y,
    };

    let expected = TestMoveCursorResult {
        char_result,
        prefix_result,
        x_result,
        y_result,
    };

    assert_eq!(actual, expected);
}

#[rstest]
#[case(CursorPosition {x: 3, y: 1}, " ", "abc\ndef", "abc\ndef", ' ', "abc", 3, 0)]
#[case(CursorPosition {x: 5, y: 1}, " ", "abc\ndefgh", "abc\ndefgh", ' ', "abc", 3, 0)]
#[case(CursorPosition {x: 3, y: 0}, " ", "abc", "abc\ndef", ' ', "abc", 3, 0)]
fn test_move_up(
    #[case] cursor_pos: CursorPosition,
    #[case] cursor_char: char,
    #[case] cursor_prefix: &str,
    #[case] input: &str,

    #[case] char_result: char,
    #[case] prefix_result: &str,
    #[case] x_result: usize,
    #[case] y_result: usize,
) {
    let mut text_area_state = TextAreaState {
        cursor_pos,
        cursor_char: cursor_char.into(),
        cursor_prefix: cursor_prefix.to_string().into(),
        display_input: input.to_string().into(),
        input: input.to_string(),
        background: "#000000".to_string().into(),
        foreground: "#ffffff".to_string().into(),
        ..Default::default()
    };

    let size = Size::new(100, 100);

    move_up(&mut text_area_state, size);

    let actual = TestMoveCursorResult {
        char_result: *text_area_state.cursor_char.to_ref(),
        prefix_result: &text_area_state.cursor_prefix.to_ref(),
        x_result: text_area_state.cursor_pos.x,
        y_result: text_area_state.cursor_pos.y,
    };

    let expected = TestMoveCursorResult {
        char_result,
        prefix_result,
        x_result,
        y_result,
    };

    assert_eq!(actual, expected);
}

#[rstest]
#[case(CursorPosition {x: 3, y: 0}, " ", "abc", "abc\ndef", 'd', "abc\n", 0, 1)]
#[case(CursorPosition {x: 3, y: 0}, " ", "abc", "abc", ' ', "abc", 3, 0)]
#[case(CursorPosition {x: 3, y: 1}, " ", "abc\ndef", "abc\ndef", ' ', "abc\ndef", 3, 1)]
fn test_move_right(
    #[case] cursor_pos: CursorPosition,
    #[case] cursor_char: char,
    #[case] cursor_prefix: &str,
    #[case] input: &str,

    #[case] char_result: char,
    #[case] prefix_result: &str,
    #[case] x_result: usize,
    #[case] y_result: usize,
) {
    let mut text_area_state = TextAreaState {
        cursor_pos,
        cursor_char: cursor_char.into(),
        cursor_prefix: cursor_prefix.to_string().into(),
        display_input: input.to_string().into(),
        background: "#000000".to_string().into(),
        foreground: "#ffffff".to_string().into(),
        ..Default::default()
    };

    move_right(&mut text_area_state);

    let actual = TestMoveCursorResult {
        char_result: *text_area_state.cursor_char.to_ref(),
        prefix_result: &text_area_state.cursor_prefix.to_ref(),
        x_result: text_area_state.cursor_pos.x,
        y_result: text_area_state.cursor_pos.y,
    };

    let expected = TestMoveCursorResult {
        char_result,
        prefix_result,
        x_result,
        y_result,
    };

    assert_eq!(actual, expected);
}

#[rstest]
#[case(CursorPosition {x: 3, y: 0}, " ", "abc", "abc", 'c', "ab", 2, 0)]
#[case(CursorPosition {x: 0, y: 1}, "d", "abc\n", "abc\ndef", " ", "abc", 3, 0)]
fn test_move_left(
    #[case] cursor_pos: CursorPosition,
    #[case] cursor_char: char,
    #[case] cursor_prefix: &str,
    #[case] input: &str,

    #[case] char_result: char,
    #[case] prefix_result: &str,
    #[case] x_result: usize,
    #[case] y_result: usize,
) {
    let mut text_area_state = TextAreaState {
        cursor_pos,
        cursor_char: cursor_char.into(),
        cursor_prefix: cursor_prefix.to_string().into(),
        display_input: input.to_string().into(),
        background: "#000000".to_string().into(),
        foreground: "#ffffff".to_string().into(),
        ..Default::default()
    };

    move_left(&mut text_area_state);

    let actual = TestMoveCursorResult {
        char_result: *text_area_state.cursor_char.to_ref(),
        prefix_result: &text_area_state.cursor_prefix.to_ref(),
        x_result: text_area_state.cursor_pos.x,
        y_result: text_area_state.cursor_pos.y,
    };

    let expected = TestMoveCursorResult {
        char_result,
        prefix_result,
        x_result,
        y_result,
    };

    assert_eq!(actual, expected);
}

#[rstest]
#[case("Som string", 3, 'e', "Some string")]
#[case("Sàm string", 3, 'e', "Sàme string")]
#[case("Söm string", 3, 'e', "Söme string")]
fn test_do_insert(
    #[case] mut input: String,
    #[case] position: usize,
    #[case] c: char,
    #[case] result: String,
) {
    do_insert(&mut input, position, c);

    assert_eq!(input, result);
}

#[rstest]
#[case("", 0, 0, 0)]
#[case("Some word\nthing", 0, 1, 10)]
#[case("Some word\nthing", 4, 1, 14)]
#[case("Some word\nthing", 1, 1, 11)]
#[case("Some word\nthing", 3, 0, 3)]
fn test_get_insert_position(
    #[case] mut input: String,
    #[case] x: usize,
    #[case] y: usize,
    #[case] result: usize,
) {
    let cursor_pos = CursorPosition { x, y };

    let insert_position = get_insert_position(&mut input, &cursor_pos);

    assert_eq!(insert_position, result);
}

#[rstest]
#[case(CursorPosition{x: 0, y: 0}, "Some text", false)]
#[case(CursorPosition{x: 9, y: 0}, "Some text", true)]
#[case(CursorPosition{x: 5, y: 1}, "Some\n text", true)]
#[case(CursorPosition{x: 4, y: 1}, "Some\n text", false)]
#[case(CursorPosition{x: 0, y: 1}, "Some\n text", false)]
#[case(CursorPosition{x: 2, y: 0}, "ab", true)]
fn test_is_cursor_at_end(
    #[case] cursor: CursorPosition,
    #[case] mut input: String,
    #[case] result: bool,
) {
    let at_end = is_cursor_at_end(&cursor, &mut input);

    assert_eq!(at_end, result);
}

#[rstest]
#[case("Some input", TextAreaState::new("Test input".to_string(), "Test input".to_string(), 10, 0, "", ""), " ".to_string())]
#[case("abc", TextAreaState::new("abc".to_string(), "ab".to_string(), 2, 0, "", ""), "c".to_string())]
fn test_update_cursor_display(
    #[case] prefix: String,
    #[case] mut state: TextAreaState,
    #[case] result: String,
) {
    update_cursor_display(prefix, &mut state);

    assert_eq!(state.cursor_char.to_ref().to_string(), result);
}
