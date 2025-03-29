use std::{
    cell::RefCell,
    cmp::min,
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    rc::Rc,
};

use anathema::{
    component::{Component, ComponentId, KeyCode, KeyEvent},
    geometry::Size,
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, Hex, List, State, Value},
    widgets::{components::events::KeyState, Elements},
};
use log::info;
use serde::{Deserialize, Serialize};
use syntect::highlighting::Theme;

use crate::{
    app::GlobalEventHandler,
    options::get_syntax_theme,
    templates::template,
    theme::{get_app_theme, get_app_theme_persisted, AppTheme},
};

use super::{
    dashboard::{DashboardMessages, KeebEvent, KeebState},
    send_message,
    syntax_highlighter::highlight,
};

pub const CODE_SAMPLE: &str = include_str!("../../themes/code_sample.rs");

#[derive(Debug)]
enum ScrollDirection {
    Up,
    Down,
}

pub struct ResponseRenderer {
    #[allow(unused)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    text_filter: TextFilter,
    theme: Option<Theme>,

    // overflow: Option<&'app mut Overflow>,
    size: Option<Size>,
    response_reader: Option<BufReader<File>>,
    response_offset: usize,
    viewport_height: usize,
    extension: String,

    // All lines from the response
    response_lines: Vec<String>,

    code_sample: Option<String>,
    code_ext: Option<String>,
}

