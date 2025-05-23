use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Arc,
};

use anathema::prelude::Context;
use anyhow::bail;
use mime::Mime;
use ureq::Response;
use ureq_multipart::MultipartBuilder;

use crate::{
    components::{
        dashboard::{DashboardComponent, DashboardDisplay, DashboardState},
        floating_windows::FloatingWindow,
        response_renderer::ResponseRendererMessages,
        send_message,
    },
    projects::{HeaderState, PersistedEndpoint, PersistedProject},
};

fn replace_variables(
    mut input: &str,
    variables: &HashMap<String, String>,
) -> anyhow::Result<String> {
    let (left, right) = input.as_bytes().windows(2).fold((0, 0), |mut a, s| {
        if s == b"{{" {
            a.0 += 1
        }

        if s == b"}}" {
            a.1 += 1
        }

        a
    });

    if left != right {
        bail!("The opening and closing brackets are imbalanced");
    }

    let mut replaced = String::from(input);

    while let Some(mut start) = input.find("{{") {
        if let Some(mut end) = input.find("}}") {
            start += 2;

            let variable = &input[start..end];

            if let Some(value) = variables.get(variable) {
                replaced = replaced.replace(&format!("{{{{{}}}}}", variable), value);
            }

            end += 2;
            input = &input[end..];
        } else {
            break;
        }
    }

    Ok(replaced)
}

#[test]
fn test_replace_variables() {
    let input = "https://{{baseUrl}}/something/something/something";
    let mut vars = HashMap::<String, String>::new();
    vars.insert("baseUrl".to_string(), "localhost".to_string());

    let replaced = replace_variables(input, &vars);

    let expect = "https://localhost/something/something/something".to_string();

    assert_eq!(replaced.unwrap(), expect);
}

#[test]
fn test_replace_variables_one_var() {
    let input = "{{baseUrl}}";
    let mut vars = HashMap::<String, String>::new();
    vars.insert("baseUrl".to_string(), "localhost".to_string());

    let replaced = replace_variables(input, &vars);

    let expect = "localhost".to_string();

    assert_eq!(replaced.unwrap(), expect);
}

#[test]
fn test_replace_multiple_variables() {
    let input = "https://{{baseUrl}}/something/something/{{path}}";
    let mut vars = HashMap::<String, String>::new();
    vars.insert("baseUrl".to_string(), "localhost".to_string());
    vars.insert("path".to_string(), "user".to_string());

    let replaced = replace_variables(input, &vars);

    let expect = "https://localhost/something/something/user".to_string();

    assert_eq!(replaced.unwrap(), expect);
}

#[test]
fn test_replace_multiple_incomplete_pairs() {
    let input = "https://{{baseUrl}/something/something/{{path}}";
    let mut vars = HashMap::<String, String>::new();
    vars.insert("baseUrl".to_string(), "localhost".to_string());
    vars.insert("path".to_string(), "user".to_string());

    let replaced = replace_variables(input, &vars);

    assert!(replaced.is_err());
}

#[test]
fn test_replace_multiple_incomplete_pairs2() {
    let input = "https://{{baseUrl}}/something/something/{{path}";
    let mut vars = HashMap::<String, String>::new();
    vars.insert("baseUrl".to_string(), "localhost".to_string());
    vars.insert("path".to_string(), "user".to_string());

    let replaced = replace_variables(input, &vars);

    assert!(replaced.is_err());
}

pub fn do_request(
    state: &mut DashboardState,
    context: &mut anathema::prelude::Context<'_, DashboardState>,
    _: &anathema::widgets::Elements<'_, '_>,
    dashboard: &mut DashboardComponent,
) -> anyhow::Result<()> {
    let project: PersistedProject = (&*state.project.to_ref()).into();
    let variables = project
        .variable
        .iter()
        .map(|variable| {
            (
                variable.key.clone().unwrap_or_default(),
                variable
                    .private
                    .clone()
                    .unwrap_or(variable.value.clone().unwrap_or_default()),
            )
        })
        .collect::<HashMap<String, String>>();

    let endpoint: PersistedEndpoint = (&*state.endpoint.to_ref()).into();

    let content_type = get_content_type(&endpoint);
    let mut url = endpoint.url.clone();

    url = replace_variables(&url, &variables)?;

    let method = endpoint.method.clone();
    let headers = endpoint.headers;

    let agent = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(native_tls::TlsConnector::new()?))
        .build();

    let mut request = agent.request(&method, &url);
    for header_value in headers.iter() {
        let header = header_value;
        let mut header_name = header.name.to_string();
        let mut header_value = header.value.to_string();

        header_name = replace_variables(&header_name, &variables)?;
        header_value = replace_variables(&header_value, &variables)?;

        // NOTE: Skip content-type header, this should be calculated based
        // on the body mode and/or raw type
        if header_name == "content-type" {
            continue;
        }

        request = request.set(&header_name, &header_value);
    }

    let response = match content_type {
        Some(content_type) => match content_type.as_str() {
            "application/json"
            | "application/javascript"
            | "text/plain"
            | "text/html"
            | "text/xml" => {
                let req_body = endpoint.body.clone();

                request.send_string(&req_body)
            }

            "x-www-form-urlencoded" => {
                let form: Vec<(&str, &str)> = endpoint
                    .body
                    .split("\n")
                    .filter_map(|entry| entry.split_once("="))
                    .collect();

                request.send_form(&form)
            }

            "multipart/form-data" => {
                let form: Vec<(&str, &str)> = endpoint
                    .body
                    .split("\n")
                    .filter_map(|entry| entry.split_once("="))
                    .collect();

                // TODO: Come back to this to remove/handle the .expect() call
                let mut builder = MultipartBuilder::new();
                builder = form.into_iter().fold(builder, |builder, (k, v)| {
                    builder
                        .add_text(k, v)
                        .expect("Should not error while adding headers?")
                });

                let (content_type, data) = builder.finish().unwrap();
                request.set("Content-Type", &content_type).send_bytes(&data)
            }

            _ => request.send_string(""),
        },

        None => request.send_string(""),
    };

    match response {
        Ok(response) => handle_successful_response(response, state, context, dashboard),
        Err(error) => handle_error_response(error, state, context, dashboard),
    }?;

    Ok(())
}

fn get_content_type(endpoint: &PersistedEndpoint) -> Option<String> {
    match endpoint.body_mode.as_str() {
        "none" => None,
        "formdata" => Some("multipart/form-data".to_string()),
        "x-www-form-urlencoded" => Some("application/x-www-form-urlencoded".to_string()),
        "graphql" => Some("application/json".to_string()),
        "raw" => match endpoint.raw_type.to_lowercase().as_str() {
            "text" => Some("text/plain".to_string()),
            "javascript" => Some("application/javascript".to_string()),
            "json" => Some("application/json".to_string()),
            "html" => Some("text/html".to_string()),
            "xml" => Some("text/xml".to_string()),

            _ => None,
        },

        _ => None,
    }
}

fn get_extension(content_type: &str) -> String {
    let mime: Mime = content_type.parse().unwrap_or(mime::TEXT_PLAIN);

    let name = mime.subtype();
    match name.as_str() {
        "plain" => "txt".to_string(),
        ext => ext.to_string(),
    }
}

fn handle_successful_response(
    response: Response,
    state: &mut DashboardState,
    context: &mut Context<'_, DashboardState>,
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
            ext = get_extension(value);
        }

        state.response_headers.push(HeaderState {
            name: name.clone().into(),
            value: value.to_string().clone().into(),
            row_color: "".to_string().into(),
            row_fg_color: "".to_string().into(),
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
    context: &mut Context<'_, DashboardState>,
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
