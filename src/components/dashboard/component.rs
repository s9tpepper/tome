use std::cell::RefCell;

use anathema::{
    component::{Children, Component, Context, KeyCode, KeyEvent, MouseEvent},
    widgets::components::events::KeyState,
};
use log::info;

use crate::{
    components::{
        app_layout::AppLayoutMessages, floating_windows::FloatingWindow, send_message,
        textarea::TextAreaMessages, textinput::TextInputMessages,
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

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
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
                    context.components.by_attribute("id", "app").focus();
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
                            context.components.by_attribute("id", "app").focus();

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

    fn on_event(
        &mut self,
        event: &mut anathema::component::UserEvent<'_>,
        state: &mut Self::State,
        children: anathema::component::Children<'_, '_>,
        context: anathema::component::Context<'_, '_, Self::State>,
    ) {
        associated_functions(self, event, context, state, children);
    }

    fn on_key(
        &mut self,
        event: KeyEvent,
        state: &mut Self::State,
        elements: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        keyboard_events(self, event, state, elements, context);
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        info!("Dashboard received focus");

        update_theme(state);

        match *state.main_display.to_ref() {
            DashboardDisplay::RequestBody => {
                context.components.by_attribute("id", "request").focus()
            }

            DashboardDisplay::RequestHeadersEditor => context
                .components
                .by_attribute("id", "request_headers_editor")
                .focus(),
            DashboardDisplay::ResponseBody => context
                .components
                .by_attribute("id", "response_renderer")
                .focus(),
            DashboardDisplay::ResponseHeaders => context
                .components
                .by_attribute("id", "response_headers")
                .focus(),
        }

        match *state.floating_window.to_ref() {
            FloatingWindow::None => {}
            FloatingWindow::Method => {
                context
                    .components
                    .by_attribute("id", "method_selector")
                    .focus();
            }
            FloatingWindow::AddHeader => context
                .components
                .by_attribute("id", "add_header_window")
                .focus(),
            FloatingWindow::Error => {}
            FloatingWindow::Message => {}
            FloatingWindow::EditHeaderSelector => context
                .components
                .by_attribute("id", "edit_header_selector")
                .focus(),
            FloatingWindow::Project => context
                .components
                .by_attribute("id", "project_selector")
                .focus(),
            FloatingWindow::ConfirmAction => context
                .components
                .by_attribute("id", "confirm_action_window")
                .focus(),
            FloatingWindow::ChangeEndpointName => context
                .components
                .by_attribute("id", "edit_endpoint_name")
                .focus(),
            FloatingWindow::ChangeProjectName => context
                .components
                .by_attribute("id", "edit_project_name")
                .focus(),
            FloatingWindow::EndpointsSelector => context
                .components
                .by_attribute("id", "endpoints_selector_window")
                .focus(),
            FloatingWindow::Commands => context
                .components
                .by_attribute("id", "commands_window")
                .focus(),
            FloatingWindow::CodeGen => context
                .components
                .by_attribute("id", "codegen_window")
                .focus(),
            FloatingWindow::PostmanFileSelector => context
                .components
                .by_attribute("id", "postman_file_selector")
                .focus(),
            FloatingWindow::BodyModeSelector => context
                .components
                .by_attribute("id", "body_mode_selector")
                .focus(),
            FloatingWindow::AddProjectVariable => context
                .components
                .by_attribute("id", "add_project_variable")
                .focus(),
            FloatingWindow::ViewProjectVariables => context
                .components
                .by_attribute("id", "project_variables")
                .focus(),
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
        mut children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        let mut context_ref = RefCell::new(context);

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "method_box")
            .first(|_, _| {
                if *state.floating_window.to_ref() != FloatingWindow::None {
                    return;
                }

                if mouse.left_up() {
                    state.floating_window.set(FloatingWindow::Method);
                    context_ref
                        .borrow_mut()
                        .components
                        .by_attribute("id", "method_selector")
                        .focus();
                }
            });

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "url_component")
            .first(|_, _| {
                if *state.floating_window.to_ref() != FloatingWindow::None {
                    return;
                }

                if mouse.left_up() {
                    self.focus_url_input(&mut context_ref, false);
                }
            });

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "request_body_component")
            .first(|_, _| {
                if *state.floating_window.to_ref() != FloatingWindow::None {
                    return;
                }

                if mouse.left_up() {
                    context_ref
                        .borrow_mut()
                        .components
                        .by_attribute("id", "textarea")
                        .focus();
                }
            });

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "body_mode_display")
            .first(|_, _| {
                if *state.floating_window.to_ref() != FloatingWindow::None {
                    return;
                }

                if mouse.left_up() {
                    self.handle_y_press(state, &mut context_ref);
                }
            });
    }

    fn on_mount(
        &mut self,
        _: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        // TODO: Get rid of all the serde_json serialize/deserialize for messages
        let Ok(message) = serde_json::to_string(&AppLayoutMessages::DashboardMounted) else {
            return;
        };

        context.components.by_name("app").send(message);
    }

    fn on_unmount(
        &mut self,
        _: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        info!("dashboard_component::on_unmount()");
    }
}
