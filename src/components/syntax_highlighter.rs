use core::panic;

use anathema::state::Hex;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

// use std::cmp::Ordering;

#[derive(Debug)]
pub struct Span<'a> {
    pub src: &'a str,
    pub fg: Hex,
    pub bold: bool,
}

impl<'a> Span<'a> {
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
        Self { src, fg, bold }
    }
}

#[derive(Debug)]
pub struct Line<'a> {
    pub head: Span<'a>,
    pub tail: Box<[Span<'a>]>,
}

// impl Line {
//     pub fn empty() -> Self {
//         Self {
//             spans: List::empty(),
//         }
//     }
// }

pub fn highlight<'a>(src: &'a str, ext: &str) -> Box<[Line<'a>]> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = ThemeSet::get_theme("themes/custom.stTheme").unwrap();

    // let ts = ThemeSet::load_defaults();
    // let theme = &ts.themes["base16-eighties.dark"];

    // let syntax = ps.find_syntax_by_extension(ext).unwrap();
    // NOTE: Hardcoded for testing, html content-type has encoding type after a ;
    let syntax = ps.find_syntax_by_extension("html").unwrap();

    let mut h = HighlightLines::new(syntax, &theme);

    let mut output = vec![];

    let mut n = 0;
    for line in LinesWithEndings::from(src) {
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

    output.into_boxed_slice()
}

pub struct Parser<'a> {
    lines: Box<[Line<'a>]>,
    instructions: Vec<Instruction>,
    foreground: Hex,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    MoveCursor(u16, u16),
    Type(char, bool),
    SetForeground(Hex),
    Newline { x: i32 },
    SetX(i32),
    Pause(u64),
    Wait,
    HideCursor,
}

impl<'a> Parser<'a> {
    pub fn new(lines: Box<[Line<'a>]>) -> Self {
        Self {
            lines,
            instructions: vec![],
            foreground: Hex::BLACK,
        }
    }

    pub fn instructions(mut self) -> Vec<Instruction> {
        let lines = std::mem::take(&mut self.lines);

        for line in &*lines {
            let mut line_start = 0;

            let (count, src, bold) = line.head.take_space();
            if let Some(x) = count {
                self.instructions.push(Instruction::SetX(x));
                line_start = x;
            } else {
                self.instructions.push(Instruction::SetX(0));
            }

            self.set_foreground(&line.head);
            self.push_chars(src, bold, line_start);

            for span in &*line.tail {
                self.set_foreground(span);
                self.push_chars(span.src, span.bold, line_start);
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