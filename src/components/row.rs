use anathema::{component::Component, state::State};

pub const ROW_TEMPLATE: &str = include_str!("./templates/row.aml");

#[derive(Default)]
pub struct Row;

#[derive(Default, State)]
pub struct RowState {}

impl RowState {
    pub fn new() -> Self {
        RowState {}
    }
}

impl Component for Row {
    type State = RowState;
    type Message = ();

    fn accept_focus(&self) -> bool {
        false
    }
}