impl ResponseRenderer {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
        ident: String,
    ) -> anyhow::Result<()> {
        let template = if ident == "response_renderer" {
            template("templates/response_renderer")
        } else {
            template("templates/syntax_highlighter_renderer")
        };

        let id = builder.register_component(
            ident.clone(),
            template,
            ResponseRenderer::new(ids.clone()),
            ResponseRendererState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(ident, id);

        Ok(())
    }

    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        ResponseRenderer {
            component_ids,
            text_filter: TextFilter {
                ..Default::default()
            },
            theme: None,
            response_reader: None,
            response_offset: 0,
            viewport_height: 0,
            size: None,
            extension: "".to_string(),
            response_lines: vec![],
            code_ext: None,
            code_sample: None,
        }
    }

    fn update_app_theme(&self, state: &mut ResponseRendererState) {
        let app_theme = get_app_theme_persisted();

        // state.app_theme.set(app_theme);
        // TODO: Temp fix for weirdness around state updates to the app_theme
        let mut at = state.app_theme.to_mut();
        at.background.set(app_theme.background);
        at.foreground.set(app_theme.foreground);
        at.project_name_background
            .set(app_theme.project_name_background);
        at.project_name_foreground
            .set(app_theme.project_name_foreground);
        at.border_focused.set(app_theme.border_focused);
        at.border_unfocused.set(app_theme.border_unfocused);
        at.overlay_heading.set(app_theme.overlay_heading);
        at.overlay_background.set(app_theme.overlay_background);
        at.overlay_foreground.set(app_theme.overlay_foreground);
        at.overlay_submit_background
            .set(app_theme.overlay_submit_background);
        at.overlay_submit_foreground
            .set(app_theme.overlay_submit_foreground);

        at.overlay_cancel_background
            .set(app_theme.overlay_cancel_background);
        at.overlay_cancel_foreground
            .set(app_theme.overlay_cancel_foreground);
        at.menu_color_1.set(app_theme.menu_color_1);
        at.menu_color_2.set(app_theme.menu_color_2);
        at.menu_color_3.set(app_theme.menu_color_3);
        at.menu_color_4.set(app_theme.menu_color_4);
        at.menu_color_5.set(app_theme.menu_color_5);

        at.endpoint_name_background
            .set(app_theme.endpoint_name_background);
        at.endpoint_name_foreground
            .set(app_theme.endpoint_name_foreground);
        at.menu_opt_background.set(app_theme.menu_opt_background);
        at.menu_opt_foreground.set(app_theme.menu_opt_foreground);
        at.top_bar_background.set(app_theme.top_bar_background);
        at.top_bar_foreground.set(app_theme.top_bar_foreground);
        at.bottom_bar_background
            .set(app_theme.bottom_bar_background);
        at.bottom_bar_foreground
            .set(app_theme.bottom_bar_foreground);
    }

    fn render_response(
        &mut self,
        extension: String,
        elements: &mut Elements<'_, '_>,
        state: &mut ResponseRendererState,
        offset: usize,
        context: Context<'_, ResponseRendererState>,
    ) {
        if self.response_reader.is_none() {
            return;
        }

        if self.size.is_none() {
            return;
        }

        self.extension = extension;

        let size = self.size.unwrap();
        let response_reader = self.response_reader.as_mut().unwrap();
        self.response_offset = offset;
        self.viewport_height = size.height;

        let mut buf: Vec<u8> = vec![];
        match response_reader.read_to_end(&mut buf) {
            Ok(_) => {
                let response = String::from_utf8(buf).unwrap_or(String::from("oops"));
                let lines = response.lines();
                let response_lines: Vec<String> = lines.map(|s| s.to_string()).collect();
                self.response_lines = response_lines;
            }

            Err(error) => {
                let error_message = format!("There was an error reading the response: {}", error);

                self.send_error_message(&error_message, context);
            }
        }

        self.scroll_response(elements, state, offset);
    }

    fn send_error_message(&self, error_message: &str, context: Context<'_, ResponseRendererState>) {
        let dashboard_msg = DashboardMessages::ShowError(error_message.to_string());
        let Ok(msg) = serde_json::to_string(&dashboard_msg) else {
            return;
        };
        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };
        let _ = send_message("dashboard", msg, &ids, context.emitter);
    }

    fn scroll_response(
        &mut self,
        _elements: &mut Elements<'_, '_>,
        state: &mut ResponseRendererState,
        offset: usize,
    ) {
        if self.response_reader.is_none() {
            return;
        }

        if self.size.is_none() {
            return;
        }

        let last_offset = self.response_offset;

        let size = self.size.unwrap();
        self.response_offset = offset;
        self.viewport_height = size.height;

        let mut viewable_lines: Vec<String> = vec![];

        let last_response_line_index = self.response_lines.len();
        let last_viewable_index = self.response_offset + self.viewport_height;
        let ending_index = min(last_viewable_index, last_response_line_index);

        info!(
            "Rendering from {} to {}",
            self.response_offset, ending_index
        );

        for index in self.response_offset..ending_index {
            info!("Rendering index: {index}");

            let line = &self.response_lines[index];
            info!("Rendering line: {line}");

            if line.len() > size.width {
                let (new_line, _) = line.split_at(size.width.saturating_sub(5));

                let t = format!("{new_line}...");

                viewable_lines.push(t);
            } else {
                viewable_lines.push(line.to_string());
            }
        }

        let theme = get_syntax_theme();
        let viewable_response = viewable_lines.join("\n");

        let screens = last_response_line_index as f32 / self.viewport_height as f32;
        let current_screen = self.response_offset as f32 / self.viewport_height as f32;
        let percent = (current_screen / screens) * 100f32;
        if percent >= 100f32 {
            self.response_offset = last_offset;
            return;
        }

        let percent_scrolled = format!("{:0>2}", percent as usize);

        state.percent_scrolled.set(percent_scrolled);

        info!("viewable_response: {viewable_response}");

        self.set_response(state, viewable_response, Some(theme));
    }

    // NOTE: This one is now the one setting the response in the response text area with the syntax
    // highlighting
    fn set_response(
        &mut self,
        state: &mut ResponseRendererState,
        viewable_response: String,
        theme: Option<String>,
    ) {
        loop {
            if state.lines.len() == 0 {
                break;
            }

            state.lines.remove(0);
        }

        let (highlighted_lines, parsed_theme) =
            highlight(&viewable_response, &self.extension, theme);

        let bg = parsed_theme.settings.background;
        if self.theme.is_none() {
            self.theme = Some(parsed_theme);
        }

        if let Some(color) = bg {
            let hex_color = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
            state.response_background.set(hex_color);
        }

        highlighted_lines.iter().for_each(|hl| {
            let mut line: Line = Line {
                spans: List::empty(),
            };

            let head_src = hl.head.src.replace("\n", "");
            head_src.chars().for_each(|char| {
                let span = Span {
                    text: char.to_string().into(),
                    bold: hl.head.bold.into(),
                    foreground: hl.head.fg.into(),
                    background: hl.head.bg.into(),
                    original_background: None.into(),
                    original_foreground: None.into(),
                };
                line.spans.push(span);
            });

            hl.tail.iter().for_each(|span| {
                let src = span.src.replace("\n", "");
                src.chars().for_each(|char| {
                    let span = Span {
                        text: char.to_string().into(),
                        bold: span.bold.into(),
                        foreground: span.fg.into(),
                        background: span.bg.into(),
                        original_background: None.into(),
                        original_foreground: None.into(),
                    };
                    line.spans.push(span);
                });
            });

            state.lines.push(line);
        });

        info!("state.lines.len(): {}", state.lines.len());

        // NOTE: Uncomment for debugging lines
        // let mut index = 0;
        // state.lines.for_each(|line| {
        //     let l = line.get_line();
        //     let i = index;
        //
        //     info!("Line {i} = {l}");
        //
        //     index += 1;
        // });

        state.viewable_response.set(viewable_response);
    }

    fn update_size(
        &mut self,
        context: Context<'_, ResponseRendererState>,
        elements: &mut Elements<'_, '_>,
    ) {
        elements
            .by_attribute("id", "response_border")
            .first(|element, _| {
                info!("{:?}", element.size());
            });

        let size = context.viewport.size();

        let app_titles = 2; // top/bottom menus of dashboard
        let url_method_inputs = 3; // height of url and method inputs with borders
        let response_borders = 2; // borders around response input

        let total_height_offset = app_titles + url_method_inputs + response_borders;

        self.size = Some(Size {
            width: size.width,
            height: size.height - total_height_offset,
        });
    }

    fn next_filter_match(
        &mut self,
        state: &mut ResponseRendererState,
        elements: Elements<'_, '_>,
        context: Context<'_, ResponseRendererState>,
    ) {
        let current_index = self.text_filter.search_navigation_cursor;
        let last_index = self.text_filter.indexes.len().saturating_sub(1);

        self.text_filter.search_navigation_cursor = if current_index == last_index {
            0
        } else {
            current_index + 1
        };

        let line = self
            .text_filter
            .indexes
            .get(self.text_filter.search_navigation_cursor);

        let line = line.unwrap_or(&0);

        self.scroll_to_line(state, elements, context, *line);
    }

    fn previous_filter_match(
        &mut self,
        state: &mut ResponseRendererState,
        elements: Elements<'_, '_>,
        context: Context<'_, ResponseRendererState>,
    ) {
        let current_index = self.text_filter.search_navigation_cursor;
        self.text_filter.search_navigation_cursor = if current_index > 0 {
            current_index.saturating_sub(1)
        } else {
            self.text_filter.indexes.len().saturating_sub(1)
        };

        let line = self
            .text_filter
            .indexes
            .get(self.text_filter.search_navigation_cursor)
            .unwrap_or(&0);
        self.scroll_to_line(state, elements, context, *line);
    }

    fn scroll(
        &mut self,
        state: &mut ResponseRendererState,
        mut elements: Elements<'_, '_>,
        context: Context<'_, ResponseRendererState>,
        direction: ScrollDirection,
    ) {
        info!("scroll() direction: {direction:?}");

        let new_offset = match direction {
            ScrollDirection::Up => self.response_offset.saturating_sub(self.viewport_height),
            ScrollDirection::Down => self.response_offset + self.viewport_height,
        };

        info!("new_offset: {new_offset}");

        self.scroll_response(&mut elements, state, new_offset);

        if !state.filter.to_ref().is_empty() {
            let filter = state.filter.to_ref().to_string();
            self.apply_response_filter(filter, state, context, elements);
        }
    }

    fn apply_response_filter(
        &mut self,
        filter: String,
        state: &mut ResponseRendererState,
        context: Context<'_, ResponseRendererState>,
        elements: Elements<'_, '_>,
    ) {
        info!("apply_response_filter");
        loop {
            if state.filter_indexes.len() == 0 {
                break;
            }

            state.filter_indexes.remove(0);
        }
        state.filter_total.set(0);
        state.filter_nav_index.set(0);

        if filter.is_empty() {
            self.text_filter = self.get_text_filter(state);
            clear_highlights(state);

            return;
        }

        self.response_lines
            .iter()
            .enumerate()
            .for_each(|(idx, line)| {
                if line.contains(&filter) {
                    state.filter_indexes.push(idx);
                }
            });

        state.filter_total.set(state.filter_indexes.len());

        if state.filter_indexes.len() > 0 {
            self.text_filter = self.get_text_filter(state);

            self.do_filter(state, elements, context);
        } else {
            if self.text_filter.total > 0 {
                self.text_filter.total = 0;
                self.text_filter.indexes.clear();
                self.text_filter.search_navigation_cursor = 0;
            }

            clear_highlights(state);
        }
    }

    fn do_filter(
        &mut self,
        state: &mut ResponseRendererState,
        elements: Elements<'_, '_>,
        context: Context<'_, ResponseRendererState>,
    ) {
        // NOTE: Go to the first search match if user is still typing in search filter
        // don't do this if user is scrolling with Ctrl D/U or N/P
        if state.filter_input_focused {
            let default_index = 0;
            let first_index = self.text_filter.indexes.first().unwrap_or(&default_index);

            self.scroll_to_line(state, elements, context, *first_index);
        }

        self.apply_filter_highlights(state);
    }

    fn apply_filter_highlights(&mut self, state: &mut ResponseRendererState) {
        if let Some(size) = self.size {
            let rows = size.height;
            let range_end = (self.response_offset + rows).saturating_sub(1);
            let match_range = (self.response_offset, range_end);

            highlight_matches(
                state,
                match_range,
                &mut self.text_filter.indexes,
                &self.text_filter.filter,
            );
        }
    }

    fn get_text_filter(&self, state: &mut ResponseRendererState) -> TextFilter {
        let mut indexes: Vec<usize> = vec![];

        let i = state.filter_indexes.to_ref();
        i.iter().for_each(|e| {
            let val = *e.to_ref();
            indexes.push(val);
        });

        TextFilter {
            indexes,
            total: *state.filter_total.to_ref(),
            search_navigation_cursor: *state.filter_nav_index.to_ref(),
            filter: state.filter.to_ref().to_string(),
        }
    }

    fn scroll_to_line(
        &mut self,
        state: &mut ResponseRendererState,
        mut elements: Elements<'_, '_>,
        _context: Context<'_, ResponseRendererState>,
        line: usize,
    ) {
        self.scroll_response(&mut elements, state, line);

        self.apply_filter_highlights(state);
    }

    fn back_to_request(&self, context: Context<'_, ResponseRendererState>) {
        if let Ok(message) = serde_json::to_string(&DashboardMessages::BackToRequest) {
            let Ok(ids) = self.component_ids.try_borrow() else {
                return;
            };

            let _ = send_message("dashboard", message, &ids, context.emitter);
        }
    }
}

