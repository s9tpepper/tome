use anathema::state::{State, Value};
use serde::{Deserialize, Serialize};

use crate::{
    app_themes::{APP_THEME_MAP, CATPPUCCIN_THEME},
    options::get_app_theme_name,
};

#[derive(Debug, Default, State)]
pub struct AppTheme {
    /// Theme name
    pub name: Value<String>,
    /// Application background
    pub background: Value<String>,
    /// Main font color
    pub foreground: Value<String>,
    /// Color of borders when not focused
    pub border_unfocused: Value<String>,
    /// Color of borders when focused
    pub border_focused: Value<String>,
    /// Color of top bar background
    pub top_bar_background: Value<String>,
    /// Color of top bar font color
    pub top_bar_foreground: Value<String>,
    /// Color of bottom bar background
    pub bottom_bar_background: Value<String>,
    /// Color of bottom bar font color
    pub bottom_bar_foreground: Value<String>,
    /// Color of background for menu options
    pub menu_opt_background: Value<String>,
    /// Color of foreground for menu options
    pub menu_opt_foreground: Value<String>,
    /// Color of project name top bar background
    pub project_name_background: Value<String>,
    /// Color of project name top bar title
    pub project_name_foreground: Value<String>,
    /// Color of endpoint name top bar background
    pub endpoint_name_background: Value<String>,
    /// Color of endpoint name top bar title
    pub endpoint_name_foreground: Value<String>,

    pub menu_color_1: Value<String>,
    pub menu_color_2: Value<String>,
    pub menu_color_3: Value<String>,
    pub menu_color_4: Value<String>,
    pub menu_color_5: Value<String>,

    pub overlay_background: Value<String>,
    pub overlay_foreground: Value<String>,
    pub overlay_heading: Value<String>,
    pub overlay_submit_background: Value<String>,
    pub overlay_submit_foreground: Value<String>,
    pub overlay_cancel_background: Value<String>,
    pub overlay_cancel_foreground: Value<String>,

    pub row_color: Value<String>,
}

#[derive(Debug, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct AppThemePersisted {
    /// Theme name
    pub name: String,
    /// Application background
    pub background: String,
    /// Main font color
    pub foreground: String,
    /// Color of borders when not focused
    pub border_unfocused: String,
    /// Color of borders when focused
    pub border_focused: String,
    /// Color of top bar background
    pub top_bar_background: String,
    /// Color of top bar font color
    pub top_bar_foreground: String,
    /// Color of bottom bar background
    pub bottom_bar_background: String,
    /// Color of bottom bar font color
    pub bottom_bar_foreground: String,
    /// Color of background for menu options
    pub menu_opt_background: String,
    /// Color of foreground for menu options
    pub menu_opt_foreground: String,
    /// Color of project name top bar background
    pub project_name_background: String,
    /// Color of project name top bar title
    pub project_name_foreground: String,
    /// Color of endpoint name top bar background
    pub endpoint_name_background: String,
    /// Color of endpoint name top bar title
    pub endpoint_name_foreground: String,

    pub menu_color_1: String,
    pub menu_color_2: String,
    pub menu_color_3: String,
    pub menu_color_4: String,
    pub menu_color_5: String,
    pub overlay_background: String,
    pub overlay_foreground: String,
    pub overlay_heading: String,
    pub overlay_submit_background: String,
    pub overlay_submit_foreground: String,
    pub overlay_cancel_background: String,
    pub overlay_cancel_foreground: String,
}

impl From<AppTheme> for AppThemePersisted {
    fn from(theme: AppTheme) -> Self {
        AppThemePersisted {
            name: theme.name.to_ref().to_string(),
            background: theme.background.to_ref().to_string(),
            foreground: theme.foreground.to_ref().to_string(),
            border_unfocused: theme.border_unfocused.to_ref().to_string(),
            border_focused: theme.border_focused.to_ref().to_string(),
            top_bar_background: theme.top_bar_background.to_ref().to_string(),
            top_bar_foreground: theme.top_bar_foreground.to_ref().to_string(),
            bottom_bar_background: theme.bottom_bar_background.to_ref().to_string(),
            bottom_bar_foreground: theme.bottom_bar_foreground.to_ref().to_string(),
            menu_opt_background: theme.menu_opt_background.to_ref().to_string(),
            menu_opt_foreground: theme.menu_opt_foreground.to_ref().to_string(),
            project_name_background: theme.project_name_background.to_ref().to_string(),
            project_name_foreground: theme.project_name_foreground.to_ref().to_string(),
            endpoint_name_background: theme.endpoint_name_background.to_ref().to_string(),
            endpoint_name_foreground: theme.endpoint_name_foreground.to_ref().to_string(),
            menu_color_1: theme.menu_color_1.to_ref().to_string(),
            menu_color_2: theme.menu_color_2.to_ref().to_string(),
            menu_color_3: theme.menu_color_3.to_ref().to_string(),
            menu_color_4: theme.menu_color_4.to_ref().to_string(),
            menu_color_5: theme.menu_color_5.to_ref().to_string(),
            overlay_background: theme.overlay_background.to_ref().to_string(),
            overlay_foreground: theme.overlay_foreground.to_ref().to_string(),
            overlay_heading: theme.overlay_heading.to_ref().to_string(),
            overlay_submit_background: theme.overlay_submit_background.to_ref().to_string(),
            overlay_submit_foreground: theme.overlay_submit_foreground.to_ref().to_string(),
            overlay_cancel_background: theme.overlay_cancel_background.to_ref().to_string(),
            overlay_cancel_foreground: theme.overlay_cancel_foreground.to_ref().to_string(),
        }
    }
}

