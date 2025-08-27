use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Children, Component, ComponentId, Context, MouseEvent},
    runtime::Builder,
    state::{Maybe, State, Value},
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        dashboard::{DashboardMessageHandler, DashboardMessages},
        send_message,
    },
    options::get_button_caps,
    projects::{rename_project, PersistedProject},
    templates::template,
    theme::{get_app_theme, get_app_theme_persisted, AppTheme},
};

use super::FloatingWindow;

#[derive(Debug, Serialize, Deserialize)]
pub enum EditProjectNameMessages {
    ClearInput,
    InputValue(String),
    Specifically(PersistedProject),
}

pub struct EditProjectName {
    persisted_project: Option<PersistedProject>,

    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl DashboardMessageHandler for EditProjectName {
    fn handle_message(
        event: &mut anathema::component::UserEvent<'_>,
        _ident: impl Into<String>,
        state: &mut crate::components::dashboard::DashboardState,
        mut context: anathema::component::Context<
            '_,
            '_,
            crate::components::dashboard::DashboardState,
        >,
        _children: anathema::component::Children<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event_name: String = event.name().to_string();
        match event_name.as_str() {
            "edit_project_name__specific_project_rename" => {
                let specific_name_update = event.data::<SpecificNameChange>();

                if *state.project.to_ref().name.to_ref() == *specific_name_update.old_name.to_ref()
                {
                    state
                        .project
                        .to_mut()
                        .name
                        .set(specific_name_update.new_name.to_ref().clone());
                }

                state.floating_window.set(FloatingWindow::None);

                context.components.by_attribute("id", "app").focus();

                if let Ok(message) = serde_json::to_string(&EditProjectNameMessages::ClearInput) {
                    let _ = send_message(
                        "edit_project_name",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                };
            }

            "edit_project_name__submit" => {
                info!("Handling edit_project_name__submit");

                let value = event.data::<String>().clone();
                let new_name = value.to_string();
                info!("new_name: {new_name}");

                state.project.to_mut().name.set(new_name);

                state.floating_window.set(FloatingWindow::None);

                context.components.by_attribute("id", "app").focus();

                if let Ok(message) = serde_json::to_string(&EditProjectNameMessages::ClearInput) {
                    info!("Clearing dialog input for edit_name");
                    let _ = send_message(
                        "edit_project_name",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                };
            }

            "edit_project_name__cancel" => {
                state.floating_window.set(FloatingWindow::None);

                context.components.by_attribute("id", "app").focus();
            }
            _ => {}
        }
    }
}

impl Component for EditProjectName {
    type State = EditProjectNameState;
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

        state
            .cancel_button_color
            .set(state.success_color_focused.clone());

        state
            .success_button_color
            .set(state.cancel_color_focused.clone());
    }

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        state
            .cancel_button_color
            .set(state.button_color_unfocused.clone());

        state
            .success_button_color
            .set(state.button_color_unfocused.clone());
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
            .by_attribute("id", "submit_button")
            .first(|_, _| {
                if mouse.left_up() {
                    self.submit(state, &mut context_ref);
                }
            });
        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "cancel_button")
            .first(|_, _| {
                if mouse.left_up() {
                    self.cancel(state, &mut context_ref);
                }
            });
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        if let Ok(msg) = serde_json::from_str::<EditProjectNameMessages>(&message) {
            match msg {
                EditProjectNameMessages::ClearInput => {
                    state.name.set("".to_string());

                    if let Ok(ids) = self.component_ids.try_borrow() {
                        let _ = send_message(
                            "edit_project_name_input",
                            "".to_string(),
                            &ids,
                            context.emitter,
                        );
                    }
                }

                EditProjectNameMessages::InputValue(input_value) => {
                    self.set_name_input(&input_value, context);

                    state.name.set(input_value);
                    state.active = true;
                }

                EditProjectNameMessages::Specifically(persisted_project) => {
                    let input_value = &persisted_project.name;
                    state.name.set(input_value.to_string());

                    self.set_name_input(input_value, context);

                    self.persisted_project = Some(persisted_project);
                    state.active = true;
                }
            }
        }
    }

    fn on_event(
        &mut self,
        event: &mut anathema::component::UserEvent<'_>,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match event.name() {
            "name_input_escape" => context
                .components
                .by_attribute("id", "edit_project_name")
                .focus(),
            "name_input_update" => state.name.set(event.data::<String>().clone()),
            "name_input_enter" => self.submit(state, &mut context.into()),

            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'p' | 'P' => context
                    .components
                    .by_attribute("id", "project_name_input")
                    .focus(),
                's' | 'S' => self.submit(state, &mut context.into()),
                'c' | 'C' => self.cancel(state, &mut context.into()),

                _ => {}
            },
            anathema::component::KeyCode::Esc => {
                context.publish("edit_project_name__cancel", state.name.to_ref().clone())
            }

            _ => {}
        }
    }
}