#[derive(Debug, State)]
pub struct Line {
    spans: Value<List<Span>>,
}

impl Line {
    #[allow(unused)]
    pub fn get_line(&mut self) -> String {
        self.spans
            .to_mut()
            .iter_mut()
            .fold(String::new(), |mut acc, span| {
                acc.push_str(&span.to_ref().text.to_ref().to_string());

                acc
            })
    }

    pub fn empty() -> Self {
        Self {
            spans: List::empty(),
        }
    }
}

#[derive(Debug, State)]
struct Span {
    text: Value<String>,
    bold: Value<bool>,
    foreground: Value<Hex>,
    background: Value<Hex>,
    original_background: Value<Option<Hex>>,
    original_foreground: Value<Option<Hex>>,
}

#[derive(Default, State)]
pub struct ResponseRendererState {
    scroll_position: Value<usize>,
    pub doc_height: Value<usize>,
    pub screen_cursor_x: Value<i32>,
    pub screen_cursor_y: Value<i32>,
    pub buf_cursor_x: Value<i32>,
    pub buf_cursor_y: Value<i32>,
    /// Rendered lines in the text area for current page
    pub lines: Value<List<Line>>,
    pub current_instruction: Value<Option<String>>,
    pub title: Value<String>,
    pub waiting: Value<String>,
    pub show_cursor: Value<bool>,
    /// String version of what is in the lines field
    pub viewable_response: Value<String>,
    pub response_background: Value<String>,
    pub percent_scrolled: Value<String>,
    pub app_theme: Value<AppTheme>,
    pub filter: Value<String>,
    pub filter_indexes: Value<List<usize>>,
    pub filter_total: Value<usize>,
    pub filter_nav_index: Value<usize>,
    pub has_search_matches: Value<bool>,
    pub transient_event_value: Value<String>,

