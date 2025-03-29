use std::cell::RefCell;

use anathema::{
    component::{Component, KeyCode, KeyEvent, MouseEvent},
    prelude::Context,
    state::CommonVal,
    widgets::{components::events::KeyState, Elements},
};

use crate::{
    components::{
        floating_windows::FloatingWindow, send_message, textarea::TextAreaMessages,
        textinput::TextInputMessages,
    },
    options::get_button_caps,
};

use super::{
    associated_functions, keyboard_events, update_theme, DashboardComponent, DashboardDisplay,
    DashboardMessages, DashboardState, KeebState,
};

impl Component for DashboardComponent {
    type State = DashboardState;
    type Message = String;

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        if let Ok(dashboard_message) = serde_json::from_str::<DashboardMessages>(&message) {
            match dashboard_message {
                DashboardMessages::KeyboardEvent(keeb_event) => {
                    let key_event = KeyEvent {
                        code: KeyCode::Char(keeb_event.character),
                        ctrl: keeb_event.ctrl,
                        state: match keeb_event.state {
                            KeebState::Press => KeyState::Press,
                            KeebState::Repeat => KeyState::Repeat,
                            KeebState::Release => KeyState::Release,
                        },
                    };

                    self.on_key(key_event, state, elements, context);
                }

                DashboardMessages::BackToRequest => {
                    state.main_display.set(DashboardDisplay::RequestBody);
                    context.set_focus("id", "app");
                }

                DashboardMessages::Confirmations(confirm_action) => {
                    self.confirm_action(confirm_action, state, context);
                }

                DashboardMessages::ShowSucces((title, message)) => {
                    self.show_message(&title, &message, state);
                }

                DashboardMessages::ShowError(message) => {
                    self.show_error(&message, state);
                }

                DashboardMessages::ThemeUpdate => {
                    // TODO: Use this message again when the state update bug is fixed in anathema
                    // println!("Changing dashboard theme");
                    // self.update_app_theme(state);

                    // let app_theme = get_app_theme_persisted();
                    // state.app_theme.set(app_theme.into());
                }

                DashboardMessages::ButtonStyleUpdate => {
                    let button_caps = get_button_caps();

                    state.button_cap_left.set(button_caps.0.to_string());
                    state.button_cap_right.set(button_caps.1.to_string());
                }

                DashboardMessages::TextInput(text_input_message) => match text_input_message {
                    // TODO: Refactor this message to not be 100% coupled to only editing the
                    // endpoint name
                    TextInputMessages::Change(value) => {
                        let mut endpoint = state.endpoint.to_mut();
                        let name_still_default = *endpoint.url.to_ref() == *endpoint.name.to_ref();

                        endpoint.url.set(value.to_string());

                        if name_still_default {
                            endpoint.name.set(value.to_string());
                        }
                    }

                    #[allow(clippy::single_match)]
                    TextInputMessages::Update(text_update) => match text_update.id.as_str() {
                        "endpoint_url_input" => {
                            state.endpoint.to_mut().url.set(text_update.value);
                        }

                        _ => {}
                    },

                    #[allow(clippy::single_match)]
                    TextInputMessages::Escape(text_update) => match text_update.id.as_str() {
                        "endpoint_url_input" => {
                            context.set_focus("id", "app");

                            if let Ok(ids) = self.component_ids.try_borrow() {
                                let _ = send_message(
                                    "url_input",
                                    "unfocus".to_string(),
                                    &ids,
                                    context.emitter,
                                );
                            }
                        }

                        _ => {}
                    },
                },

                // TODO: Refactor this message to not be 100% coupled to only editing the
                // endpoint body
                DashboardMessages::TextArea(text_area_message) => match text_area_message {
                    TextAreaMessages::InputChange(value) => {
                        state.endpoint.to_mut().body.set(value);
                    }

                    // NOTE: SetInput is only used for sending the TextArea a new value
                    TextAreaMessages::SetInput(_) => todo!(),
                },
            }
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: CommonVal<'_>,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        associated_functions(self, ident, value, context, state, elements);
    }

    fn on_key(
        &mut self,
        event: KeyEvent,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        keyboard_events(self, event, state, elements, context);
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        update_theme(state);

        match *state.main_display.to_ref() {
            DashboardDisplay::RequestBody => context.set_focus("id", "request"),

            DashboardDisplay::RequestHeadersEditor => {
                context.set_focus("id", "request_headers_editor")
            }
            DashboardDisplay::ResponseBody => context.set_focus("id", "response_renderer"),
            DashboardDisplay::ResponseHeaders => context.set_focus("id", "response_headers"),
        }

        match *state.floating_window.to_ref() {
            FloatingWindow::None => {}
            FloatingWindow::Method => {
                context.set_focus("id", "method_selector");
            }
            FloatingWindow::AddHeader => context.set_focus("id", "add_header_window"),
            FloatingWindow::Error => {}
            FloatingWindow::Message => {}
            FloatingWindow::EditHeaderSelector => context.set_focus("id", "edit_header_selector"),
            FloatingWindow::Project => context.set_focus("id", "project_selector"),
            FloatingWindow::ConfirmAction => context.set_focus("id", "confirm_action_window"),
            FloatingWindow::ChangeEndpointName => context.set_focus("id", "edit_endpoint_name"),
            FloatingWindow::ChangeProjectName => context.set_focus("id", "edit_project_name"),
            FloatingWindow::EndpointsSelector => {
                context.set_focus("id", "endpoints_selector_window")
            }
            FloatingWindow::Commands => context.set_focus("id", "commands_window"),
            FloatingWindow::CodeGen => context.set_focus("id", "codegen_window"),
            FloatingWindow::PostmanFileSelector => context.set_focus("id", "postman_file_selector"),
            FloatingWindow::BodyModeSelector => context.set_focus("id", "body_mode_selector"),
            FloatingWindow::AddProjectVariable => context.set_focus("id", "add_project_variable"),
            FloatingWindow::ViewProjectVariables => context.set_focus("id", "project_variables"),
        }

        if self.test {
            return;
        }

        #[cfg(feature = "runtime_templates")]
        self.add_test_url(context);
    }

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let mut context_ref = RefCell::new(context);

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "method_box")
            .first(|_, _| {
                if *state.floating_window.to_ref() != FloatingWindow::None {
                    return;
                }

                if mouse.lsb_up() {
                    state.floating_window.set(FloatingWindow::Method);
                    context_ref.borrow_mut().set_focus("id", "method_selector");
                }
            });

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "url_component")
            .first(|_, _| {
                if *state.floating_window.to_ref() != FloatingWindow::None {
                    return;
                }

                if mouse.lsb_up() {
                    self.focus_url_input(&mut context_ref, false);
                }
            });

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "request_body_component")
            .first(|_, _| {
                if *state.floating_window.to_ref() != FloatingWindow::None {
                    return;
                }

                if mouse.lsb_up() {
                    context_ref.borrow_mut().set_focus("id", "textarea");
                }
            });

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "body_mode_hstack")
            .first(|_, _| {
                if *state.floating_window.to_ref() != FloatingWindow::None {
                    return;
                }

                if mouse.lsb_up() {
                    self.handle_y_press(state, &mut context_ref);
                }
            });
    }
}
