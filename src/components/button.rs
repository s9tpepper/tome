use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId},
    prelude::{ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::{State, Value},
};

use crate::{
    app::GlobalEventHandler,
    theme::{get_app_theme, AppTheme},
};

pub struct Button {
    #[allow(dead_code)]
    app_theme: AppTheme,
}

#[derive(State)]
pub struct ButtonState {
    button_id: Value<String>,
}

impl ButtonState {
    pub fn new(id: &str) -> Self {
        ButtonState {
            button_id: id.to_string().into(),
        }
    }
}

impl Component for Button {
    type State = ButtonState;
    type Message = String;
}

impl Button {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
        template: impl ToSourceKind,
    ) -> anyhow::Result<()> {
        builder.register_prototype(
            "button",
            template,
            || Button {
                app_theme: get_app_theme(),
            },
            || ButtonState::new("button"),
        )?;

        Ok(())
    }
}