impl From<AppThemePersisted> for AppTheme {
    fn from(theme_persisted: AppThemePersisted) -> Self {
        AppTheme {
            name: theme_persisted.name.into(),
            background: theme_persisted.background.into(),
            foreground: theme_persisted.foreground.into(),
            border_unfocused: theme_persisted.border_unfocused.clone().into(),
            border_focused: theme_persisted.border_focused.into(),
            top_bar_background: theme_persisted.top_bar_background.into(),
            top_bar_foreground: theme_persisted.top_bar_foreground.into(),
            bottom_bar_background: theme_persisted.bottom_bar_background.into(),
            bottom_bar_foreground: theme_persisted.bottom_bar_foreground.into(),
            menu_opt_background: theme_persisted.menu_opt_background.into(),
            menu_opt_foreground: theme_persisted.menu_opt_foreground.into(),
            project_name_background: theme_persisted.project_name_background.into(),
            project_name_foreground: theme_persisted.project_name_foreground.into(),
            endpoint_name_background: theme_persisted.endpoint_name_background.into(),
            endpoint_name_foreground: theme_persisted.endpoint_name_foreground.into(),
            menu_color_1: theme_persisted.menu_color_1.into(),
            menu_color_2: theme_persisted.menu_color_2.into(),
            menu_color_3: theme_persisted.menu_color_3.into(),
            menu_color_4: theme_persisted.menu_color_4.into(),
            menu_color_5: theme_persisted.menu_color_5.into(),
            overlay_background: theme_persisted.overlay_background.into(),
            overlay_foreground: theme_persisted.overlay_foreground.into(),
            overlay_heading: theme_persisted.overlay_heading.into(),
            overlay_submit_background: theme_persisted.overlay_submit_background.into(),
            overlay_submit_foreground: theme_persisted.overlay_submit_foreground.into(),
            overlay_cancel_background: theme_persisted.overlay_cancel_background.into(),
            overlay_cancel_foreground: theme_persisted.overlay_cancel_foreground.into(),
            row_color: theme_persisted.border_unfocused.into(),
        }
    }
}

pub fn get_app_themes_list() -> Vec<AppThemePersisted> {
    let mut list: Vec<AppThemePersisted> = APP_THEME_MAP
        .values()
        .flat_map(|app_theme_bytes| {
            let app_theme_contents = String::from_utf8_lossy(app_theme_bytes);

            serde_json::from_str::<AppThemePersisted>(&app_theme_contents)
        })
        .collect();

    list.sort();

    list
}

pub fn get_app_theme() -> AppTheme {
    let app_theme_name = get_app_theme_name();

    get_app_theme_by_name(&app_theme_name)
}

pub fn get_app_theme_persisted() -> AppThemePersisted {
    let app_theme_name = get_app_theme_name();

    get_app_theme_persisted_by_name(&app_theme_name)
}

pub fn get_app_theme_by_name(app_theme_name: &str) -> AppTheme {
    let app_theme_opt = APP_THEME_MAP.get_key_value(app_theme_name);

    match app_theme_opt {
        Some((_, app_theme_bytes)) => {
            let app_theme_contents = String::from_utf8_lossy(app_theme_bytes);

            let app_theme = serde_json::from_str::<AppThemePersisted>(&app_theme_contents);
            match app_theme {
                Ok(theme) => theme.into(),
                Err(_) => get_default_app_theme(),
            }
        }

        None => get_default_app_theme(),
    }
}

pub fn get_app_theme_persisted_by_name(app_theme_name: &str) -> AppThemePersisted {
    let app_theme_opt = APP_THEME_MAP.get_key_value(app_theme_name);

    match app_theme_opt {
        Some((_, app_theme_bytes)) => {
            let app_theme_contents = String::from_utf8_lossy(app_theme_bytes);

            let app_theme = serde_json::from_str::<AppThemePersisted>(&app_theme_contents);
            match app_theme {
                Ok(theme) => theme,
                Err(_) => get_default_app_theme_persisted(),
            }
        }

        None => get_default_app_theme_persisted(),
    }
}