    #[state_ignore]
    pub filter_input_focused: bool,
}

impl ResponseRendererState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        ResponseRendererState {
            transient_event_value: "".to_string().into(),
            filter_input_focused: false,
            has_search_matches: false.into(),

            viewable_response: "".to_string().into(),
            scroll_position: 0.into(),
            doc_height: 1.into(),
            screen_cursor_x: 0.into(),
            screen_cursor_y: 0.into(),
            buf_cursor_x: 0.into(),
            buf_cursor_y: 0.into(),
            lines: List::from_iter(vec![Line::empty()]),
            current_instruction: None.into(),
            title: "".to_string().into(),
            waiting: false.to_string().into(),
            show_cursor: true.into(),
            response_background: "#000000".to_string().into(),
            percent_scrolled: "0".to_string().into(),
            app_theme: app_theme.into(),
            filter: "".to_string().into(),
            filter_indexes: List::from_iter(vec![]),
            filter_total: 0.into(),
            filter_nav_index: 0.into(),
        }
    }
}

impl Component for ResponseRenderer {
    type State = ResponseRendererState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn receive(
        &mut self,
        ident: &str,
        value: CommonVal<'_>,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        match ident {
            "response_filter__input_update" => {
                info!("response_filter__input_update");
                state.filter.set(value.to_string());
                self.apply_response_filter(value.to_string(), state, context, elements);

                info!("self.text_filter.total: {}", self.text_filter.total);
                if self.text_filter.total > 0 {
                    state.has_search_matches.set(true);
                } else {
                    state.has_search_matches.set(false);
                }
            }

            "response_filter__input_escape" => {
                context.set_focus("id", "response_renderer");
                info!("Set focus back to response_renderer");
            }

            _ => {
                context.publish(ident, |state| &state.transient_event_value);
            }
        }
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        self.update_size(context, &mut elements);
        info!("response_renderer has focus");

