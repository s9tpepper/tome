use anathema::{
    component::{KeyCode, KeyEvent},
    prelude::Context,
    widgets::Elements,
};

use crate::{
    components::{
        edit_header_selector::EditHeaderSelectorMessages, floating_windows::FloatingWindow,
        send_message,
    },
    fs::save_response,
    projects::Header,
    requests::do_request,
};

use super::{DashboardComponent, DashboardDisplay, DashboardState};

pub fn keyboard_events(
    dashboard: &mut DashboardComponent,
    event: KeyEvent,
    state: &mut DashboardState,
    elements: Elements<'_, '_>,
    mut context: Context<'_, DashboardState>,
) {
    match event.code {
        KeyCode::Char(char) => {
            let main_display = *state.main_display.to_ref();

            match char {
                'c' => dashboard.open_commands_window(state, context),
                's' => dashboard.save_project(state, true),
                'n' => dashboard.open_edit_endpoint_name_window(state, &mut context),
                'j' => dashboard.open_edit_project_name_window(state, &mut context),
                'i' => dashboard.save_endpoint(state, &context, true),
                // 'f' => context.set_focus("id", "response_body_input"),
                'o' => dashboard.send_options_open(state, context),
                't' => dashboard.new_endpoint(state, &mut context),
                'w' => dashboard.new_project(state, &mut context),

                'v' => match main_display {
                    DashboardDisplay::RequestBody => {}
                    DashboardDisplay::RequestHeadersEditor => {}
                    DashboardDisplay::ResponseBody => save_response(dashboard, state, context),
                    DashboardDisplay::ResponseHeaders => {}
                },

                // Set focus to the request url text input
                'u' => dashboard.focus_url_input(&mut context.into(), event.ctrl),

                // Quit app
                'q' => quit::with_code(0),

                // Make the request
                'r' => {
                    let response = do_request(state, context, elements, dashboard);
                    match response {
                        Ok(_) => {}
                        Err(err) => {
                            dashboard.show_error(&err.to_string(), state);
                        }
                    }
                }

                // Show request body editor window
                'b' => match main_display {
                    DashboardDisplay::RequestBody => context.set_focus("id", "textarea"),
                    DashboardDisplay::RequestHeadersEditor => {
                        state.main_display.set(DashboardDisplay::RequestBody);
                    }
                    DashboardDisplay::ResponseBody => {
                        state.main_display.set(DashboardDisplay::RequestBody);
                        context.set_focus("id", "app");
                    }
                    DashboardDisplay::ResponseHeaders => {
                        state.main_display.set(DashboardDisplay::ResponseBody)
                    }
                },

                // Show request headers editor window
                'd' => {
                    if !event.ctrl {
                        state
                            .main_display
                            .set(DashboardDisplay::RequestHeadersEditor);
                    }
                }

                // Open Endpoints selector
                'e' => {
                    dashboard.open_endpoints_selector(state, context);
                }

                // Show projects window
                'p' => {
                    if let Ok(component_ids) = dashboard.component_ids.try_borrow() {
                        state.floating_window.set(FloatingWindow::Project);
                        context.set_focus("id", "project_selector");

                        let _ = component_ids.get("project_selector").map(|id| {
                            context.emit(*id, "projects".to_string());
                        });
                    }
                }

                // Show response headers display
                'h' => match main_display {
                    DashboardDisplay::RequestBody => {}
                    DashboardDisplay::RequestHeadersEditor => {
                        state
                            .floating_window
                            .set(FloatingWindow::EditHeaderSelector);
                        context.set_focus("id", "edit_header_selector");

                        let headers: Vec<Header> = state
                            .endpoint
                            .to_ref()
                            .headers
                            .to_ref()
                            .iter()
                            .map(|header_state| (&*header_state.to_ref()).into())
                            .collect();

                        let edit_header_selector_messages =
                            EditHeaderSelectorMessages::HeadersList(headers);

                        let Ok(message) = serde_json::to_string(&edit_header_selector_messages)
                        else {
                            return;
                        };

                        let Ok(ids) = dashboard.component_ids.try_borrow() else {
                            return;
                        };

                        let _ =
                            send_message("edit_header_selector", message, &ids, context.emitter);
                    }
                    DashboardDisplay::ResponseBody => {
                        state.main_display.set(DashboardDisplay::ResponseHeaders)
                    }
                    DashboardDisplay::ResponseHeaders => {}
                },

                // Open Request Method selection window
                'm' => {
                    state.floating_window.set(FloatingWindow::Method);
                    context.set_focus("id", "method_selector");
                }

                'a' => match main_display {
                    DashboardDisplay::RequestBody => {}
                    DashboardDisplay::RequestHeadersEditor => {
                        // Open header window
                        dashboard.open_add_header_window(state, context);
                    }
                    DashboardDisplay::ResponseBody => {}
                    DashboardDisplay::ResponseHeaders => {}
                },

                'y' => match main_display {
                    DashboardDisplay::RequestBody => {
                        dashboard.open_body_mode_selector(state, context)
                    }
                    DashboardDisplay::RequestHeadersEditor => {}
                    DashboardDisplay::ResponseBody => {
                        // Copy response body to clipboard
                        dashboard.yank_response(state)
                    }
                    DashboardDisplay::ResponseHeaders => {}
                },

                _ => {}
            }
        }

        KeyCode::Esc => {
            context.set_focus("id", "app");

            if *state.floating_window.to_ref() != FloatingWindow::None {
                state.floating_window.set(FloatingWindow::None);
            // NOTE: Test the UX of the focus movement with this Esc binding
            } else if *state.main_display.to_ref() != DashboardDisplay::RequestBody {
                state.main_display.set(DashboardDisplay::RequestBody);
            }
        }

        KeyCode::Enter => {
            // TODO: Do something with the Enter button
        }

        _ => {}
    }
}