impl EditProjectName {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
    ) -> anyhow::Result<()> {
        let app_theme = get_app_theme();

        let submit_bg = app_theme.overlay_submit_background.to_ref().to_string();
        let cancel_bg = app_theme.overlay_cancel_background.to_ref().to_string();
        let unfocused_bg = app_theme.border_unfocused.to_ref().to_string();
        let (left, right) = get_button_caps();

        let id = builder.component(
            "edit_project_name",
            template("floating_windows/templates/edit_project_name"),
            EditProjectName {
                component_ids: ids.clone(),
                persisted_project: None,
            },
            EditProjectNameState {
                name: String::from("").into(),
                specific_name_change: None.into(),
                success_color_focused: submit_bg,
                cancel_color_focused: cancel_bg,
                button_color_unfocused: unfocused_bg.clone(),
                success_button_color: unfocused_bg.clone().into(),
                cancel_button_color: unfocused_bg.into(),

                button_cap_left: left.to_string().into(),
                button_cap_right: right.to_string().into(),

                app_theme: app_theme.into(),
                active: false,
            },
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("edit_project_name"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut EditProjectNameState) {
        // let app_theme = get_app_theme();
        // state.app_theme.set(app_theme);
        info!("update_app_theme()");
        update_theme(state);
    }

    fn set_name_input(
        &self,
        input_value: &str,
        mut context: Context<'_, '_, EditProjectNameState>,
    ) {
        if let Ok(ids) = self.component_ids.try_borrow() {
            let _ = send_message(
                "edit_project_name_input",
                input_value.to_string(),
                &ids,
                context.emitter,
            );
        }

        context
            .components
            .by_attribute("id", "project_name_input")
            .focus();
    }

    fn rename_specific_project(
        &self,
        project: &PersistedProject,
        state: &mut EditProjectNameState,
        context_ref: &mut RefCell<Context<'_, '_, EditProjectNameState>>,
    ) {
        let mut context = context_ref.borrow_mut();

        state.specific_name_change = Some(SpecificNameChange {
            old_name: project.name.to_string().into(),
            new_name: state.name.to_ref().to_string().into(),
        })
        .into();

        match rename_project(project, &state.name.to_ref()) {
            Ok(_) => {
                let snc_ref = state.specific_name_change.to_ref();
                let Some(snc_value) = snc_ref.get_ref() else {
                    return;
                };
                let specific_name_change = snc_value.to_ref().clone();

                context.publish(
                    "edit_project_name__specific_project_rename",
                    specific_name_change,
                );
            }
            Err(_) => {
                let error_message = "There was an error renaming the project".to_string();
                let dashboard_messages = DashboardMessages::ShowError(error_message);

                let Ok(message) = serde_json::to_string(&dashboard_messages) else {
                    return;
                };

                let Ok(ids) = self.component_ids.try_borrow() else {
                    return;
                };

                let _ = send_message("dashboard", message, &ids, context.emitter);
            }
        }
    }

    fn submit(
        &self,
        state: &mut EditProjectNameState,
        context: &mut RefCell<Context<'_, '_, EditProjectNameState>>,
    ) {
        match &self.persisted_project {
            Some(persisted_project) => {
                info!("Renaming a specific project: {}", persisted_project.name);
                self.rename_specific_project(persisted_project, state, context);
            }

            None => {
                info!("Publishing edit_project_name__submit event");
                context
                    .borrow_mut()
                    .publish("edit_project_name__submit", state.name.to_ref().clone());
            }
        }

        state.active = false;
    }

    fn cancel(
        &self,
        state: &mut EditProjectNameState,
        context: &mut RefCell<Context<'_, '_, EditProjectNameState>>,
    ) {
        context
            .borrow_mut()
            .publish("edit_project_name__cancel", state.name.to_ref().clone());

        state.active = false;
    }
}

#[derive(Deserialize, Serialize)]
pub struct SpecificNameUpdate {
    pub old_name: String,
    pub new_name: String,
}

#[derive(State)]
pub struct EditProjectNameState {
    app_theme: Value<AppTheme>,
    name: Value<String>,
    success_button_color: Value<String>,
    cancel_button_color: Value<String>,
    button_cap_left: Value<String>,
    button_cap_right: Value<String>,

    specific_name_change: Value<Maybe<SpecificNameChange>>,

    #[state_ignore]
    success_color_focused: String,

    #[state_ignore]
    cancel_color_focused: String,

    #[state_ignore]
    button_color_unfocused: String,

    // TODO: Remove this active flag when Anathema gets updated to next version, the
    // component will no longer be a part of the tree when it gets conditionally rendered
    #[state_ignore]
    active: bool,
}

#[derive(Default, Debug, State)]
pub struct SpecificNameChange {
    pub old_name: Value<String>,
    pub new_name: Value<String>,
}

impl Clone for SpecificNameChange {
    fn clone(&self) -> Self {
        Self {
            old_name: self.old_name.to_ref().clone().into(),
            new_name: self.new_name.to_ref().clone().into(),
        }
    }
}

fn update_theme(state: &mut EditProjectNameState) {
    let app_theme = get_app_theme_persisted();

    // TODO: Figure out why this breaks the styling and messaging of the dashboard components
    // println!("{app_theme:?}");
    // state.app_theme.set(app_theme.into());
    // state.app_theme.set(app_theme.into());

    let mut at = state.app_theme.to_mut();
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

    // *state.app_theme.to_mut() = app_theme.into();
}
