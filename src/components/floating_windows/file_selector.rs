use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    env,
    fs::{self, DirEntry, File},
    io::BufReader,
    os::macos::raw::stat,
    path::PathBuf,
    rc::Rc,
};

use anathema::{
    component::{Component, ComponentId},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{List, State, Value},
    widgets::Elements,
};
use log::info;

use crate::{
    compatibility::postman::PostmanJson,
    components::{
        dashboard::{DashboardMessageHandler, DashboardMessages, DashboardState, FloatingWindow},
        send_message,
    },
    projects::save_project,
    theme::{get_app_theme, AppTheme},
};

pub const FILE_SELECTOR_TEMPLATE: &str =
    "./src/components/floating_windows/templates/file_selector.aml";

#[derive(Debug, Default, State)]
pub struct FileSelectorState {
    current_directory: Value<String>,
    cursor: Value<u8>,
    current_first_index: Value<u8>,
    current_last_index: Value<u8>,
    visible_rows: Value<u8>,
    window_list: Value<List<Entry>>,
    count: Value<u8>,
    selected_item: Value<String>,
    app_theme: Value<AppTheme>,
}

#[derive(Debug, State)]
struct Entry {
    name: Value<String>,
    row_color: Value<String>,
    row_fg_color: Value<String>,

    #[state_ignore]
    path_buf: PathBuf,
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self {
            name: Value::new(self.name.to_ref().to_string()),
            row_color: Value::new(self.row_color.to_ref().to_string()),
            row_fg_color: Value::new(self.row_fg_color.to_ref().to_string()),
            path_buf: self.path_buf.clone(),
        }
    }
}

impl From<DirEntry> for Entry {
    fn from(dir_entry: DirEntry) -> Self {
        let path = dir_entry.path();
        let name = path.to_str().unwrap_or_default();

        let app_theme = get_app_theme();

        Entry {
            name: name.to_string().into(),
            row_color: app_theme.overlay_background,
            row_fg_color: app_theme.overlay_foreground,
            path_buf: dir_entry.path(),
        }
    }
}

