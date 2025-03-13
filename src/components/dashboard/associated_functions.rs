use anathema::{prelude::Context, state::CommonVal, widgets::Elements};

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

use super::{DashboardComponent, DashboardMessageHandler, DashboardState};

pub fn associated_functions(
    dashboard: &mut DashboardComponent,
    ident: &str,
    value: CommonVal<'_>,
    mut context: Context<'_, DashboardState>,
    state: &mut DashboardState,
    elements: Elements<'_, '_>,
) {
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
