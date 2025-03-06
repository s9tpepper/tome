use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum FocusChange {
    Focused,
    Unfocused,
}