fn get_default_app_theme_persisted() -> AppThemePersisted {
    let default_app_theme_contents = String::from_utf8_lossy(CATPPUCCIN_THEME);
    serde_json::from_str::<AppThemePersisted>(&default_app_theme_contents).unwrap()
}

fn get_default_app_theme() -> AppTheme {
    let default_app_theme_contents = String::from_utf8_lossy(CATPPUCCIN_THEME);
    serde_json::from_str::<AppThemePersisted>(&default_app_theme_contents)
        .unwrap()
        .into()
}

pub fn update_component_theme(state_theme: &mut Value<AppTheme>) {
    let app_theme = get_app_theme_persisted();

    let mut at = state_theme.to_mut();
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

// impl Serialize for AppTheme {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut state = serializer.serialize_struct("AppTheme", 9)?;
//         state.serialize_field("name", &*self.name.to_ref())?;
//         state.serialize_field("background", &*self.background.to_ref())?;
//         state.serialize_field("foreground", &*self.foreground.to_ref())?;
//         state.serialize_field("border_unfocused", &*self.border_unfocused.to_ref())?;
//         state.serialize_field("border_focused", &*self.border_focused.to_ref())?;
//         state.serialize_field("menu_opt_background", &*self.top_bar_background.to_ref())?;
//         state.serialize_field("menu_opt_foreground", &*self.top_bar_foreground.to_ref())?;
//         state.serialize_field("top_bar_background", &*self.top_bar_background.to_ref())?;
//         state.serialize_field("top_bar_foreground", &*self.top_bar_foreground.to_ref())?;
//         state.serialize_field(
//             "bottom_bar_background",
//             &*self.bottom_bar_background.to_ref(),
//         )?;
//         state.serialize_field(
//             "bottom_bar_foreground",
//             &*self.bottom_bar_foreground.to_ref(),
//         )?;
//         state.end()
//     }
// }

// impl<'de> Deserialize<'de> for AppTheme {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         enum Field {
//             Name,
//             Background,
//             Foreground,
//             BorderUnfocused,
//             BorderFocused,
//             TopBarBackground,
//             TopBarForeground,
//             BottomBarBackground,
//             BottomBarForeground,
//             MenuOptBackground,
//             MenuOptForeground,
//         }
//         impl<'de> Deserialize<'de> for Field {
//             fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
//             where
//                 D: Deserializer<'de>,
//             {
//                 struct FieldVisitor;
//
//                 impl<'de> Visitor<'de> for FieldVisitor {
//                     type Value = Field;
//
//                     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                         formatter.write_str("`name` or `background`...")
//                     }
//
//                     fn visit_str<E>(self, value: &str) -> Result<Field, E>
//                     where
//                         E: de::Error,
//                     {
//                         match value {
//                             "name" => Ok(Field::Name),
//                             "background" => Ok(Field::Background),
//                             "foreground" => Ok(Field::Foreground),
//                             "border_unfocused" => Ok(Field::BorderUnfocused),
//                             "border_focused" => Ok(Field::BorderFocused),
//                             "top_bar_background" => Ok(Field::TopBarBackground),
//                             "top_bar_foreground" => Ok(Field::TopBarForeground),
//                             "bottom_bar_background" => Ok(Field::BottomBarBackground),
//                             "bottom_bar_foreground" => Ok(Field::BottomBarForeground),
//                             "menu_opt_background" => Ok(Field::MenuOptBackground),
//                             "menu_opt_foreground" => Ok(Field::MenuOptForeground),
//
//                             _ => Err(de::Error::unknown_field(value, FIELDS)),
//                         }
//                     }
//                 }
//
//                 deserializer.deserialize_identifier(FieldVisitor)
//             }
//         }
//
//         struct AppThemeVisitor;
//         impl<'de> Visitor<'de> for AppThemeVisitor {
//             type Value = AppTheme;
//
//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 formatter.write_str("struct AppTheme")
//             }
//
//             fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
//             where
//                 A: MapAccess<'de>,
//             {
//                 let mut name = None;
//                 let mut background = None;
//                 let mut foreground = None;
//                 let mut border_unfocused = None;
//                 let mut border_focused = None;
//                 let mut top_bar_background = None;
//                 let mut top_bar_foreground = None;
//                 let mut bottom_bar_background = None;
//                 let mut bottom_bar_foreground = None;
//                 let mut menu_opt_background = None;
//                 let mut menu_opt_foreground = None;
//
//                 while let Some(key) = map.next_key::<Field>()? {
//                     match key {
//                         Field::Name => {
//                             if name.is_some() {
//                                 return Err(de::Error::duplicate_field("name"));
//                             }
//
//                             name = Some(Value::new(map.next_value()?));
//                         }
//                         Field::Background => {
//                             if background.is_some() {
//                                 return Err(de::Error::duplicate_field("background"));
//                             }
//
//                             background = Some(Value::new(map.next_value()?));
//                         }
//                         Field::Foreground => {
//                             if foreground.is_some() {
//                                 return Err(de::Error::duplicate_field("foreground"));
//                             }
//
//                             foreground = Some(Value::new(map.next_value()?));
//                         }
//                         Field::BorderUnfocused => {
//                             if border_unfocused.is_some() {
//                                 return Err(de::Error::duplicate_field("border_unfocused"));
//                             }
//
//                             border_unfocused = Some(Value::new(map.next_value()?));
//                         }
//                         Field::BorderFocused => {
//                             if border_focused.is_some() {
//                                 return Err(de::Error::duplicate_field("border_focused"));
//                             }
//
//                             border_focused = Some(Value::new(map.next_value()?));
//                         }
//                         Field::TopBarBackground => {
//                             if top_bar_background.is_some() {
//                                 return Err(de::Error::duplicate_field("top_bar_background"));
//                             }
//
//                             top_bar_background = Some(Value::new(map.next_value()?));
//                         }
//                         Field::TopBarForeground => {
//                             if top_bar_foreground.is_some() {
//                                 return Err(de::Error::duplicate_field("top_bar_foreground"));
//                             }
//
//                             top_bar_foreground = Some(Value::new(map.next_value()?));
//                         }
//                         Field::BottomBarBackground => {
//                             if bottom_bar_background.is_some() {
//                                 return Err(de::Error::duplicate_field("bottom_bar_background"));
//                             }
//
//                             bottom_bar_background = Some(Value::new(map.next_value()?));
//                         }
//                         Field::BottomBarForeground => {
//                             if bottom_bar_foreground.is_some() {
//                                 return Err(de::Error::duplicate_field("bottom_bar_foreground"));
//                             }
//
//                             bottom_bar_foreground = Some(Value::new(map.next_value()?));
//                         }
//                         Field::MenuOptBackground => {
//                             if menu_opt_background.is_some() {
//                                 return Err(de::Error::duplicate_field("menu_opt_background"));
//                             }
//
//                             menu_opt_background = Some(Value::new(map.next_value()?));
//                         }
//                         Field::MenuOptForeground => {
//                             if menu_opt_foreground.is_some() {
//                                 return Err(de::Error::duplicate_field("menu_opt_foreground"));
//                             }
//
//                             menu_opt_foreground = Some(Value::new(map.next_value()?));
//                         }
//                     }
//                 }
//
//                 let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
//                 let background =
//                     background.ok_or_else(|| de::Error::missing_field("background"))?;
//                 let foreground =
//                     foreground.ok_or_else(|| de::Error::missing_field("foreground"))?;
//                 let border_unfocused =
//                     border_unfocused.ok_or_else(|| de::Error::missing_field("border_unfocused"))?;
//                 let border_focused =
//                     border_focused.ok_or_else(|| de::Error::missing_field("border_focused"))?;
//                 let top_bar_background = top_bar_background
//                     .ok_or_else(|| de::Error::missing_field("top_bar_background"))?;
//                 let top_bar_foreground = top_bar_foreground
//                     .ok_or_else(|| de::Error::missing_field("top_bar_foreground"))?;
//                 let bottom_bar_background = bottom_bar_background
//                     .ok_or_else(|| de::Error::missing_field("bottom_bar_background"))?;
//                 let bottom_bar_foreground = bottom_bar_foreground
//                     .ok_or_else(|| de::Error::missing_field("bottom_bar_foreground"))?;
//                 let menu_opt_background = menu_opt_background
//                     .ok_or_else(|| de::Error::missing_field("menu_opt_background"))?;
//                 let menu_opt_foreground = menu_opt_foreground
//                     .ok_or_else(|| de::Error::missing_field("menu_opt_foreground"))?;
//
//                 Ok(AppTheme {
//                     name,
//                     background,
//                     foreground,
//                     border_unfocused,
//                     border_focused,
//                     top_bar_background,
//                     top_bar_foreground,
//                     bottom_bar_background,
//                     bottom_bar_foreground,
//                     menu_opt_background,
//                     menu_opt_foreground,
//                 })
//             }
//         }
//
//         const FIELDS: &[&str] = &[
//             "name",
//             "background",
//             "foreground",
//             "border_unfocused",
//             "top_bar_background",
//             "top_bar_foreground",
//             "bottom_bar_background",
//             "bottom_bar_foreground",
//         ];
//         deserializer.deserialize_struct("AppTheme", FIELDS, AppThemeVisitor)
//     }
// }
