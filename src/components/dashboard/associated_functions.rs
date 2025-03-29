use anathema::{prelude::Context, state::CommonVal, widgets::Elements};
use log::info;

use crate::components::{
    add_header_window::AddHeaderWindow,
    confirm_action_window::ConfirmActionWindow,
    edit_header_selector::EditHeaderSelector,
    floating_windows::{
        add_project_variable::{AddProjectVariable, AddProjectVariableMessages},
        body_mode_selector::BodyModeSelector,
        code_gen::CodeGen,
        commands::Commands,
        edit_endpoint_name::EditEndpointName,
        edit_project_name::EditProjectName,
        endpoints_selector::EndpointsSelector,
        file_selector::FileSelector,
        project_variables::ProjectVariables,
        FloatingWindow,
    },
    method_selector::MethodSelector,
    project_window::ProjectWindow,
    send_message,
};

use super::{DashboardComponent, DashboardDisplay, DashboardMessageHandler, DashboardState};

pub fn associated_functions(
    dashboard: &mut DashboardComponent,
    ident: &str,
    value: CommonVal<'_>,
    mut context: Context<'_, DashboardState>,
    state: &mut DashboardState,
    elements: Elements<'_, '_>,
) {
    let current_display = *state.main_display.to_ref();
    let is_request_body = current_display == DashboardDisplay::RequestBody;
    let is_headers_editor = current_display == DashboardDisplay::RequestHeadersEditor;
    let is_response_body = current_display == DashboardDisplay::ResponseBody;
    let is_response_headers = current_display == DashboardDisplay::ResponseHeaders;

    info!("associated_functions(): current_display: {current_display:?}, is_request_body: {is_request_body}, is_headers_editor: {is_headers_editor}");

    #[allow(clippy::single_match)]
    match ident {
        // Unfocus the url input and set back to dashboard
        "url_input_focus" => {
            context.set_focus("id", "app");
        }

        "add_new_project" => {
            dashboard.new_project(state, &mut context);
        }

        "project_name_click" => dashboard.open_edit_project_name_window(state, &mut context),
        "endpoint_name_click" => dashboard.open_edit_endpoint_name_window(state, &mut context),
        "new_project_click" => dashboard.new_project(state, &mut context),
        "new_endpoint_click" => dashboard.new_endpoint(state, &mut context),
        "commands_button_click" => dashboard.open_commands_window(state, &mut context),

        "send_back_to_request_from_response_renderer_click" if is_response_body => {
            dashboard.go_back(state, &mut context);
        }

        "send_show_response_headers_click" if is_response_body => {
            dashboard.go_to_headers(state, &mut context);
        }

        "send_copy_response_click" if is_response_body => {
            dashboard.handle_y_press(state, &mut context);
        }

        "send_save_response_click" if is_response_body => {
            dashboard.send_save_response(state);
        }

        "send_request_click" if is_request_body => {
            dashboard.send_request(state, &mut context, &elements)
        }

        "show_request_headers" if is_request_body => {
            dashboard.show_request_headers(None, state, &mut context)
        }

        "send_request_click_request_body" if is_headers_editor => {
            dashboard.send_request(state, &mut context, &elements)
        }

        "add_header_click" if is_headers_editor => {
            dashboard.open_add_header_window(state, &mut context)
        }

        "edit_header_click" if is_headers_editor => {
            dashboard.open_edit_header_window(state, &mut context)
        }

        "back_to_request_click" if is_headers_editor => {
            state.main_display.set(DashboardDisplay::RequestBody)
        }

        "save_project_click" => dashboard.save_project(state, true),
        "save_endpoint_click" => dashboard.save_endpoint(state, true),
        "swap_project_click" => dashboard.open_projects_window(state, &mut context),
        "swap_endpoint_click" => dashboard.open_endpoints_selector(state, &mut context),
        "options_button_click" => dashboard.send_options_open(&mut context),

        "rename_project" => {
            dashboard.rename_project(&value.to_string(), state, &mut context);
        }

        "rename_endpoint" => {
            dashboard.rename_endpoint(&value.to_string(), state, &mut context);
        }

        "rename_variable" => {
            dashboard.rename_variable(&value.to_string(), state, &mut context);
        }

        "open_add_variable_window" => {
            state
                .floating_window
                .set(FloatingWindow::AddProjectVariable);
            context.set_focus("id", "add_project_variable");

            let Ok(message) = serde_json::to_string(&AddProjectVariableMessages::InitialFocus)
            else {
                return;
            };

            let Ok(ids) = dashboard.component_ids.try_borrow() else {
                return;
            };

            let _ = send_message("add_project_variable", message, &ids, context.emitter);
        }

        _ => {}
    }

    let (component, _event) = ident.split_once("__").unwrap_or(("", ""));

    if let Ok(component_ids) = dashboard.component_ids.try_borrow() {
        match component {
            "confirm_action" => ConfirmActionWindow::handle_message(
                value,
                ident,
                state,
                context,
                elements,
                component_ids,
            ),

            "project_variables" => ProjectVariables::handle_message(
                value,
                ident,
                state,
                context,
                elements,
                component_ids,
            ),

            "add_project_variable" => AddProjectVariable::handle_message(
                value,
                ident,
                state,
                context,
                elements,
                component_ids,
            ),

            "body_mode_selector" => BodyModeSelector::handle_message(
                value,
                ident,
                state,
                context,
                elements,
                component_ids,
            ),

            "file_selector" => {
                FileSelector::handle_message(value, ident, state, context, elements, component_ids);
            }

            "commands" => {
                Commands::handle_message(value, ident, state, context, elements, component_ids);
            }

            "codegen" => {
                CodeGen::handle_message(value, ident, state, context, elements, component_ids);
            }

            "add_header" => {
                AddHeaderWindow::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                );
            }

            "edit_header_selector" => {
                EditHeaderSelector::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                );
            }

            "method_selector" => {
                MethodSelector::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                );
            }

            "project_window" => {
                ProjectWindow::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                );
            }

            "edit_endpoint_name" => {
                EditEndpointName::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                );
            }

            "edit_project_name" => {
                EditProjectName::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                );
            }

            "endpoints_selector" => {
                EndpointsSelector::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    elements,
                    component_ids,
                );
            }

            _ => {}
        }
    } else {
        println!("Could not find id for {ident}");
    }
}
