use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::LazyLock;

use anathema::state::Hex;
use log::info;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::options::get_syntax_theme;
use crate::themes::{MONOKAI_DARK, THEME_MAP};

#[derive(Debug)]
pub struct Span<'a> {
    pub src: &'a str,
    pub fg: Hex,
    pub bg: Hex,
    pub bold: bool,
}

impl Span<'_> {
    pub fn take_space(&self) -> (Option<i32>, &str, bool) {
        let count = self.src.bytes().take_while(|b| *b == b' ').count();

        let opt_count = match count {
            0 => None,
            n => Some(n as i32),
        };

        (opt_count, &self.src[count..], self.bold)
    }
}

impl<'a> From<(Style, &'a str)> for Span<'a> {
    fn from((style, src): (Style, &'a str)) -> Self {
        let bold = style.font_style.contains(FontStyle::BOLD);
        let fg = (style.foreground.r, style.foreground.g, style.foreground.b).into();
        let bg = (style.background.r, style.background.g, style.background.b).into();

        Self { src, fg, bg, bold }
    }
}

#[derive(Debug)]
pub struct Line<'a> {
    pub head: Span<'a>,
    pub tail: Box<[Span<'a>]>,
}

pub fn get_constant_from_name(name: &str) -> String {
    name.to_uppercase()
        .replace("'", "")
        .replace("[", "")
        .replace("]", "")
        .replace("(", "")
        .replace(")", "")
        .replace("&", "")
        .replace("-", "")
        .trim()
        .replace(" ", "_")
        .replace("__", "_")
}

#[allow(clippy::declare_interior_mutable_const)]
const THEME_CACHE: LazyLock<RefCell<HashMap<String, Theme>>> =
    LazyLock::<RefCell<HashMap<String, Theme>>>::new(|| {
        let theme_map: RefCell<HashMap<String, Theme>> = RefCell::new(HashMap::new());

        theme_map
    });

pub fn get_highlight_theme(name: Option<String>) -> Theme {
    let theme_name = name.unwrap_or(get_syntax_theme());
    let cache = THEME_CACHE;
    let borrowed_cache = cache.try_borrow_mut();

    match borrowed_cache {
        Ok(mut cache) => {
            let theme = cache.get(theme_name.as_str());

            match theme {
                Some(theme) => theme.clone(),
                None => {
                    let const_name = get_constant_from_name(&theme_name);

                    let theme_arr = THEME_MAP.get_key_value(&const_name.as_ref());
                    let default_theme = &MONOKAI_DARK.as_ref();
                    let (_, theme_bytes) = theme_arr.unwrap_or((&"PLUM_DUMB", default_theme));

                    let mut cursor = Cursor::new(*theme_bytes);
                    let theme = ThemeSet::load_from_reader(&mut cursor);

                    let theme = theme.unwrap();
                    let name = theme_name.clone();
                    cache.insert(name.to_owned(), theme.clone());

                    theme
                }
            }
        }

        Err(_) => {
            let const_name = get_constant_from_name(&theme_name);

            let theme_arr = THEME_MAP.get_key_value(&const_name.as_ref());
            let default_theme = &MONOKAI_DARK.as_ref();
            let (_, theme_bytes) = theme_arr.unwrap_or((&"PLUM_DUMB", default_theme));

            let mut cursor = Cursor::new(*theme_bytes);
            let theme = ThemeSet::load_from_reader(&mut cursor);

            theme.unwrap()
        }
    }
}

pub fn highlight<'a>(src: &'a str, ext: &str, name: Option<String>) -> (Box<[Line<'a>]>, Theme) {
    let ps = SyntaxSet::load_defaults_newlines();
    let theme = get_highlight_theme(name);

    let mut extension = ext;
    if ext.contains(";") {
        if let Some((ex, _)) = ext.split_once(';') {
            extension = ex;
        }
    }

    let syntax = ps.find_syntax_by_extension(extension).unwrap();
    let mut h = HighlightLines::new(syntax, &theme);
    let mut output = vec![];

    for line in LinesWithEndings::from(src) {
        info!("Highlinting this slice: {line}");

        let mut head = h
            .highlight_line(line, &ps)
            .unwrap()
            .into_iter()
            .map(Span::from)
            .collect::<Vec<_>>();

        let tail = head.split_off(1);

        let head = head.remove(0);
        output.push(Line {
            tail: tail.into_boxed_slice(),
            head,
        });
    }

    info!("output length: {}", output.len());
    (output.into_boxed_slice(), theme)
}

pub struct Parser<'a> {
    lines: Box<[Line<'a>]>,
    instructions: Vec<Instruction>,
    foreground: Hex,
    background: Hex,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Type(char, bool),
    SetForeground(Hex),
    SetBackground(Hex),
    Newline { x: i32 },
    SetX(i32),
}

impl<'a> Parser<'a> {
    pub fn new(lines: Box<[Line<'a>]>) -> Self {
        Self {
            lines,
            instructions: vec![],
            foreground: Hex::WHITE,
            background: Hex::BLACK,
        }
    }

    pub fn instructions(mut self) -> Vec<Instruction> {
        let lines = std::mem::take(&mut self.lines);

        for line in &*lines {
            let mut line_start = 0;

            let (count, src, bold) = line.head.take_space();
            if let Some(x) = count {
                // self.instructions.push(Instruction::SetX(x));
                line_start = x;
            } else {
                self.instructions.push(Instruction::SetX(0));
            }

            self.set_foreground(&line.head);
            self.set_background(&line.head);

            if line_start == 0 {
                self.push_chars(src, bold, line_start);
            } else {
                let mut spaces = "".to_string();
                for _ in 0..line_start {
                    spaces.push(' ');
                }

                self.push_chars(&spaces, bold, 0);
            }

            for span in &*line.tail {
                self.set_foreground(span);
                self.set_background(span);
                self.push_chars(span.src, span.bold, 0);
            }
        }

        self.instructions
    }

    fn set_foreground(&mut self, span: &Span) {
        if span.fg != self.foreground {
            self.instructions.push(Instruction::SetForeground(span.fg));
            self.foreground = span.fg;
        }
    }

    fn set_background(&mut self, span: &Span) {
        if span.bg != self.background {
            self.instructions.push(Instruction::SetBackground(span.bg));
            self.background = span.bg;
        }
    }

    fn push_chars(&mut self, src: &str, bold: bool, line_start: i32) {
        for c in src.chars() {
            match c {
                '\n' => self
                    .instructions
                    .push(Instruction::Newline { x: line_start }),
                c => self.instructions.push(Instruction::Type(c, bold)),
            }
        }
    }
}