impl FileSelectorState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        FileSelectorState {
            current_directory: "".to_string().into(),
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
pub struct FileSelector {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
    items_list: Vec<Entry>,
}

impl FileSelector {
    pub fn register(
        ident: &str,
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let id = builder.register_component(
            ident,
            FILE_SELECTOR_TEMPLATE,
            FileSelector::new(ids.clone()),
            FileSelectorState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from(ident), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut FileSelectorState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    fn read_directory(
        &mut self,
        path_buf: PathBuf,
        state: &mut FileSelectorState,
        context: Context<'_, FileSelectorState>,
    ) {
        let mut parent_path_buf = path_buf.clone();
        parent_path_buf.pop();

        let current_directory: String = path_buf.to_string_lossy().to_string();
        state.current_directory.set(current_directory);

        match fs::read_dir(path_buf) {
            Ok(read_dir) => {
                self.items_list = read_dir
                    .flatten()
                    .map(|dir_entry| dir_entry.into())
                    .collect();

                self.items_list.insert(
                    0,
                    Entry {
                        name: "..".to_string().into(),
                        row_fg_color: state
                            .app_theme
                            .to_ref()
                            .overlay_foreground
                            .to_ref()
                            .clone()
                            .into(),
                        row_color: state
                            .app_theme
                            .to_ref()
                            .overlay_background
                            .to_ref()
                            .clone()
                            .into(),
                        path_buf: parent_path_buf,
                    },
                );

                let last_index = (*state.visible_rows.to_ref()).saturating_sub(1) as usize;
                state.current_last_index.set(last_index as u8);
                state.current_first_index.set(0);
                state.cursor.set(0);

                self.update_list(0, last_index, 0, state);
            }

            Err(error) => {
                let error_message = error.to_string();

                self.send_error_message(error_message, context);
            }
        }
    }

    pub fn new(component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>) -> Self {
        FileSelector {
            component_ids,
            items_list: vec![],
        }
    }

    fn move_cursor_down(&self, state: &mut FileSelectorState) {
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

    fn move_cursor_up(&self, state: &mut FileSelectorState) {
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
        state: &mut FileSelectorState,
    ) {
        if self.items_list.is_empty() {
            return;
        }

        let mut range_end = last_index;
        let actual_last_index = self.items_list.len().saturating_sub(1);
        if last_index > actual_last_index {
            range_end = actual_last_index;
        }

        info!("Creating new_items_list, first_index: {first_index}, range_end: {range_end}");

        let display_items = &self.items_list[first_index..=range_end];
        let mut new_items_list: Vec<Entry> = vec![];
        display_items.iter().for_each(|display_entry| {
            new_items_list.push((*display_entry).clone());
        });

        loop {
            if state.window_list.len() > 0 {
                state.window_list.pop_front();
            } else {
                break;
            }
        }

        let mut new_list_state = List::<Entry>::empty();
        new_items_list
            .into_iter()
            .enumerate()
            .for_each(|(index, mut entry)| {
                let visible_index = selected_index.saturating_sub(first_index);
                if index == visible_index {
                    entry.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                    entry.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                } else {
                    entry.row_fg_color = state
                        .app_theme
                        .to_ref()
                        .overlay_foreground
                        .to_ref()
                        .clone()
                        .into();
                    entry.row_color = state
                        .app_theme
                        .to_ref()
                        .overlay_background
                        .to_ref()
                        .clone()
                        .into();
                }

                new_list_state.push(entry);
            });

        state.window_list = new_list_state;
    }

    fn handle_file(&self, entry: &Entry, context: Context<'_, FileSelectorState>) {
        let error_message = "Invalid Postman file type to import, choose a .json file".to_string();

        match entry.path_buf.extension() {
            Some(extension) => match extension.to_str().unwrap_or_default() {
                "json" => self.import_postman_file(entry, context),

                _ => self.send_error_message(error_message, context),
            },
            None => self.send_error_message(error_message, context),
        }
    }

    fn import_postman_file(&self, entry: &Entry, context: Context<'_, FileSelectorState>) {
        let error_message = format!(
            "Could not read the file at {}",
            entry.path_buf.to_string_lossy()
        );

        let Ok(file) = File::open(&entry.path_buf) else {
            self.send_error_message(error_message, context);
            return;
        };

        let reader = BufReader::new(file);

        match serde_json::from_reader::<BufReader<File>, PostmanJson>(reader) {
            Ok(postman_json) => match save_project(postman_json.into()) {
                Ok(_) => {
                    let title = "Postman Import".to_string();
                    let message = "Postman file was imported successfully".to_string();

                    self.send_success_message(title, message, context);
                }
                Err(_) => {
                    let error_message = format!(
                        "Could not save the imported project file from {}",
                        entry.path_buf.to_string_lossy()
                    );

                    self.send_error_message(error_message, context);
                }
            },
            Err(deserialize_error) => {
                let err = deserialize_error.to_string();
                let error_message = format!(
                    "Could not deserialize the JSON file at {}\n{}",
                    err,
                    entry.path_buf.to_string_lossy()
                );

                self.send_error_message(error_message, context);
            }
        }
    }

    fn send_success_message(
        &self,
        title: String,
        message: String,
        mut context: Context<'_, FileSelectorState>,
    ) {
        let dashboard_message = DashboardMessages::ShowSucces((title, message));

        let _ = serde_json::to_string(&dashboard_message).map(|json| {
            let Ok(component_ids) = self.component_ids.try_borrow() else {
                return;
            };

            let _ = send_message("dashboard", json, &component_ids, context.emitter);

            context.set_focus("id", "app");
        });
    }

    fn send_error_message(&self, message: String, mut context: Context<'_, FileSelectorState>) {
        let dashboard_message = DashboardMessages::ShowError(message);

        let _ = serde_json::to_string(&dashboard_message).map(|json| {
            let Ok(component_ids) = self.component_ids.try_borrow() else {
                return;
            };

            let _ = send_message("dashboard", json, &component_ids, context.emitter);

            context.set_focus("id", "app");
        });
    }

    fn handle_directory(
        &mut self,
        entry: &Entry,
        state: &mut FileSelectorState,
        context: Context<'_, FileSelectorState>,
    ) {
        self.read_directory(entry.path_buf.clone(), state, context);
    }
}

impl DashboardMessageHandler for FileSelector {
    fn handle_message(
        _: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut DashboardState,
        mut context: anathema::prelude::Context<'_, DashboardState>,
        _: Elements<'_, '_>,
        _: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();

        #[allow(clippy::single_match)]
        match event.as_str() {
            "file_selector__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");
            }

            _ => {}
        }
    }
}

impl Component for FileSelector {
    type State = FileSelectorState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);

        let Ok(path_buf) = env::current_dir() else {
            return;
        };

        self.read_directory(path_buf, state, context);
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
                context.publish("file_selector__cancel", |state| &state.cursor)
            }

            anathema::component::KeyCode::Enter => {
                let selected_index = *state.cursor.to_ref() as usize;
                let items_list = self.items_list.clone();
                let entry = items_list.get(selected_index);

                match entry {
                    Some(entry) => {
                        if entry.path_buf.is_file() {
                            self.handle_file(entry, context);
                        } else if entry.path_buf.is_dir() {
                            self.handle_directory(entry, state, context);
                        }
                    }
                    None => panic!("How did I get an Option<Entry> that is None?"),
                }
            }

            _ => {}
        }
    }
}
