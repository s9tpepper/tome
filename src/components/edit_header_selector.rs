use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{Children, Component, ComponentId, Context, KeyCode, MouseEvent},
    runtime::Builder,
    state::{List, State, Value},
};
use serde::{Deserialize, Serialize};

use crate::{
    components::dashboard::{DashboardMessageHandler, DashboardState},
    messages::confirm_actions::{ConfirmAction, ConfirmDetails},
    projects::{Header, HeaderState},
    templates::template,
    theme::{get_app_theme, AppTheme},
};

use super::{
    add_header_window::AddHeaderWindowMessages, floating_windows::FloatingWindow, send_message,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum EditHeaderSelectorMessages {
    HeadersList(Vec<Header>),
}

#[derive(Default, State)]
pub struct EditHeaderSelectorState {
    #[state_ignore]
    active: bool,

    cursor: Value<u8>,
    current_first_index: Value<u8>,
    current_last_index: Value<u8>,
    visible_rows: Value<u8>,
    window_list: Value<List<HeaderState>>,
    count: Value<u8>,
    selected_item: Value<String>,
    app_theme: Value<AppTheme>,
}

impl EditHeaderSelectorState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        EditHeaderSelectorState {
            active: false,
            cursor: 0.into(),
            count: 0.into(),
            current_first_index: 0.into(),
            current_last_index: 4.into(),
            visible_rows: 5.into(),
            window_list: List::empty().into(),
            selected_item: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

#[derive(Default)]
pub struct EditHeaderSelector {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    items_list: Vec<Header>,
}

impl EditHeaderSelector {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
    ) -> anyhow::Result<()> {
        let id = builder.component(
            "edit_header_selector",
            template("templates/edit_header_selector"),
            EditHeaderSelector::new(ids.clone()),
            EditHeaderSelectorState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("edit_header_selector"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut EditHeaderSelectorState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        EditHeaderSelector {
            component_ids,
            items_list: vec![],
        }
    }

    fn move_cursor_down(&self, state: &mut EditHeaderSelectorState) {
        let last_complete_list_index = self.items_list.len().saturating_sub(1);
        let new_cursor = min(*state.cursor.to_ref() + 1, last_complete_list_index as u8);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor > last_index {
            last_index = new_cursor;
            first_index = new_cursor - (*state.visible_rows.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_list(
            first_index.into(),
            last_index.into(),
            new_cursor.into(),
            state,
        );
    }

    fn move_cursor_up(&self, state: &mut EditHeaderSelectorState) {
        let new_cursor = max(state.cursor.to_ref().saturating_sub(1), 0);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor < first_index {
            first_index = new_cursor;
            last_index = new_cursor + (*state.visible_rows.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_list(
            first_index.into(),
            last_index.into(),
            new_cursor.into(),
            state,
        );
    }

    fn update_list(
        &self,
        first_index: usize,
        last_index: usize,
        selected_index: usize,
        state: &mut EditHeaderSelectorState,
    ) {
        if self.items_list.is_empty() {
            loop {
                if state.window_list.is_empty() {
                    state.window_list.pop_front();
                } else {
                    break;
                }
            }

            return;
        }

        let mut range_end = last_index;
        let actual_last_index = self.items_list.len().saturating_sub(1);
        if last_index > actual_last_index {
            range_end = actual_last_index;
        }

        let display_items = &self.items_list[first_index..=range_end];
        let mut new_items_list: Vec<HeaderState> = vec![];
        display_items.iter().for_each(|display_header| {
            new_items_list.push(display_header.into());
        });

        loop {
            if state.window_list.is_empty() {
                state.window_list.pop_front();
            } else {
                break;
            }
        }

        let new_list_state = List::<HeaderState>::empty();
        state.window_list = new_list_state.into();

        new_items_list
            .into_iter()
            .enumerate()
            .for_each(|(index, mut header)| {
                let visible_index = selected_index.saturating_sub(first_index);
                if index == visible_index {
                    header.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                    header.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                } else {
                    header.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                    header.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                }

                state.window_list.push(header);
            });
    }

    fn delete_header(
        &self,
        state: &mut EditHeaderSelectorState,
        context: &mut RefCell<Context<'_, '_, EditHeaderSelectorState>>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let persisted_header = self.items_list.get(selected_index);

        match persisted_header {
            Some(persisted_header) => match serde_json::to_string(persisted_header) {
                Ok(project_json) => {
                    state.selected_item.set(project_json);
                    context.borrow_mut().publish(
                        "edit_header_selector__delete",
                        |state: EditHeaderSelectorState| state.selected_item,
                    )
                }

                Err(_) => context.borrow_mut().publish(
                    "edit_header_selector__cancel",
                    |state: EditHeaderSelectorState| state.cursor,
                ),
            },
            None => context.borrow_mut().publish(
                "edit_header_selector__cancel",
                |state: EditHeaderSelectorState| state.cursor,
            ),
        }
    }

    fn edit_header(
        &self,
        state: &mut EditHeaderSelectorState,
        context: &mut RefCell<Context<'_, '_, EditHeaderSelectorState>>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let header = self.items_list.get(selected_index);

        match header {
            Some(header) => match serde_json::to_string(header) {
                Ok(header_json) => {
                    state.selected_item.set(header_json);
                    context.borrow_mut().publish(
                        "edit_header_selector__edit",
                        |state: EditHeaderSelectorState| state.selected_item,
                    )
                }

                Err(_) => context.borrow_mut().publish(
                    "edit_header_selector__cancel",
                    |state: EditHeaderSelectorState| state.cursor,
                ),
            },
            None => context.borrow_mut().publish(
                "edit_header_selector__cancel",
                |state: EditHeaderSelectorState| state.cursor,
            ),
        }
    }

    fn add_header(&self, context: &mut RefCell<Context<'_, '_, EditHeaderSelectorState>>) {
        context.borrow_mut().publish(
            "edit_header_selector__add",
            |state: EditHeaderSelectorState| state.cursor,
        );
    }
}

impl DashboardMessageHandler for EditHeaderSelector {
    fn handle_message(
        event: &mut anathema::component::UserEvent<'_>,
        _: impl Into<String>,
        state: &mut DashboardState,
        mut context: Context<'_, '_, DashboardState>,
        _: anathema::component::Children<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event_name: String = event.name().to_string();

        match event_name.as_str() {
            "edit_header_selector__add" => {
                state.floating_window.set(FloatingWindow::AddHeader);
                context
                    .components
                    .by_attribute("id", "add_header_window")
                    .focus();

                let _ = send_message(
                    "add_header_window",
                    "open".to_string(),
                    &component_ids,
                    context.emitter,
                );
            }

            "edit_header_selector__edit" => {
                let header = event.data::<Header>();

                let current_names: Vec<String> = state
                    .endpoint
                    .to_ref()
                    .headers
                    .to_ref()
                    .iter()
                    .map(|e| e.to_ref().name.to_ref().to_string())
                    .collect();

                let current_header_name = header.name.clone();
                let add_header_window_messages = AddHeaderWindowMessages::Specifically((
                    current_header_name,
                    header.clone(),
                    current_names,
                ));

                let Ok(message) = serde_json::to_string(&add_header_window_messages) else {
                    return;
                };

                state.floating_window.set(FloatingWindow::AddHeader);
                context
                    .components
                    .by_attribute("id", "add_header_window")
                    .focus();

                let _ = send_message(
                    "add_header_window",
                    message,
                    &component_ids,
                    context.emitter,
                );
            }

            "edit_header_selector__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                context.components.by_attribute("id", "app").focus();
            }

            "edit_header_selector__delete" => {
                state.floating_window.set(FloatingWindow::ConfirmAction);
                context
                    .components
                    .by_attribute("id", "confirm_action_window")
                    .focus();

                let header = event.data::<Header>();
                let confirm_delete_header = ConfirmDetails {
                    title: format!("Delete {}", header.name),
                    message: "Are you sure you want to delete?".into(),
                    data: header.clone(),
                };

                let confirm_message = ConfirmAction::ConfirmDeleteHeader(confirm_delete_header);

                if let Ok(message) = serde_json::to_string(&confirm_message) {
                    let confirm_action_window_id = component_ids.get("confirm_action_window");
                    if let Some(id) = confirm_action_window_id {
                        context.emit(*id, message);
                    }
                }
            }

            _ => {}
        }
    }
}

impl Component for EditHeaderSelector {
    type State = EditHeaderSelectorState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        self.update_app_theme(state);
        state.active = true;
    }

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        // TODO: Remove this state.active after Anathema update
        if !state.active {
            return;
        }

        let mut context_ref = RefCell::new(context);

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "add_button")
            .first(|_, _| {
                if mouse.left_up() {
                    self.add_header(&mut context_ref);
                }
            });

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "edit_button")
            .first(|_, _| {
                if mouse.left_up() {
                    self.edit_header(state, &mut context_ref);
                }
            });

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "delete_button")
            .first(|_, _| {
                if mouse.left_up() {
                    self.delete_header(state, &mut context_ref);
                }
            });
    }

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        state.active = false;
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match event.code {
            KeyCode::Char(char) => match char {
                'j' => self.move_cursor_down(state),
                'k' => self.move_cursor_up(state),
                'd' => self.delete_header(state, &mut context.into()),
                'e' => self.edit_header(state, &mut context.into()),
                'a' => self.add_header(&mut context.into()),
                _ => {}
            },

            KeyCode::Up => self.move_cursor_up(state),
            KeyCode::Down => self.move_cursor_down(state),

            KeyCode::Esc => {
                // NOTE: This sends cursor to satisfy publish() but is not used
                context.publish("edit_header_selector__cancel", |state: Self::State| {
                    state.cursor
                })
            }

            _ => {}
        }
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        let endpoints_selector_message =
            serde_json::from_str::<EditHeaderSelectorMessages>(&message);

        match endpoints_selector_message {
            Ok(deserialized_message) => match deserialized_message {
                EditHeaderSelectorMessages::HeadersList(endpoints) => {
                    self.items_list = endpoints;

                    let current_last_index =
                        min(*state.visible_rows.to_ref(), self.items_list.len() as u8)
                            .saturating_sub(1);
                    state.cursor.set(0);
                    state.current_first_index.set(0);
                    state.current_last_index.set(current_last_index);

                    let first_index: usize = *state.current_first_index.to_ref() as usize;
                    let last_index: usize = *state.current_last_index.to_ref() as usize;
                    let selected_index = 0;

                    self.update_list(first_index, last_index, selected_index, state);
                }
            },

            // TODO: Figure out what to do with deserialization errors
            Err(_error) => {
                // eprintln!("{error}");
                // dbg!(error);
            }
        }
    }
}
