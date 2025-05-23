use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{
        Component, ComponentId,
        KeyCode::{Char, Down, Enter, Esc, Up},
        MouseEvent,
    },
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{List, State, Value},
    widgets::Elements,
};
use log::info;

use crate::{
    app::GlobalEventHandler,
    messages::confirm_actions::{ConfirmAction, ConfirmDetails},
    projects::{get_projects, PersistedProject, Project},
    templates::template,
    theme::{get_app_theme, AppTheme},
};

use super::{
    dashboard::{DashboardMessageHandler, DashboardMessages},
    floating_windows::FloatingWindow,
    send_message,
    textarea::TextAreaMessages,
};

#[derive(Default, State)]
pub struct ProjectWindowState {
    #[state_ignore]
    active: bool,

    cursor: Value<u8>,
    current_first_index: Value<u8>,
    current_last_index: Value<u8>,
    visible_projects: Value<u8>,
    window_list: Value<List<Project>>,
    project_count: Value<u8>,
    selected_project: Value<String>,
    app_theme: Value<AppTheme>,
}

impl ProjectWindowState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        ProjectWindowState {
            active: false,
            cursor: 0.into(),
            project_count: 0.into(),
            current_first_index: 0.into(),
            current_last_index: 4.into(),
            visible_projects: 5.into(),
            window_list: List::empty(),
            selected_project: "".to_string().into(),
            app_theme: app_theme.into(),
        }
    }
}

#[derive(Default)]
pub struct ProjectWindow {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    project_list: Vec<PersistedProject>,
}

