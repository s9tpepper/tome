use std::{cell::Ref, collections::HashMap};

use anathema::component::{ComponentId, Emitter};

pub mod add_header_window;
pub mod app_layout;
pub mod app_section;
pub mod confirm_action_window;
pub mod dashboard;
pub mod edit_header_selector;
pub mod edit_header_window;
pub mod edit_input;
pub mod floating_windows;
pub mod focusable_section;
pub mod header_name_textinput;
pub mod header_value_textinput;
pub mod inputs;
pub mod menu_item;
pub mod method_selector;
pub mod options;
pub mod project_window;
pub mod request_body_section;
pub mod request_headers_editor;
pub mod response_renderer;
pub mod row;
pub mod syntax_highlighter;
pub mod textarea;
pub mod textinput;

pub fn send_message(
    target: &str,
    message: String,
    component_ids: &Ref<'_, HashMap<String, ComponentId<String>>>,
    emitter: &Emitter,
) -> anyhow::Result<()> {
    if let Some(id) = component_ids.get(target) {
        return Ok(emitter.emit(*id, message)?);
    }

    Err(anyhow::Error::msg(format!(
        "Could not send message to {target}"
    )))
}
