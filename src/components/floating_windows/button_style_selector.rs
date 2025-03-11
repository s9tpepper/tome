use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{Component, ComponentId},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{List, State, Value},
};

use crate::{
    app::GlobalEventHandler,
    options::{get_button_style, BUTTON_STYLE_ANGLED, BUTTON_STYLE_ROUNDED, BUTTON_STYLE_SQUARED},
    projects::DEFAULT_ROW_COLOR,
    templates::template,
    theme::{get_app_theme, AppTheme},
};

// TODO: Fix the default project row color to the correct gray
const DEFAULT_PROJECT_ROW_COLOR: &str = "#333333";
const SELECTED_PROJECT_ROW_COLOR: &str = "#FFFFFF";

#[derive(Default, State)]
struct ButtonStyleState {
    name: Value<String>,
    row_color: Value<String>,
}

#[derive(Default, State)]
pub struct ButtonStyleSelectorState {
    cursor: Value<u8>,
    visible_items: Value<u8>,
    window_list: Value<List<ButtonStyleState>>,
    selected_button_style: Value<String>,
    app_theme: Value<AppTheme>,
}

impl ButtonStyleSelectorState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        let button_styles: Vec<ButtonStyleState> = vec![
            ButtonStyleState {
                name: BUTTON_STYLE_ANGLED.to_string().into(),
                row_color: DEFAULT_ROW_COLOR.to_string().into(),
            },
            ButtonStyleState {
                name: BUTTON_STYLE_ROUNDED.to_string().into(),
                row_color: DEFAULT_ROW_COLOR.to_string().into(),
            },
            ButtonStyleState {
                name: BUTTON_STYLE_SQUARED.to_string().into(),
                row_color: DEFAULT_ROW_COLOR.to_string().into(),
            },
        ];

        ButtonStyleSelectorState {
            cursor: 0.into(),
            visible_items: 5.into(),
            window_list: List::from_iter(button_styles),
            selected_button_style: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

#[derive(Default)]
pub struct ButtonStyleSelector {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl ButtonStyleSelector {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
    ) -> anyhow::Result<()> {
        let id = builder.register_component(
            "button_style_selector",
            template("floating_windows/templates/button_style_selector"),
            ButtonStyleSelector::new(ids.clone()),
            ButtonStyleSelectorState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("button_style_selector"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut ButtonStyleSelectorState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        ButtonStyleSelector { component_ids }
    }

    fn move_cursor_down(&self, state: &mut ButtonStyleSelectorState) {
        let last_complete_list_index = state.window_list.len().saturating_sub(1);
        let new_cursor = min(*state.cursor.to_ref() + 1, last_complete_list_index as u8);
        state.cursor.set(new_cursor);

        self.update_item_list(new_cursor.into(), state);
    }

    fn move_cursor_up(&self, state: &mut ButtonStyleSelectorState) {
        let new_cursor = max(state.cursor.to_ref().saturating_sub(1), 0);
        state.cursor.set(new_cursor);

        self.update_item_list(new_cursor.into(), state);
    }

    fn update_item_list(&self, selected_index: usize, state: &mut ButtonStyleSelectorState) {
        state.window_list.to_mut().iter_mut().enumerate().for_each(
            |(index, button_style_state)| {
                if index == selected_index {
                    button_style_state
                        .to_mut()
                        .row_color
                        .set(SELECTED_PROJECT_ROW_COLOR.to_string());
                } else {
                    button_style_state
                        .to_mut()
                        .row_color
                        .set(DEFAULT_PROJECT_ROW_COLOR.to_string());
                }
            },
        );
    }
}

impl Component for ButtonStyleSelector {
    type State = ButtonStyleSelectorState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match event.code {
            anathema::component::KeyCode::Char(char) => match char {
                'j' => self.move_cursor_down(state),
                'k' => self.move_cursor_up(state),
                _ => {}
            },

            anathema::component::KeyCode::Up => self.move_cursor_up(state),
            anathema::component::KeyCode::Down => self.move_cursor_down(state),

            anathema::component::KeyCode::Esc => {
                // NOTE: This sends cursor to satisfy publish() but is not used
                context.publish("button_style_selector__cancel", |state| &state.cursor)
            }

            anathema::component::KeyCode::Enter => {
                let selected_index = *state.cursor.to_ref() as usize;
                let window_list = state.window_list.to_ref();
                let app_theme_persisted = window_list.get(selected_index);

                match app_theme_persisted {
                    Some(button_style_state) => {
                        state
                            .selected_button_style
                            .set(button_style_state.to_ref().name.to_ref().to_string());

                        context.publish("button_style_selector__selection", |state| {
                            &state.selected_button_style
                        });
                    }
                    None => context.publish("button_style_selector__cancel", |state| &state.cursor),
                }
            }

            _ => {}
        }
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);

        let current_button_style = get_button_style();

        state
            .window_list
            .to_mut()
            .iter_mut()
            .for_each(|button_style_state| {
                let btn_style = button_style_state.to_ref().name.to_ref().to_string();
                if btn_style == current_button_style {
                    button_style_state
                        .to_mut()
                        .row_color
                        .set(SELECTED_PROJECT_ROW_COLOR.to_string());
                } else {
                    button_style_state
                        .to_mut()
                        .row_color
                        .set(DEFAULT_PROJECT_ROW_COLOR.to_string());
                }
            });
    }
}
