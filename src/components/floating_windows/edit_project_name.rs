use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, State, Value},
    widgets::Elements,
};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        dashboard::{DashboardMessageHandler, DashboardMessages},
        send_message,
    },
    projects::{rename_project, PersistedProject},
    theme::{get_app_theme, AppTheme},
};

const TEMPLATE: &str = "./src/components/floating_windows/templates/edit_project_name.aml";

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
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut crate::components::dashboard::DashboardState,
        mut context: anathema::prelude::Context<'_, crate::components::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        component_ids: std::cell::Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();
        match event.as_str() {
            "edit_project_name__specific_project_rename" => {
                let Ok(specific_name_update) =
                    serde_json::from_str::<SpecificNameUpdate>(&value.to_string())
                else {
                    let error_message =
                        "There was an error while processing the name update".to_string();
                    let dashboard_messages = DashboardMessages::ShowError(error_message);

                    let Ok(message) = serde_json::to_string(&dashboard_messages) else {
                        return;
                    };

                    let _ = send_message("dashboard", message, &component_ids, context.emitter);
                    return;
                };

                if *state.project.to_ref().name.to_ref() == specific_name_update.old_name {
                    state
                        .project
                        .to_mut()
                        .name
                        .set(specific_name_update.new_name);
                }

                state
                    .floating_window
                    .set(crate::components::dashboard::FloatingWindow::None);

                context.set_focus("id", "app");

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
                let new_name = value.to_string();
                state.project.to_mut().name.set(new_name);

                state
                    .floating_window
                    .set(crate::components::dashboard::FloatingWindow::None);

                context.set_focus("id", "app");

                if let Ok(message) = serde_json::to_string(&EditProjectNameMessages::ClearInput) {
                    let _ = send_message(
                        "edit_project_name",
                        message,
                        &component_ids,
                        context.emitter,
                    );
                };
            }

            "edit_project_name__cancel" => {
                state
                    .floating_window
                    .set(crate::components::dashboard::FloatingWindow::None);

                context.set_focus("id", "app");
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
        _: Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
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
                }

                EditProjectNameMessages::Specifically(persisted_project) => {
                    let input_value = &persisted_project.name;
                    state.name.set(input_value.to_string());

                    self.set_name_input(input_value, context);

                    self.persisted_project = Some(persisted_project);
                }
            }
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match ident {
            "name_input_escape" => context.set_focus("id", "edit_project_name"),
            "name_input_update" => state.name.set(value.to_string()),
            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'p' => context.set_focus("id", "project_name_input"),
                's' => match &self.persisted_project {
                    Some(persisted_project) => {
                        self.rename_specific_project(persisted_project, state, context);
                    }
                    None => {
                        context.publish("edit_project_name__submit", |state| &state.name);
                    }
                },
                'c' => context.publish("edit_project_name__cancel", |state| &state.name),

                _ => {}
            },
            anathema::component::KeyCode::Esc => {
                context.publish("edit_project_name__cancel", |state| &state.name)
            }

            _ => {}
        }
    }
}

impl EditProjectName {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let app_theme = get_app_theme();

        let id = builder.register_component(
            "edit_project_name",
            TEMPLATE,
            EditProjectName {
                component_ids: ids.clone(),
                persisted_project: None,
            },
            EditProjectNameState {
                app_theme: app_theme.into(),
                name: String::from("").into(),
                specific_name_change: None.into(),
            },
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("edit_project_name"), id);

        Ok(())
    }

    fn update_app_theme(&self, state: &mut EditProjectNameState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    fn set_name_input(&self, input_value: &str, mut context: Context<'_, EditProjectNameState>) {
        if let Ok(ids) = self.component_ids.try_borrow() {
            let _ = send_message(
                "edit_project_name_input",
                input_value.to_string(),
                &ids,
                context.emitter,
            );
        }

        context.set_focus("id", "project_name_input");
    }

    fn rename_specific_project(
        &self,
        project: &PersistedProject,
        state: &mut EditProjectNameState,
        mut context: Context<'_, EditProjectNameState>,
    ) {
        let specific_name_update = SpecificNameUpdate {
            old_name: project.name.to_string(),
            new_name: state.name.to_ref().to_string(),
        };

        let Ok(common) = serde_json::to_string(&specific_name_update) else {
            let error_message = "There was an error with the name update".to_string();
            let dashboard_messages = DashboardMessages::ShowError(error_message);

            let Ok(message) = serde_json::to_string(&dashboard_messages) else {
                return;
            };

            let Ok(ids) = self.component_ids.try_borrow() else {
                return;
            };

            let _ = send_message("dashboard", message, &ids, context.emitter);
            return;
        };

        state.specific_name_change = Some(SpecificNameChange {
            old_name: project.name.to_string().into(),
            new_name: state.name.to_ref().to_string().into(),
            common,
        })
        .into();

        match rename_project(project, &state.name.to_ref()) {
            Ok(_) => {
                context.publish("edit_project_name__specific_project_rename", |state| {
                    &state.specific_name_change
                });
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

    specific_name_change: Value<Option<SpecificNameChange>>,
}

pub struct SpecificNameChange {
    pub old_name: Value<String>,
    pub new_name: Value<String>,
    pub common: String,
}

impl ::anathema::state::State for SpecificNameChange {
    fn state_get(
        &self,
        path: ::anathema::state::Path<'_>,
        sub: ::anathema::state::Subscriber,
    ) -> ::core::prelude::v1::Option<::anathema::state::ValueRef> {
        let ::anathema::state::Path::Key(key) = path else {
            return ::core::prelude::v1::None;
        };
        match key {
            "old_name" => ::core::prelude::v1::Some(self.old_name.value_ref(sub)),
            "new_name" => ::core::prelude::v1::Some(self.new_name.value_ref(sub)),
            _ => ::core::prelude::v1::None,
        }
    }

    fn state_lookup(
        &self,
        path: ::anathema::state::Path<'_>,
    ) -> ::core::prelude::v1::Option<::anathema::state::PendingValue> {
        let ::anathema::state::Path::Key(key) = path else {
            return ::core::prelude::v1::None;
        };
        match key {
            "old_name" => ::core::prelude::v1::Some(self.old_name.to_pending()),
            "new_name" => ::core::prelude::v1::Some(self.new_name.to_pending()),
            _ => ::core::prelude::v1::None,
        }
    }

    fn to_common(&self) -> ::core::prelude::v1::Option<::anathema::state::CommonVal<'_>> {
        Some(CommonVal::Str(&self.common))
    }
}
