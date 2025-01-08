use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use anathema::prelude::Context;
use ureq::Response;

use crate::{
    components::{
        dashboard::{DashboardComponent, DashboardDisplay, DashboardState, FloatingWindow},
        response_renderer::ResponseRendererMessages,
        send_message,
    },
    projects::{HeaderState, PersistedEndpoint},
};

pub fn do_request(
    state: &mut DashboardState,
    context: anathema::prelude::Context<'_, DashboardState>,
    _: anathema::widgets::Elements<'_, '_>,
    dashboard: &mut DashboardComponent,
) -> anyhow::Result<()> {
    let endpoint: PersistedEndpoint = (&*state.endpoint.to_ref()).into();

    let url = endpoint.url.clone();
    let method = endpoint.method.clone();
    let headers = endpoint.headers;

    let mut content_type = String::new();

    let mut request = ureq::request(&method, &url);
    for header_value in headers.iter() {
        let header = header_value;
        let header_name = header.name.to_string();
        let header_value = header.value.to_string();

        if header_name.to_lowercase() == "content-type" {
            content_type.push_str(header_value.as_str());
        }

        request = request.set(&header_name, &header_value);
    }

    let response = match content_type.as_str() {
        "application/json" => {
            let req_body = endpoint.body.clone();

            request.send_string(&req_body)
        }

        // TODO: Figure out how to support form k/v pairs in the request body builder interface
        // "multipart/form" => request.send_form("")
        //
        _ => request.send_string(""),
    };

    match response {
        Ok(response) => handle_successful_response(response, state, context, dashboard),
        Err(error) => handle_error_response(error, state, context, dashboard),
    }?;

    Ok(())
}

fn handle_successful_response(
    response: Response,
    state: &mut DashboardState,
    mut context: Context<'_, DashboardState>,
    dashboard: &mut DashboardComponent,
) -> anyhow::Result<()> {
    let status = response.status();

    loop {
        if state.response_headers.len() > 0 {
            state.response_headers.pop_back();
        } else {
            break;
        }
    }

    let mut ext = String::from("txt");
    for name in response.headers_names() {
        let Some(value) = response.header(&name) else {
            continue;
        };

        if name.to_lowercase() == "content-type" {
            // TODO: THis extension handling needs to be better than a split "/"
            // eg: application/json, text/plain, etc etc
            if let Some((_, extension)) = value.to_string().split_once("/") {
                ext = extension.to_string();
            }
        }

        state.response_headers.push(HeaderState {
            name: name.clone().into(),
            value: value.to_string().clone().into(),
        });
    }

    let mut response_path = PathBuf::from("/tmp");
    response_path.push("tome_response.txt");

    let mut response_reader = response.into_reader();
    let mut buf: Vec<u8> = vec![];
    response_reader.read_to_end(&mut buf)?;

    let mut file_path = PathBuf::from("/tmp");
    file_path.push("tome_response.txt");

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(file_path.clone())?;

    let write_result = file.write_all(buf.as_slice());
    // TODO: Fix the error handling to message the user
    if write_result.is_err() {
        return Ok(());
    }

    let window_label = format!("Response Body (Status Code: {status})");

    // TODO: Fix the response handling so it doesnt have to be read from file since
    // response renderer is reading it all into lines anyway
    let full_response = fs::read_to_string(file_path)?;
    state.response.set(full_response);

    state.response_body_window_label.set(window_label);
    state.main_display.set(DashboardDisplay::ResponseBody);

    context.set_focus("id", "response_renderer");

    let response_msg = ResponseRendererMessages::ResponseUpdate(ext);
    if let Ok(msg) = serde_json::to_string(&response_msg) {
        if let Ok(component_ids) = dashboard.component_ids.try_borrow() {
            let _ = send_message("response_renderer", msg, &component_ids, context.emitter);
        };
    };

    Ok(())
}

fn handle_error_response(
    error: ureq::Error,
    state: &mut DashboardState,
    mut context: Context<'_, DashboardState>,
    dashboard: &mut DashboardComponent,
) -> anyhow::Result<()> {
    match error {
        ureq::Error::Status(code, response) => {
            let body = response
                .into_string()
                .unwrap_or("Could not read error response body".to_string());
            let window_label = format!("Response Body (Status Code: {code})");

            // TODO: The error response handling needs to extract headers from the response
            // to display the response headers when there is an error

            state.response.set(body.clone());
            state.response_body_window_label.set(window_label);
            state.main_display.set(DashboardDisplay::ResponseBody);
            context.set_focus("id", "response_renderer");

            // TODO: Once the response headers are being extracted, figure out the correct
            // extension type to use to syntax highlight the response
            let response_msg = ResponseRendererMessages::SyntaxPreview(None);
            if let Ok(msg) = serde_json::to_string(&response_msg) {
                if let Ok(component_ids) = dashboard.component_ids.try_borrow() {
                    let _ = send_message("response_renderer", msg, &component_ids, context.emitter);
                };
            };

            Ok(())
        }

        ureq::Error::Transport(transport_error) => {
            let error = transport_error.message().unwrap_or("Network error");
            state.error_message.set(error.to_string());
            state.floating_window.set(FloatingWindow::Error);

            Ok(())
        }
    }
}
