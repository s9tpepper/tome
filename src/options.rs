use std::{
    fs::{self, File},
    io::BufReader,
};

use serde::{Deserialize, Serialize};

use crate::fs::get_app_dir;

const DEFAULT_SYNTAX_THEME: &str = "monokai";
const DEFAULT_APP_THEME: &str = "gruvbox";
const SYNTAX_THEMES_LIST: &str = include_str!("../themes/themes.txt");
const BUTTONS_ANGLED: (&str, &str) = ("█", "█");
const BUTTONS_SQUARED: (&str, &str) = (" █", "█ ");
const BUTTONS_ROUNDED: (&str, &str) = (" █", "█ ");

pub const BUTTON_STYLE_ANGLED: &str = "Angled ██";
pub const BUTTON_STYLE_SQUARED: &str = "Squared  ███";
pub const BUTTON_STYLE_ROUNDED: &str = "Rounded ██";

#[derive(Default, Debug, Deserialize, Serialize)]
pub enum ButtonStyle {
    Angled,
    Squared,

    #[default]
    Rounded,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Options {
    pub syntax_theme: String,
    pub app_theme_name: String,
    pub button_style: Option<ButtonStyle>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SyntaxTheme {
    pub name: String,
}

pub fn get_syntax_themes() -> Vec<String> {
    let themes: std::str::Lines<'_> = SYNTAX_THEMES_LIST.lines();
    let iterator = themes.into_iter().map(|st| st.to_string());

    iterator.collect()
}

pub fn get_syntax_theme() -> String {
    get_options().syntax_theme
}

pub fn get_app_theme_name() -> String {
    get_options().app_theme_name
}

pub fn get_default_options() -> Options {
    Options {
        syntax_theme: String::from(DEFAULT_SYNTAX_THEME),
        app_theme_name: String::from(DEFAULT_APP_THEME),
        ..Default::default()
    }
}

pub fn get_button_style() -> String {
    match get_options().button_style {
        Some(style) => match style {
            ButtonStyle::Angled => BUTTON_STYLE_ANGLED,
            ButtonStyle::Squared => BUTTON_STYLE_SQUARED,
            ButtonStyle::Rounded => BUTTON_STYLE_ROUNDED,
        },

        None => BUTTON_STYLE_ANGLED,
    }
    .to_string()
}

pub fn get_button_caps() -> (&'static str, &'static str) {
    match get_options().button_style {
        Some(style) => match style {
            ButtonStyle::Angled => BUTTONS_ANGLED,
            ButtonStyle::Squared => BUTTONS_SQUARED,
            ButtonStyle::Rounded => BUTTONS_ROUNDED,
        },

        None => BUTTONS_ANGLED,
    }
}

pub fn get_options() -> Options {
    match get_app_dir("options") {
        Ok(mut options_dir) => {
            options_dir.push("options.json");

            match File::open(options_dir) {
                Ok(file) => {
                    let reader = BufReader::new(file);

                    let options = serde_json::from_reader(reader);

                    match options {
                        Ok(opts) => opts,
                        Err(_) => get_default_options(),
                    }
                }
                Err(_) => get_default_options(),
            }
        }

        Err(_) => get_default_options(),
    }
}

pub fn save_options(options: Options) -> anyhow::Result<()> {
    match get_app_dir("options") {
        Ok(mut directory) => {
            directory.push("options.json");

            let contents = serde_json::to_string(&options)?;

            Ok(fs::write(directory, contents)?)
        }
        Err(_) => Err(anyhow::Error::msg("Could not save options")),
    }
}
