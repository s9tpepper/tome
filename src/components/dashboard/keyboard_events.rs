use anathema::component::{Children, Context, KeyCode, KeyEvent};

use crate::components::floating_windows::FloatingWindow;

use super::{DashboardComponent, DashboardDisplay, DashboardState};

pub fn keyboard_events(
    dashboard: &mut DashboardComponent,
    event: KeyEvent,
    state: &mut DashboardState,
    elements: Children<'_, '_>,
    mut context: Context<'_, '_, DashboardState>,
) {
    match event.code {
        KeyCode::Char(char) => {
            let main_display = *state.main_display.to_ref();

            match char {
                'c' => dashboard.open_commands_window(state, &mut context),
                's' => dashboard.save_project(state, true),
                'n' => dashboard.open_edit_endpoint_name_window(state, &mut context),
                'j' => dashboard.open_edit_project_name_window(state, &mut context),
                'i' => dashboard.save_endpoint(state, true),
                // 'f' => context.set_focus("id", "response_body_input"),
                'o' => dashboard.send_options_open(&mut context),
                't' => dashboard.new_endpoint(state, &mut context),
                'w' => dashboard.new_project(state, &mut context),

                'v' => dashboard.send_save_response(state),

                // Set focus to the request url text input
                'u' => dashboard.focus_url_input(&mut context.into(), event.ctrl),

                // Quit app
                'q' => quit::with_code(0),

                // Make the request
                'r' => dashboard.send_request(state, &mut context, &elements),

                // Show request body editor window
                'b' => dashboard.go_back(state, &mut context),

                // Show request headers editor window
                'd' => dashboard.show_request_headers(Some(event), state, &mut context),

                // Open Endpoints selector
                'e' => {
                    dashboard.open_endpoints_selector(state, &mut context);
                }

                // Show projects window
                'p' => dashboard.open_projects_window(state, &mut context),

                // Show response headers display
                'h' => dashboard.go_to_headers(state, &mut context),

                // Open Request Method selection window
                'm' => {
                    state.floating_window.set(FloatingWindow::Method);
                    context
                        .components
                        .by_attribute("id", "method_selector")
                        .focus();
                }

                'a' => match main_display {
                    DashboardDisplay::RequestBody => {}
                    DashboardDisplay::RequestHeadersEditor => {
                        // Open header window
                        dashboard.open_add_header_window(state, &mut context);
                    }
                    DashboardDisplay::ResponseBody => {}
                    DashboardDisplay::ResponseHeaders => {}
                },

                'y' => dashboard.handle_y_press(state, &mut context.into()),

                _ => {}
            }
        }

        KeyCode::Esc => {
            context.components.by_attribute("id", "app").focus();

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
