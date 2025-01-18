use anathema::{component::Component, state::State};

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
