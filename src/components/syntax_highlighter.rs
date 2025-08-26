use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Cursor;
use std::sync::LazyLock;

use anathema::state::Hex;
use log::info;
use once_cell::sync::Lazy;
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

static THEME_CACHE: Lazy<HashMap<String, Theme>> = Lazy::new(|| {
    let mut map = HashMap::new();

    THEME_MAP.iter().for_each(|(name, theme_bytes)| {
        let mut cursor = Cursor::new(*theme_bytes);
        let theme = ThemeSet::load_from_reader(&mut cursor).unwrap();

        map.insert(name.to_string(), theme);
    });

    map
});

pub fn get_highlight_theme(name: Option<String>) -> &'static Theme {
    let theme_name = name
        .unwrap_or(get_syntax_theme())
        .to_uppercase()
        .replace("[ ", "")
        .replace(" ]", "")
        .replace(" ", "_");

    THEME_CACHE
        .get(&theme_name)
        .expect("all themes should exist")
}

thread_local! {
    static PS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    // static PS: SyntaxSet = SyntaxSet::new();
}

pub fn highlight<'a>(
    src: &'a str,
    ext: &str,
    name: Option<String>,
) -> (Box<[Line<'a>]>, &'static Theme) {
    let theme = get_highlight_theme(name);

    let mut extension = ext;
    if ext.contains(";")
        && let Some((ex, _)) = ext.split_once(';')
    {
        extension = ex;
    }

    let output = PS.with(|ps| {
        let syntax = ps
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        let mut h = HighlightLines::new(syntax, theme);
        let mut output = vec![];

        for line in LinesWithEndings::from(src) {
            // info!("Highlinting this slice: {line}");

            let mut head = h
                .highlight_line(line, ps)
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

        output
    });

    info!("output length: {}", output.len());
    (output.into_boxed_slice(), theme)
}
