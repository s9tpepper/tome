use anathema::{component::Component, state::State};

pub const REQUEST_HEADERS_EDITOR_TEMPLATE: &str =
    include_str!("./templates/request_headers_editor.aml");

#[derive(Default)]
pub struct RequestHeadersEditor;

#[derive(Default, State)]
pub struct RequestHeadersEditorState {}

impl RequestHeadersEditorState {
    pub fn new() -> Self {
        RequestHeadersEditorState {}
    }
}

impl Component for RequestHeadersEditor {
    type State = RequestHeadersEditorState;
    type Message = ();
}