impl ProjectWindow {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
    ) -> anyhow::Result<()> {
        let id = builder.register_component(
            "project_selector",
            template("templates/project_window"),
            ProjectWindow::new(ids.clone()),
            ProjectWindowState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("project_selector"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut ProjectWindowState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        ProjectWindow {
            component_ids,
            project_list: vec![],
        }
    }

    fn load(&mut self, state: &mut ProjectWindowState) -> anyhow::Result<()> {
        self.project_list = get_projects()?;
        state.project_count.set(self.project_list.len() as u8);

        Ok(())
    }

    fn move_cursor_down(&self, state: &mut ProjectWindowState) {
        let last_complete_list_index = self.project_list.len().saturating_sub(1);
        let new_cursor = min(*state.cursor.to_ref() + 1, last_complete_list_index as u8);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor > last_index {
            last_index = new_cursor;
            first_index = new_cursor - (*state.visible_projects.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_project_list(
            first_index.into(),
            last_index.into(),
            new_cursor.into(),
            state,
        );
    }

    fn move_cursor_up(&self, state: &mut ProjectWindowState) {
        let new_cursor = max(state.cursor.to_ref().saturating_sub(1), 0);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor < first_index {
            first_index = new_cursor;
            last_index = new_cursor + (*state.visible_projects.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_project_list(
            first_index.into(),
            last_index.into(),
            new_cursor.into(),
            state,
        );
    }

    fn update_project_list(
        &self,
        first_index: usize,
        last_index: usize,
        selected_index: usize,
        state: &mut ProjectWindowState,
    ) {
        if self.project_list.is_empty() {
            return;
        }

        let display_projects = &self.project_list[first_index..=last_index];
        let mut new_project_list: Vec<Project> = vec![];
        display_projects.iter().for_each(|display_project| {
            new_project_list.push(display_project.into());
        });

        loop {
            if state.window_list.len() > 0 {
                state.window_list.pop_front();
            } else {
                break;
            }
        }

        new_project_list
            .into_iter()
            .enumerate()
            .for_each(|(index, mut project)| {
                let visible_index = selected_index.saturating_sub(first_index);
                if index == visible_index {
                    project.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                    project.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                } else {
                    project.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                    project.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                }

                state.window_list.push(project);
            });
    }

    fn rename_project(
        &self,
        state: &mut ProjectWindowState,
        context: &mut RefCell<Context<'_, ProjectWindowState>>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let project = self.project_list.get(selected_index);

        match project {
            Some(project) => match serde_json::to_string(project) {
                Ok(project_json) => {
                    state.selected_project.set(project_json);
                    context
                        .borrow_mut()
                        .publish("rename_project", |state| &state.selected_project)
                }

                Err(_) => context
                    .borrow_mut()
                    .publish("project_window__cancel", |state| &state.cursor),
            },
            None => context
                .borrow_mut()
                .publish("project_window__cancel", |state| &state.cursor),
        }
    }

    fn add_project(&self, context: &mut RefCell<Context<'_, ProjectWindowState>>) {
        context
            .borrow_mut()
            .publish("add_new_project", |state| &state.cursor);
    }

    fn delete_project(
        &self,
        state: &mut ProjectWindowState,
        context: &mut RefCell<Context<'_, ProjectWindowState>>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let project = self.project_list.get(selected_index);

        match project {
            Some(project) => match serde_json::to_string(project) {
                Ok(project_json) => {
                    state.selected_project.set(project_json);
                    context
                        .borrow_mut()
                        .publish("project_window__delete", |state| &state.selected_project)
                }

                Err(_) => context
                    .borrow_mut()
                    .publish("project_window__cancel", |state| &state.cursor),
            },
            None => context
                .borrow_mut()
                .publish("project_window__cancel", |state| &state.cursor),
        }
    }
}

impl DashboardMessageHandler for ProjectWindow {
    fn handle_message(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::dashboard::DashboardState,
        mut context: anathema::prelude::Context<'_, super::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();

        match event.as_str() {
            "project_window__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");
            }

            "project_window__selection" => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");

                let value = &*value.to_common_str();
                let project = serde_json::from_str::<PersistedProject>(value);

                match project {
                    Ok(project) => {
                        info!("Loaded selected project: {project:?}");

                        state.endpoint_count.set(project.endpoints.len() as u8);

                        let mut state_project = state.project.to_mut();
                        state_project.update_from_persisted(&project);

                        let endpoints = state_project.endpoints.to_ref();

                        let mut state_endpoint = state.endpoint.to_mut();
                        let Some(new_endpoint) = endpoints.get(0) else {
                            state_endpoint.reset();
                            return;
                        };

                        let new_endpoint = new_endpoint.to_ref();
                        state_endpoint.update(&new_endpoint);

                        let _ = send_message(
                            "url_text_input",
                            state_endpoint.url.to_ref().to_string(),
                            &component_ids,
                            context.emitter,
                        );

                        let textarea_msg =
                            TextAreaMessages::SetInput(state_endpoint.body.to_ref().to_string());

                        let Ok(message) = serde_json::to_string(&textarea_msg) else {
                            return;
                        };

                        let _ = send_message(
                            "request_body_input",
                            message,
                            &component_ids,
                            context.emitter,
                        );
                    }
                    Err(_) => todo!(),
                }
            }

            "project_window__delete" => {
                state.floating_window.set(FloatingWindow::ConfirmAction);
                context.set_focus("id", "confirm_action_window");

                let value = &*value.to_common_str();
                let project = serde_json::from_str::<PersistedProject>(value);

                match project {
                    Ok(project) => {
                        let confirm_delete_project = ConfirmDetails {
                            title: format!("Delete {}", project.name),
                            message: "Are you sure you want to delete?".into(),
                            data: project,
                        };

                        let confirm_message =
                            ConfirmAction::ConfirmDeletePersistedProject(confirm_delete_project);

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

impl Component for ProjectWindow {
    type State = ProjectWindowState;
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
            Char(char) => match char {
                'j' => self.move_cursor_down(state),
                'k' => self.move_cursor_up(state),
                'd' => self.delete_project(state, &mut context.into()),
                'r' => self.rename_project(state, &mut context.into()),
                'a' => self.add_project(&mut context.into()),

                _ => {}
            },

            Up => self.move_cursor_up(state),
            Down => self.move_cursor_down(state),

            Esc => {
                // NOTE: This sends cursor to satisfy publish() but is not used
                context.publish("project_window__cancel", |state| &state.cursor)
            }

            Enter => {
                let selected_index = *state.cursor.to_ref() as usize;
                let project = self.project_list.get(selected_index);

                match project {
                    Some(project) => match serde_json::to_string(project) {
                        Ok(project_json) => {
                            state.selected_project.set(project_json);
                            context.publish("project_window__selection", |state| {
                                &state.selected_project
                            });
                        }
                        Err(_) => context.publish("project_window__cancel", |state| &state.cursor),
                    },
                    None => context.publish("project_window__cancel", |state| &state.cursor),
                }
            }

            _ => {}
        }
    }

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        // TODO: Remove this state.active after Anathema update
        if !state.active {
            return;
        }

        let mut context_ref = RefCell::new(context);

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "add_button")
            .first(|_, _| {
                if mouse.lsb_up() {
                    self.add_project(&mut context_ref);
                }
            });

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "rename_button")
            .first(|_, _| {
                if mouse.lsb_up() {
                    self.rename_project(state, &mut context_ref);
                }
            });

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "delete_button")
            .first(|_, _| {
                if mouse.lsb_up() {
                    self.delete_project(state, &mut context_ref);
                }
            });
    }

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
    ) {
        state.active = false;
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);

        match self.load(state) {
            Ok(_) => {
                // Reset navigation state
                let current_last_index = min(
                    *state.visible_projects.to_ref(),
                    self.project_list.len() as u8,
                )
                .saturating_sub(1);
                state.cursor.set(0);
                state.current_first_index.set(0);
                state.current_last_index.set(current_last_index);

                let first_index: usize = *state.current_first_index.to_ref() as usize;
                let last_index: usize = *state.current_last_index.to_ref() as usize;
                let selected_index = 0;

                self.update_project_list(first_index, last_index, selected_index, state);

                state.active = true;
            }

            Err(error) => {
                let error_message = format!("Error loading projects: {}", error);

                let dashboard_msg = DashboardMessages::ShowError(error_message);
                let Ok(msg) = serde_json::to_string(&dashboard_msg) else {
                    return;
                };
                let Ok(ids) = self.component_ids.try_borrow() else {
                    return;
                };
                let _ = send_message("dashboard", msg, &ids, context.emitter);
            }
        }
    }
}
