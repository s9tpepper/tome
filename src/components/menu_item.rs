use anathema::{
    component::Component,
    state::{State, Value},
};

use crate::theme::{get_app_theme, AppTheme};

pub const MENU_ITEM_TEMPLATE: &str = include_str!("./templates/menu_item.aml");

#[derive(Default)]
pub struct MenuItem;

impl MenuItem {
    #[allow(unused)]
    fn update_app_theme(&self, state: &mut MenuItemState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }
}

#[derive(Default, State)]
pub struct MenuItemState {
    label: Value<String>,
    key_binding: Value<char>,
    app_theme: Value<AppTheme>,
}

impl MenuItemState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        MenuItemState {
            label: "".to_string().into(),
            key_binding: ' '.into(),
            app_theme: app_theme.into(),
        }
    }
}

impl Component for MenuItem {
    type State = MenuItemState;
    type Message = ();

    fn accept_focus(&self) -> bool {
        false
    }
}
