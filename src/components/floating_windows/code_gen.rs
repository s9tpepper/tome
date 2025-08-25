use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{self, Children, Component, ComponentId, Context, Emitter},
    runtime::Builder,
    state::{State, Value},
};

use crate::{
    code_gen::{generate_rust, generate_web, WebType},
    components::{
        dashboard::{DashboardMessageHandler, DashboardMessages, DashboardState},
        send_message,
    },
    templates::template,
    theme::{get_app_theme, AppTheme},
};

use super::FloatingWindow;

#[derive(Default)]
pub struct CodeGen {
    languages: HashMap<char, String>,
}

impl CodeGen {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
    ) -> anyhow::Result<()> {
        let mut languages = HashMap::<char, String>::new();
        languages.insert('r', String::from("rust"));
        languages.insert('t', String::from("typescript"));
        languages.insert('j', String::from("javascript"));

        let app_id = builder.component(
            "codegen_window",
            template("floating_windows/templates/code_gen"),
            CodeGen { languages },
            CodeGenState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert("codegen_window".to_string(), app_id);

        Ok(())
    }

    #[allow(unused)]
    fn update_app_theme(&self, state: &mut CodeGenState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }
}

fn show_successful_code_gen_msg(
    language_name: &str,
    component_ids: Ref<'_, HashMap<String, ComponentId<String>>>,
    emitter: Emitter,
) {
    let title = format!("{language_name} Code Gen");
    let msg = String::from("Code generated successfully");
    let message = DashboardMessages::ShowSucces((title, msg));

    let _ = serde_json::to_string(&message).map(|message| {
        let _ = send_message("dashboard", message, &component_ids, &emitter);
    });
}

fn show_error_code_gen_msg(
    language_name: &str,
    component_ids: Ref<'_, HashMap<String, ComponentId<String>>>,
    emitter: Emitter,
) {
    let msg = format!("Error generating {language_name} code");
    let message = DashboardMessages::ShowError(msg);

    let _ = serde_json::to_string(&message).map(|message| {
        let _ = send_message("dashboard", message, &component_ids, &emitter);
    });
}

#[derive(Default, State)]
pub struct CodeGenState {
    app_theme: Value<AppTheme>,
    language: Value<String>,
}

impl CodeGenState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        CodeGenState {
            app_theme: app_theme.into(),
            language: "".to_string().into(),
        }
    }
}

impl Component for CodeGen {
    type State = CodeGenState;
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
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => {
                let default_language = "rust".to_string();
                let language = self.languages.get(&char).unwrap_or(&default_language);
                state.language.set(language.clone());

                context.publish("codegen__selection", |state: Self::State| state.language);
            }

            anathema::component::KeyCode::Esc => {
                context.publish("codegen__cancel", |state: Self::State| state.language);
            }

            _ => {}
        }
    }
}

impl DashboardMessageHandler for CodeGen {
    fn handle_message(
        event: &mut component::UserEvent<'_>,
        _ident: impl Into<String>,
        state: &mut DashboardState,
        mut context: Context<'_, '_, DashboardState>,
        _children: Children<'_, '_>,
        component_ids: Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let value: String = event.data::<String>().clone();
        let event_name: String = event.name().to_string();
        match event_name.as_str() {
            #[allow(clippy::single_match)]
            "codegen__selection" => match value.as_str() {
                "rust" => {
                    state.floating_window.set(FloatingWindow::None);
                    context.components.by_attribute("id", "app").focus();

                    let project = state.project.to_ref();
                    match generate_rust((&*project).into()) {
                        Ok(_) => {
                            show_successful_code_gen_msg(
                                "Rust",
                                component_ids,
                                context.emitter.clone(),
                            );
                        }

                        Err(_) => {
                            show_error_code_gen_msg("Rust", component_ids, context.emitter.clone());
                        }
                    }
                }

                code_type @ ("typescript" | "javascript") => {
                    let web_type = match code_type {
                        "typescript" => WebType::TypeScript,
                        "javascript" => WebType::JavaScript,
                        _ => WebType::TypeScript,
                    };

                    let language_name = match code_type {
                        "typescript" => "TypeScript",
                        "javascript" => "JavaScript",
                        _ => "",
                    };

                    state.floating_window.set(FloatingWindow::None);
                    context.components.by_attribute("id", "app").focus();

                    let project = state.project.to_ref();

                    match generate_web((&*project).into(), web_type) {
                        Ok(_) => {
                            show_successful_code_gen_msg(
                                language_name,
                                component_ids,
                                context.emitter.clone(),
                            );
                        }

                        Err(_) => {
                            show_error_code_gen_msg(
                                language_name,
                                component_ids,
                                context.emitter.clone(),
                            );
                        }
                    }
                }

                _ => {}
            },

            "codegen__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                context.components.by_attribute("id", "app").focus();
            }

            _ => {}
        }
    }
}