        state.filter_input_focused = false;
    }

    fn on_blur(&mut self, _: &mut Self::State, _: Elements<'_, '_>, _: Context<'_, Self::State>) {
        info!("response_renderer lost focus");
    }

    fn resize(
        &mut self,
        _: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        self.update_size(context, &mut elements);

        // TODO: Update response text when the window gets resized
        // NOTE: Causes panic!
        // self.scroll_response(&mut elements, state, self.response_offset);
    }

    fn on_key(
        &mut self,
        event: KeyEvent,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match event.code {
            KeyCode::Esc => self.back_to_request(context),

            KeyCode::Char(char) => match event.ctrl {
                true => match char {
                    'd' => self.scroll(state, elements, context, ScrollDirection::Down),
                    'u' => self.scroll(state, elements, context, ScrollDirection::Up),
                    'p' => self.previous_filter_match(state, elements, context),
                    'n' => self.next_filter_match(state, elements, context),
                    _ => {}
                },

                false => match char {
                    'b' => self.back_to_request(context),
                    'f' => {
                        context.set_focus("id", "response_body_input");
                        state.filter_input_focused = true;
                        info!("Set focus to response_body_input");

                        if !state.filter.to_ref().is_empty() {
                            let filter = state.filter.to_ref().to_string();
                            self.apply_response_filter(filter, state, context, elements);
                        }
                    }

                    'h' | 'y' | 'v' => {
                        let dashboard_message = DashboardMessages::KeyboardEvent(KeebEvent {
                            character: char,
                            ctrl: false,
                            state: match event.state {
                                KeyState::Press => KeebState::Press,
                                KeyState::Repeat => KeebState::Repeat,
                                KeyState::Release => KeebState::Release,
                            },
                        });

                        let Ok(ids) = self.component_ids.try_borrow() else {
                            return;
                        };

                        let Ok(message) = serde_json::to_string(&dashboard_message) else {
                            return;
                        };

                        let _ = send_message("dashboard", message, &ids, &context.emitter.clone());
                    }

                    _ => {}
                },
            },

            _ => {}
        }
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        // TODO: Fix this later, why is this ending up here?
        // This is ending up here after pressing T to create a new endpoint.
        // This messages should be going to request_body_input, but it is ending
        // up ad response_renderer for an unknown reason.
        if message.contains("SetInput") {
            return;
        }

        let response_renderer_message = serde_json::from_str::<ResponseRendererMessages>(&message);

        #[allow(clippy::single_match)]
        match response_renderer_message {
            Ok(message) => match message {
                ResponseRendererMessages::ThemeUpdate => {
                    self.update_app_theme(state);
                }

                ResponseRendererMessages::ResponseUpdate(extension) => {
                    // TODO: Try to delete this file if the program closes/quits/crashes
                    let reader_result = get_file_reader("/tmp/tome_response.txt");
                    if reader_result.is_err() {
                        println!("Error getting reader for response...");
                        return;
                    }

                    let response_reader = reader_result.unwrap();
                    self.response_reader = Some(response_reader);
                    self.render_response(extension, &mut elements, state, 0, context);
                }

                ResponseRendererMessages::SyntaxPreview(theme) => {
                    if theme.is_none() {
                        return;
                    }

                    if self.code_sample.is_none() {
                        self.code_sample = Some(String::from(CODE_SAMPLE));
                    }

                    if self.code_ext.is_none() {
                        self.code_ext = Some(String::from("rs"));
                    }

                    self.extension = self.code_ext.clone().unwrap();
                    let code = self.code_sample.clone().unwrap();
                    self.set_response(state, code, theme);
                }
            },

            Err(error) => {
                let error_message =
                    format!("There was an error handling a response message: {}", error);

                self.send_error_message(&error_message, context);
            }
        }
    }
}

