use std::ops::Deref;

use anathema::prelude::Context;

use crate::{
    components::floating_windows::FloatingWindow,
    messages::confirm_actions::ConfirmAction,
    projects::{
        delete_endpoint, delete_project, save_project, Endpoint, PersistedEndpoint,
        PersistedProject, PersistedVariable, Project,
    },
};

use super::{DashboardComponent, DashboardState};

pub fn handle_confirmations(
    dashboard: &DashboardComponent,
    confirm_action: ConfirmAction,
    state: &mut DashboardState,
    mut context: Context<'_, DashboardState>,
) {
    match confirm_action {
        ConfirmAction::ConfirmationDeleteHeader(delete_header_answer) => match delete_header_answer
            .answer
        {
            true => {
                let header = delete_header_answer.data;
                let mut current_endpoint: PersistedEndpoint =
                    state.endpoint.to_ref().deref().into();

                // TODO: Fix bug where this call to remove expects that the endpoint has already
                // been saved, and if it hasn't this panics. Make this not panic.
                current_endpoint.headers.remove(
                    current_endpoint
                        .headers
                        .iter()
                        .enumerate()
                        .find(|(_, h)| h.name == header.name)
                        .map(|(ndx, _)| ndx)
                        .expect("Header should exist in the current endpoint"),
                );

                let mut current_project: PersistedProject = state.project.to_ref().deref().into();

                let e = current_project
                    .endpoints
                    .iter_mut()
                    .find(|endpoint| endpoint.name == current_endpoint.name)
                    .expect("Expect the endpoint to exist");

                let new_endpoint: Endpoint = (&current_endpoint).into();
                e.headers = current_endpoint.headers;

                let result =
                    delete_project(&current_project).map(|_| save_project(&current_project));

                match result {
                    Ok(_) => {
                        let new_project: Project = (&current_project).into();
                        state.project.set(new_project);
                        state.endpoint.set(new_endpoint);

                        dashboard.show_message(
                            "Delete Header",
                            "Header was deleted successfully",
                            state,
                        );
                    }
                    Err(_) => dashboard.show_error("There was an error deleting the header", state),
                }
            }

            false => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");
            }
        },

        ConfirmAction::ConfirmationDeletePersistedProject(delete_project_details_answer) => {
            match delete_project_details_answer.answer {
                true => {
                    // TODO: Verify this line is correct, because this deletes the current
                    // project that's selected for the dashboard, and I believe that this
                    // should be deleting the project in delete_project_details_answer.project
                    //
                    let persisted_project: PersistedProject =
                        state.project.to_ref().as_ref().into();

                    state.project.set(Project::new());
                    state.endpoint.set(Endpoint::new());
                    dashboard.clear_url_and_request_body(&context);

                    let project_name = persisted_project.name.clone();
                    let delete_result = delete_project(&persisted_project);
                    match delete_result {
                        Ok(_) => dashboard.show_message(
                            "Delete Project Success",
                            &format!("Project '{project_name}' has been deleted"),
                            state,
                        ),

                        Err(_) => dashboard.show_error(
                            &format!("There was an error deleting '{project_name}"),
                            state,
                        ),
                    }
                }

                false => {
                    state.floating_window.set(FloatingWindow::None);
                    context.set_focus("id", "app");
                }
            }
        }

        ConfirmAction::ConfirmationDeletePersistedEndpoint(delete_endpoint_details_answer) => {
            match delete_endpoint_details_answer.answer {
                true => {
                    let persisted_endpoint = delete_endpoint_details_answer.data;
                    let current_endpoint: PersistedEndpoint =
                        state.endpoint.to_ref().deref().into();

                    if persisted_endpoint == current_endpoint {
                        state.endpoint.set(Endpoint::new());
                        dashboard.clear_url_and_request_body(&context);
                    }

                    let endpoint_name = persisted_endpoint.name.clone();
                    let mut current_project: PersistedProject =
                        state.project.to_ref().deref().into();
                    match delete_endpoint(&mut current_project, &persisted_endpoint) {
                        Ok(_) => {
                            let project: Project = (&current_project).into();
                            state.project.set(project);

                            dashboard.show_message(
                                "Delete Endpoint Success",
                                &format!("The {endpoint_name} endpoint has been deleted."),
                                state,
                            );
                        }
                        Err(_) => dashboard.show_error(
                            &format!("There was an error deleting the {endpoint_name} endpoint"),
                            state,
                        ),
                    }
                }

                false => {
                    state.floating_window.set(FloatingWindow::None);
                    context.set_focus("id", "app");
                }
            }
        }

        ConfirmAction::ConfirmationDeletePersistedVariable(delete_persisted_variable_answer) => {
            match delete_persisted_variable_answer.answer {
                true => {
                    let persisted_variable = delete_persisted_variable_answer.data;
                    let deleted_variable = persisted_variable.name.clone().unwrap_or_default();

                    let mut delete_index: i64 = -1;
                    for (index, var) in state.project.to_ref().variable.to_ref().iter().enumerate()
                    {
                        let project_persisted_variable: PersistedVariable =
                            (var.to_ref().deref()).into();

                        if project_persisted_variable == persisted_variable {
                            delete_index = index as i64;
                            break;
                        }
                    }

                    if delete_index > -1 {
                        state
                            .project
                            .to_mut()
                            .variable
                            .remove(delete_index as usize);

                        let project: PersistedProject = state.project.to_ref().deref().into();
                        match save_project(&project) {
                            Ok(_) => {
                                let title = format!("Delete '{deleted_variable}'");
                                let message = "Variable deleted successfully";
                                dashboard.show_message(&title, message, state);
                            }
                            Err(_) => {
                                let message = format!("Error deleting '{deleted_variable}'");
                                dashboard.show_error(&message, state);
                            }
                        }
                    }
                }

                false => {
                    state.floating_window.set(FloatingWindow::None);
                    context.set_focus("id", "app");
                }
            }
        }

        _ => {}
    }
}
