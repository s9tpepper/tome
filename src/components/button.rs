use std::cell::RefCell;

use anathema::{
    component::{Children, Component, Context, MouseEvent},
    prelude::ToSourceKind,
    runtime::Builder,
    state::{State, Value},
};

use crate::{
    options::get_button_caps,
    theme::{get_app_theme, AppTheme},
};

pub struct Button {
    #[allow(dead_code)]
    app_theme: AppTheme,
}

#[derive(State)]
pub struct ButtonState {
    button_id: Value<String>,
    button_cap_left: Value<String>,
    button_cap_right: Value<String>,
}

impl ButtonState {
    pub fn new(id: &str) -> Self {
        let button_caps = get_button_caps();

        ButtonState {
            button_id: id.to_string().into(),
            button_cap_left: button_caps.0.to_string().into(),
            button_cap_right: button_caps.1.to_string().into(),
        }
    }
}

impl Component for Button {
    type State = ButtonState;
    type Message = String;

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        let context_ref = RefCell::new(context);

        let attribute = {
            let c = context_ref.borrow();
            let Some(button_id) = c.attribute("button_id") else {
                return;
            };

            let Some(common_val) = button_id.as_str() else {
                return;
            };

            &*common_val.to_string()
        };

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", attribute)
            .first(|_, _| {
                if mouse.left_up() {
                    context_ref
                        .borrow_mut()
                        .publish("click", state.button_id.to_ref().clone());
                }
            });
    }
}

impl Button {
    pub fn register(builder: &mut Builder<()>, template: impl ToSourceKind) -> anyhow::Result<()> {
        builder.prototype(
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