fn clear_highlights(state: &mut ResponseRendererState) {
    let mut lines = state.lines.to_mut();

    lines.iter_mut().for_each(|line| {
        let mut l = line.to_mut();
        let mut spans = l.spans.to_mut();
        spans.iter_mut().for_each(|span| {
            let mut s = span.to_mut();
            let og_opt = *s.original_background.to_ref();
            if let Some(og_bg) = og_opt {
                s.background.set(og_bg);
                s.original_background.set(None);
            };

            let og_opt = *s.original_foreground.to_ref();
            if let Some(og_fg) = og_opt {
                s.foreground.set(og_fg);
                s.original_foreground.set(None);
            }
        });
    });
}

fn highlight_matches(
    state: &mut ResponseRendererState,
    match_range: (usize, usize),
    matches: &mut [usize],
    filter: &str,
) {
    info!("Highlighting");
    clear_highlights(state);

    let response = state.viewable_response.to_ref();
    let response_lines = response.lines().collect::<Vec<&str>>();
    let mut lines = state.lines.to_mut();

    matches.iter_mut().for_each(|match_index| {
        let view_index = match_index.saturating_sub(match_range.0);

        if *match_index < match_range.0 || *match_index > match_range.1 {
            return;
        }

        info!("Checking match_index: {match_index} for range: {match_range:?}");
        info!("Getting view_index: {view_index}, match_index: {match_index}");

        if let Some(matching_line) = response_lines.get(view_index) {
            let mut matched_display_line = lines.get_mut(view_index);

            if let Some(ref mut display_line_value) = matched_display_line {
                //info!("display line: {:?}", display_line_value.to_ref().spans.);
                let r = display_line_value
                    .to_ref()
                    .spans
                    .to_ref()
                    .iter()
                    .map(|span| {
                        let s = span.to_ref().text.to_ref().to_string();
                        info!("actual span: {s}");

                        s
                    })
                    .collect::<String>();
                info!("span: {:?}", r);

                let mut display_line = (*display_line_value).to_mut();

                let mut spans = display_line.spans.to_mut();

                info!("Applying highlighting to line: {matching_line}");
                matching_line.match_indices(filter).for_each(|(index, _)| {
                    let last_ndx = index + filter.len();
                    for span_ndx in index..last_ndx {
                        if let Some(span) = spans.get_mut(span_ndx) {
                            info!("span.to_ref().text: {:?}", span.to_ref().text.to_ref());

                            let mut s = span.to_mut();
                            let og_bg = Some(*s.background.to_ref());
                            s.original_background.set(og_bg);
                            s.background.set(Hex::from((255, 255, 0)));

                            let og_fg = Some(*s.foreground.to_ref());
                            s.original_foreground.set(og_fg);
                            s.foreground.set(Hex::from((0, 0, 0)));
                        }
                    }
                });
            };
        };
    });
}

fn get_file_reader(file_path: &str) -> anyhow::Result<BufReader<File>> {
    let file = File::open(file_path)?;
    Ok(BufReader::new(file))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseRendererMessages {
    ResponseUpdate(String),
    SyntaxPreview(Option<String>),
    ThemeUpdate,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TextFilter {
    pub indexes: Vec<usize>,
    pub total: usize,
    pub search_navigation_cursor: usize,
    pub filter: String,
}
