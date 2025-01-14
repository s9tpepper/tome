use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{Component, ComponentId},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{List, State, Value},
    widgets::Elements,
};
use serde::{Deserialize, Serialize};

use crate::{
    components::dashboard::{DashboardMessageHandler, DashboardState, FloatingWindow},
    messages::{
        confirm_delete_endpoint::ConfirmDeleteEndpoint,
        confirm_delete_project::{ConfirmAction, DeleteEndpointDetails},
    },
    projects::{Endpoint, PersistedEndpoint},
    theme::{get_app_theme, AppTheme},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum EndpointsSelectorMessages {
    EndpointsList(Vec<PersistedEndpoint>),
}

pub const ENDPOINTS_SELECTOR_TEMPLATE: &str =
    "./src/components/floating_windows/templates/endpoints_selector.aml";

// TODO: Fix the default project row color to the correct gray
const DEFAULT_ROW_COLOR: &str = "#333333";
const SELECTED_ROW_COLOR: &str = "#FFFFFF";

#[derive(Default, State)]
pub struct EndpointsSelectorState {
    cursor: Value<u8>,
    current_first_index: Value<u8>,
    current_last_index: Value<u8>,
    visible_rows: Value<u8>,
    window_list: Value<List<Endpoint>>,
    count: Value<u8>,
    selected_item: Value<String>,
    app_theme: Value<AppTheme>,
}

impl EndpointsSelectorState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        EndpointsSelectorState {
            cursor: 0.into(),
            count: 0.into(),
            current_first_index: 0.into(),
            current_last_index: 4.into(),
            visible_rows: 5.into(),
            window_list: List::empty(),
            selected_item: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

#[derive(Default)]
pub struct EndpointsSelector {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    items_list: Vec<PersistedEndpoint>,
}

impl EndpointsSelector {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let id = builder.register_component(
            "endpoints_selector_window",
            ENDPOINTS_SELECTOR_TEMPLATE,
            EndpointsSelector::new(ids.clone()),
            EndpointsSelectorState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("endpoints_selector_window"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut EndpointsSelectorState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        EndpointsSelector {
            component_ids,
            items_list: vec![],
        }
    }

    fn move_cursor_down(&self, state: &mut EndpointsSelectorState) {
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

    fn move_cursor_up(&self, state: &mut EndpointsSelectorState) {
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
        state: &mut EndpointsSelectorState,
    ) {
        if self.items_list.is_empty() {
            return;
        }

        let mut range_end = last_index;
        let actual_last_index = self.items_list.len().saturating_sub(1);
        if last_index > actual_last_index {
            range_end = actual_last_index;
        }

        let display_items = &self.items_list[first_index..=range_end];
        let mut new_items_list: Vec<Endpoint> = vec![];
        display_items.iter().for_each(|display_endpoint| {
            new_items_list.push(display_endpoint.into());
        });

        loop {
            if state.window_list.len() > 0 {
                state.window_list.pop_front();
            } else {
                break;
            }
        }

        let mut new_list_state = List::<Endpoint>::empty();
        new_items_list
            .into_iter()
            .enumerate()
            .for_each(|(index, mut endpoint)| {
                let visible_index = selected_index.saturating_sub(first_index);
                if index == visible_index {
                    endpoint.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                    endpoint.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                } else {
                    endpoint.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                    endpoint.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                }

                new_list_state.push(endpoint);
            });

        state.window_list = new_list_state;
    }

    fn delete_endpoint(
        &self,
        state: &mut EndpointsSelectorState,
        mut context: Context<'_, EndpointsSelectorState>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let persisted_endpoint = self.items_list.get(selected_index);

        match persisted_endpoint {
            Some(persisted_endpoint) => match serde_json::to_string(persisted_endpoint) {
                Ok(project_json) => {
                    state.selected_item.set(project_json);
                    context.publish("endpoints_selector__delete", |state| &state.selected_item)
                }

                Err(_) => context.publish("endpoints_selector__cancel", |state| &state.cursor),
            },
            None => context.publish("endpoints_selector__cancel", |state| &state.cursor),
        }
    }

    fn rename_endpoint(
        &self,
        state: &mut EndpointsSelectorState,
        mut context: Context<'_, EndpointsSelectorState>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let project = self.items_list.get(selected_index);

        match project {
            Some(project) => match serde_json::to_string(project) {
                Ok(project_json) => {
                    state.selected_item.set(project_json);
                    context.publish("rename_endpoint", |state| &state.selected_item)
                }

                Err(_) => context.publish("endpoints_selector__cancel", |state| &state.cursor),
            },
            None => context.publish("endpoints_selector__cancel", |state| &state.cursor),
        }
    }
}

impl DashboardMessageHandler for EndpointsSelector {
    fn handle_message(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut DashboardState,
        mut context: anathema::prelude::Context<'_, DashboardState>,
        _: Elements<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();

        match event.as_str() {
            "endpoints_selector__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");
            }

            "endpoints_selector__selection" => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");

                let value = &*value.to_common_str();
                let endpoint = serde_json::from_str::<PersistedEndpoint>(value);

                match endpoint {
                    Ok(endpoint) => {
                        state.endpoint.set((&endpoint).into());
                    }
                    Err(_) => todo!(),
                }
            }

            "endpoints_selector__delete" => {
                state.floating_window.set(FloatingWindow::ConfirmAction);
                context.set_focus("id", "confirm_action_window");

                let value = &*value.to_common_str();
                let endpoint = serde_json::from_str::<PersistedEndpoint>(value);

                match endpoint {
                    Ok(endpoint) => {
                        let confirm_delete_endpoint = DeleteEndpointDetails {
                            title: format!("Delete {}", endpoint.name),
                            message: "Are you sure you want to delete?".into(),
                            endpoint,
                        };

                        let confirm_message =
                            ConfirmAction::ConfirmDeletePersistedEndpoint(confirm_delete_endpoint);

                        if let Ok(message) = serde_json::to_string(&confirm_message) {
                            let confirm_action_window_id =
                                component_ids.get("confirm_action_window");
                            if let Some(id) = confirm_action_window_id {
                                context.emit(*id, message);
                            }
                        }
                    }
                    Err(_) => todo!(),
                }
            }

            _ => {}
        }
    }
}

impl Component for EndpointsSelector {
    type State = EndpointsSelectorState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);
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
                'd' => self.delete_endpoint(state, context),
                'r' => self.rename_endpoint(state, context),
                _ => {}
            },

            anathema::component::KeyCode::Up => self.move_cursor_up(state),
            anathema::component::KeyCode::Down => self.move_cursor_down(state),

            anathema::component::KeyCode::Esc => {
                // NOTE: This sends cursor to satisfy publish() but is not used
                context.publish("endpoints_selector__cancel", |state| &state.cursor)
            }

            anathema::component::KeyCode::Enter => {
                let selected_index = *state.cursor.to_ref() as usize;
                let endpoint = self.items_list.get(selected_index);

                match endpoint {
                    Some(endpoint) => match serde_json::to_string(endpoint) {
                        Ok(endpoint_json) => {
                            state.selected_item.set(endpoint_json);
                            context.publish("endpoints_selector__selection", |state| {
                                &state.selected_item
                            });
                        }
                        Err(_) => {
                            context.publish("endpoints_selector__cancel", |state| &state.cursor)
                        }
                    },
                    None => context.publish("endpoints_selector__cancel", |state| &state.cursor),
                }
            }

            _ => {}
        }
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        let endpoints_selector_message =
            serde_json::from_str::<EndpointsSelectorMessages>(&message);

        match endpoints_selector_message {
            Ok(deserialized_message) => match deserialized_message {
                EndpointsSelectorMessages::EndpointsList(endpoints) => {
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
