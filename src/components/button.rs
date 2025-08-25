use std::cell::RefCell;

use anathema::{
    component::{Children, Component, Context, MouseEvent},
    prelude::ToSourceKind,
    runtime::Builder,
    state::{State, Value},
};

use crate::theme::{get_app_theme, AppTheme};

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
                        .publish("click", |state: Self::State| {
                            state.button_id.to_ref().clone()
                        });
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
