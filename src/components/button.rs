use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Component, ComponentId, MouseEvent},
    prelude::{Context, ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::{State, Value},
    widgets::Elements,
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

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        _: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let context_ref = RefCell::new(context);

        let attribute = {
            let c = context_ref.borrow();
            let Some(button_id) = c.get_external("button_id") else {
                return;
            };

            let Some(common_val) = button_id.to_common() else {
                return;
            };

            &*common_val.to_string()
        };

        elements
            .at_position(mouse.pos())
            .by_attribute("id", attribute)
            .first(|_, _| {
                if mouse.lsb_up() {
                    context_ref
                        .borrow_mut()
                        .publish("click", |state| &state.button_id);
                }
            });
    }
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
