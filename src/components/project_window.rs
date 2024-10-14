use std::cmp::{max, min};

use anathema::{
    component::Component,
    state::{List, State, Value},
};
use serde::{Deserialize, Serialize};

use super::request_headers_editor::HeaderState;

pub const PROJECT_WINDOW_TEMPLATE: &str = "./src/components/templates/project_window.aml";

// TODO: Fix the default project row color to the correct gray
const DEFAULT_PROJECT_ROW_COLOR: &str = "#333333";
const SELECTED_PROJECT_ROW_COLOR: &str = "#FFFFFF";

#[derive(Default, State)]
pub struct ProjectWindowState {
    cursor: Value<u8>,
    current_first_index: Value<u8>,
    current_last_index: Value<u8>,
    visible_projects: Value<u8>,
    window_list: Value<List<ProjectState>>,
}

impl ProjectWindowState {
    pub fn new() -> Self {
        ProjectWindowState {
            cursor: 0.into(),
            current_first_index: 0.into(),
            current_last_index: 4.into(),
            visible_projects: 5.into(),
            window_list: List::empty(),
        }
    }
}

#[derive(Default)]
pub struct ProjectWindow {
    project_list: Vec<Project>,
}

impl ProjectWindow {
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
        let display_projects = &self.project_list[first_index..=last_index];

        let new_project_list = display_projects.iter().map(|project| {
            let requests = project.requests.iter().map(|request| {
                let headers = request.headers.iter().map(|header| HeaderState {
                    name: header.name.clone().into(),
                    value: header.value.clone().into(),
                });

                RequestState {
                    name: request.name.clone().into(),
                    method: request.method.clone().into(),
                    url: request.url.clone().into(),
                    headers: List::from_iter(headers),
                }
            });

            ProjectState {
                row_color: DEFAULT_PROJECT_ROW_COLOR.to_string().into(),
                name: project.name.clone().into(),
                requests: List::from_iter(requests),
            }
        });

        loop {
            if state.window_list.len() > 0 {
                state.window_list.pop_front();
            } else {
                break;
            }
        }

        new_project_list
            .enumerate()
            .for_each(|(index, mut project)| {
                let visible_index = selected_index - first_index;
                if index == visible_index {
                    project.row_color = SELECTED_PROJECT_ROW_COLOR.to_string().into();
                } else {
                    project.row_color = DEFAULT_PROJECT_ROW_COLOR.to_string().into();
                }

                state.window_list.push(project)
            });
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
            anathema::component::KeyCode::Char(char) => match char {
                'j' => self.move_cursor_down(state),
                'k' => self.move_cursor_up(state),
                _ => {}
            },

            anathema::component::KeyCode::Up => self.move_cursor_up(state),
            anathema::component::KeyCode::Down => self.move_cursor_down(state),

            anathema::component::KeyCode::Esc => {
                // NOTE: This sends cursor to satisfy publish() but is not used
                context.publish("cancel_project_window", |state| &state.cursor)
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
        // NOTE: Should this stay on focus? Focus is triggered whenever the window is opened
        // This data should come from disk, eventually from GitHub?
        self.load();

        let first_index: usize = *state.current_first_index.to_ref() as usize;
        let last_index: usize = *state.current_last_index.to_ref() as usize;
        let selected_index = 0;

        self.update_project_list(first_index, last_index, selected_index, state)
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

impl ProjectWindow {
    pub fn new() -> Self {
        ProjectWindow {
            project_list: vec![],
        }
    }

    fn load(&mut self) {
        // println!("self.load()");

        // TODO: Replace this hard coded list of test data with data read from disk
        self.project_list = vec![
            Project {
                name: "Twitch API".into(),
                requests: vec![],
            },
            Project {
                name: "Twitter API".into(),
                requests: vec![],
            },
            Project {
                name: "Facebook API".into(),
                requests: vec![],
            },
            Project {
                name: "OpenAI".into(),
                requests: vec![],
            },
            Project {
                name: "Claude".into(),
                requests: vec![],
            },
            Project {
                name: "Spotify API".into(),
                requests: vec![],
            },
            Project {
                name: "TikTok API".into(),
                requests: vec![],
            },
        ];
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Project {
    name: String,
    requests: Vec<Request>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Request {
    name: String,
    url: String,
    method: String,
    headers: Vec<Header>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Header {
    name: String,
    value: String,
}

#[derive(Default, State)]
struct ProjectState {
    row_color: Value<String>,
    name: Value<String>,
    requests: Value<List<RequestState>>,
}

#[derive(Default, State)]
struct RequestState {
    name: Value<String>,
    url: Value<String>,
    method: Value<String>,
    headers: Value<List<HeaderState>>,
}
