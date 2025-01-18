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
    templates::template,
    theme::{
        get_app_theme, get_app_theme_by_name, get_app_themes_list, AppTheme, AppThemePersisted,
    },
};

// TODO: Fix the default project row color to the correct gray
const DEFAULT_PROJECT_ROW_COLOR: &str = "#333333";
const SELECTED_PROJECT_ROW_COLOR: &str = "#FFFFFF";

#[derive(Default, State)]
pub struct AppThemeSelectorState {
    cursor: Value<u8>,
    current_first_index: Value<u8>,
    current_last_index: Value<u8>,
    visible_items: Value<u8>,
    window_list: Value<List<AppTheme>>,
    app_themes_count: Value<u8>,
    selected_app_theme: Value<String>,
    app_theme: Value<AppTheme>,
}

impl AppThemeSelectorState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        AppThemeSelectorState {
            cursor: 0.into(),
            app_themes_count: 0.into(),
            current_first_index: 0.into(),
            current_last_index: 4.into(),
            visible_items: 5.into(),
            window_list: List::empty(),
            selected_app_theme: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

#[derive(Default)]
pub struct AppThemeSelector {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    app_theme_list: Vec<AppThemePersisted>,
}

impl AppThemeSelector {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let id = builder.register_component(
            "app_theme_selector",
            template("floating_windows/templates/app_theme_selector"),
            AppThemeSelector::new(ids.clone()),
            AppThemeSelectorState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("app_theme_selector"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut AppThemeSelectorState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        AppThemeSelector {
            component_ids,
            app_theme_list: vec![],
        }
    }

    fn load(&mut self, state: &mut AppThemeSelectorState) -> anyhow::Result<()> {
        self.app_theme_list = get_app_themes_list();
        state.app_themes_count.set(self.app_theme_list.len() as u8);

        Ok(())
    }

    fn move_cursor_down(&self, state: &mut AppThemeSelectorState) {
        let last_complete_list_index = self.app_theme_list.len().saturating_sub(1);
        let new_cursor = min(*state.cursor.to_ref() + 1, last_complete_list_index as u8);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor > last_index {
            last_index = new_cursor;
            first_index = new_cursor - (*state.visible_items.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_item_list(
            first_index.into(),
            last_index.into(),
            new_cursor.into(),
            state,
        );
    }

    fn move_cursor_up(&self, state: &mut AppThemeSelectorState) {
        let new_cursor = max(state.cursor.to_ref().saturating_sub(1), 0);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor < first_index {
            first_index = new_cursor;
            last_index = new_cursor + (*state.visible_items.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_item_list(
            first_index.into(),
            last_index.into(),
            new_cursor.into(),
            state,
        );
    }

    fn update_item_list(
        &self,
        first_index: usize,
        last_index: usize,
        selected_index: usize,
        state: &mut AppThemeSelectorState,
    ) {
        let display_projects = &self.app_theme_list[first_index..=last_index];
        let mut new_app_theme_list: Vec<AppTheme> = vec![];
        display_projects.iter().for_each(|app_theme_persisted| {
            let app_theme: AppTheme = get_app_theme_by_name(&app_theme_persisted.name);

            new_app_theme_list.push(app_theme);
        });

        loop {
            if state.window_list.len() > 0 {
                state.window_list.pop_front();
            } else {
                break;
            }
        }

        new_app_theme_list
            .into_iter()
            .enumerate()
            .for_each(|(index, mut app_theme)| {
                let visible_index = selected_index.saturating_sub(first_index);
                if index == visible_index {
                    app_theme.row_color = SELECTED_PROJECT_ROW_COLOR.to_string().into();
                } else {
                    app_theme.row_color = DEFAULT_PROJECT_ROW_COLOR.to_string().into();
                }

                state.window_list.push(app_theme);
            });
    }
}

impl Component for AppThemeSelector {
    type State = AppThemeSelectorState;
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
                context.publish("app_theme_selector__cancel", |state| &state.cursor)
            }

            anathema::component::KeyCode::Enter => {
                let selected_index = *state.cursor.to_ref() as usize;
                let app_theme_persisted = self.app_theme_list.get(selected_index);

                match app_theme_persisted {
                    Some(app_theme_persisted) => {
                        state
                            .selected_app_theme
                            .set(app_theme_persisted.name.clone());

                        context.publish("app_theme_selector__selection", |state| {
                            &state.selected_app_theme
                        });

                        let app_theme = get_app_theme_by_name(&state.selected_app_theme.to_ref());
                        state.app_theme.set(app_theme);
                    }
                    None => context.publish("app_theme_selector__cancel", |state| &state.cursor),
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

        match self.load(state) {
            Ok(_) => {
                // Reset navigation state
                let current_last_index = min(
                    *state.visible_items.to_ref(),
                    self.app_theme_list.len() as u8,
                )
                .saturating_sub(1);
                state.cursor.set(0);
                state.current_first_index.set(0);
                state.current_last_index.set(current_last_index);

                let first_index: usize = *state.current_first_index.to_ref() as usize;
                let last_index: usize = *state.current_last_index.to_ref() as usize;
                let selected_index = 0;

                self.update_item_list(first_index, last_index, selected_index, state)
            }

            // TODO: Figure out what to do if the list of projects can't be loaded
            Err(_) => todo!(),
        }
    }

    fn message(
        &mut self,
        _: Self::Message,
        _: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        // println!("Received message in project window: {message}");

        // NOTE: The currently selected project might need to be sent from the dashboard
        // when opening the project window after choosing a project
    }
}
