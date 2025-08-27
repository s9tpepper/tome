use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use ::anathema::state::State;
use anathema::{
    component::{
        self, Children, Component, ComponentId, Context, KeyCode, KeyEvent, MouseEvent, UserEvent,
    },
    runtime::Builder,
    state::Value,
};

use serde::{Deserialize, Serialize};

use crate::{
    options::get_button_caps,
    projects::{Header, HeaderState},
    templates::template,
    theme::{get_app_theme, AppTheme},
};

use super::{dashboard::DashboardMessageHandler, floating_windows::FloatingWindow, send_message};

#[derive(Default)]
pub struct AddHeaderWindow {
    #[allow(unused)]
    pub component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl AddHeaderWindow {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
    ) -> anyhow::Result<()> {
        let name: String = "add_header_window".to_string();

        let app_id = builder.component(
            name.clone(),
            template("templates/add_header_window"),
            AddHeaderWindow {
                component_ids: ids.clone(),
            },
            AddHeaderWindowState::new(),
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(name, app_id);

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub enum AddHeaderWindowMessages {
    Specifically((String, Header, Vec<String>)),
}

impl AddHeaderWindow {
    fn update_app_theme(&self, state: &mut AddHeaderWindowState) {
        let app_theme = get_app_theme();
        state.app_theme.set(app_theme);
    }

    pub fn set_values_for_inputs(
        &self,
        state: &AddHeaderWindowState,
        context: Context<'_, '_, AddHeaderWindowState>,
    ) {
        let Ok(ids) = self.component_ids.try_borrow() else {
            return;
        };

        let _ = send_message(
            "headernameinput",
            state.header.to_ref().name.to_ref().to_string(),
            &ids,
            context.emitter,
        );

        let _ = send_message(
            "headervalueinput",
            state.header.to_ref().value.to_ref().to_string(),
            &ids,
            context.emitter,
        );
    }

    pub fn submit(
        &self,
        state: &mut AddHeaderWindowState,
        context: &mut RefCell<Context<'_, '_, AddHeaderWindowState>>,
    ) {
        let header = state.header.to_ref();
        let new_header = NewHeader {
            name: header.name.to_ref().clone().into(),
            value: header.value.to_ref().clone().into(),
        };

        context
            .borrow_mut()
            .publish("add_header__submit", new_header);

        state.active = false;
    }

    pub fn cancel(
        &self,
        state: &mut AddHeaderWindowState,
        context: &mut RefCell<Context<'_, '_, AddHeaderWindowState>>,
    ) {
        context
            .borrow_mut()
            .publish("add_header__cancel", None::<()>);

        state.active = false;
    }
}

#[derive(Default, State)]
pub struct AddHeaderWindowState {
    header: Value<NewHeader>,
    unique_name_error: Value<String>,
    success_button_color: Value<String>,
    cancel_button_color: Value<String>,
    app_theme: Value<AppTheme>,
    button_cap_left: Value<String>,
    button_cap_right: Value<String>,

    #[state_ignore]
    active: bool,

    #[state_ignore]
    current_name: String,

    #[state_ignore]
    current_names: Vec<String>,

    #[state_ignore]
    success_color_focused: String,

    #[state_ignore]
    cancel_color_focused: String,

    #[state_ignore]
    button_color_unfocused: String,
}

impl AddHeaderWindowState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        let submit_bg = app_theme.overlay_submit_background.to_ref().to_string();
        let cancel_bg = app_theme.overlay_cancel_background.to_ref().to_string();
        let unfocused_bg = app_theme.border_unfocused.to_ref().to_string();

        // TODO: Update these cap values when the options change
        let (left, right) = get_button_caps();

        AddHeaderWindowState {
            active: false,
            app_theme: app_theme.into(),
            button_cap_left: left.to_string().into(),
            button_cap_right: right.to_string().into(),

            current_name: "".to_string(),
            current_names: vec![],

            header: NewHeader::default().into(),
            unique_name_error: "".to_string().into(),

            success_color_focused: submit_bg,
            cancel_color_focused: cancel_bg,
            button_color_unfocused: unfocused_bg.clone(),
            success_button_color: unfocused_bg.clone().into(),
            cancel_button_color: unfocused_bg.into(),
        }
    }
}

impl DashboardMessageHandler for AddHeaderWindow {
    fn handle_message(
        event: &mut UserEvent<'_>,

        // TODO: Need to remove this, ident is now event.name()
        _ident: impl Into<String>,

        state: &mut super::dashboard::DashboardState,
        mut context: Context<'_, '_, super::dashboard::DashboardState>,

        // TODO: Remove these two, they're now unneeded
        _: Children<'_, '_>,
        _component_ids: Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        match event.name() {
            "add_header__name_update" => {
                let header_name: &String = event.data();

                state.new_header_name.set(header_name.clone());
            }
            "add_header__value_update" => {
                let header_value: &String = event.data();

                state.new_header_value.set(header_value.clone());
            }
            "add_header__submit" => {
                // TODO: Update to check if its a edit of existing header or new header being added

                let header_name = state.new_header_name.to_ref().to_string();
                let header_value = state.new_header_value.to_ref().to_string();

                state.floating_window.set(FloatingWindow::None);
                context.components.by_attribute("id", "app").focus();

                if header_name.trim().is_empty() || header_value.trim().is_empty() {
                    return;
                }

                let header = HeaderState {
                    name: header_name.into(),
                    value: header_value.into(),
                    row_color: "".to_string().into(),
                    row_fg_color: "".to_string().into(),
                };
                state.endpoint.to_mut().headers.push(header);
            }
            "add_header__cancel" => {
                state.floating_window.set(FloatingWindow::None);
                state.new_header_name.set("".to_string());
                state.new_header_value.set("".to_string());
                context.components.by_attribute("id", "app").focus();
            }

            _ => {}
        }
    }
}

impl Component for AddHeaderWindow {
    type State = AddHeaderWindowState;
    type Message = String;

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        state
            .cancel_button_color
            .set(state.button_color_unfocused.clone());

        state
            .success_button_color
            .set(state.button_color_unfocused.clone());
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        self.update_app_theme(state);

        state
            .cancel_button_color
            .set(state.success_color_focused.clone());

        state
            .success_button_color
            .set(state.cancel_color_focused.clone());
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match message.as_str() {
            "open" => {
                context
                    .components
                    .by_attribute("id", "header_name_input")
                    .focus();

                state.active = true;
            }

            component_messages => {
                let Ok(add_header_window_messages) =
                    serde_json::from_str::<AddHeaderWindowMessages>(component_messages)
                else {
                    return;
                };

                match add_header_window_messages {
                    AddHeaderWindowMessages::Specifically((
                        current_name,
                        header,
                        current_names,
                    )) => {
                        state.current_name = current_name;
                        state.current_names = current_names;
                        state.unique_name_error.set("".to_string());

                        state.header.set(NewHeader {
                            name: header.name.to_string().into(),
                            value: header.value.to_string().into(),
                        });

                        self.set_values_for_inputs(state, context);

                        state.active = true;
                    }
                }
            }
        }
    }

    fn on_event(
        &mut self,
        event: &mut component::UserEvent<'_>,
        state: &mut Self::State,
        _children: component::Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match event.name() {
            "header_name_update" => {
                let header_name = event.data::<String>();
                state.header.to_mut().name.set(header_name.clone());

                context.publish(
                    "add_header__name_update",
                    state.header.to_ref().name.to_ref().clone(),
                )
            }

            "header_value_update" => {
                let header_value = event.data::<String>();
                state.header.to_mut().value.set(header_value.clone());

                context.publish(
                    "add_header__value_update",
                    state.header.to_ref().value.to_ref().clone(),
                )
            }

            "name_input_focus" | "value_input_focus" => {
                context
                    .components
                    .by_attribute("id", "add_header_window")
                    .focus();
            }

            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        _elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match key.code {
            KeyCode::Esc => {
                context.publish("add_header__cancel", None::<()>);
            }

            KeyCode::Char(char) => {
                let mut context = RefCell::new(context);

                match char {
                    's' => self.submit(state, &mut context),

                    'c' => self.cancel(state, &mut context),

                    // Sets focus to header name text input
                    'n' => context
                        .borrow_mut()
                        .components
                        .by_attribute("id", "header_name_input")
                        .focus(),

                    // Sets focus to header value text input
                    'v' => context
                        .borrow_mut()
                        .components
                        .by_attribute("id", "header_value_input")
                        .focus(),

                    _ => {}
                }
            }

            _ => {}
        }
    }

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
    ) {
        // TODO: Remove this state.active after Anathema update
        if !state.active {
            return;
        }

        let mut context_ref = RefCell::new(context);

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "submit_button")
            .first(|_, _| {
                if mouse.left_up() {
                    self.submit(state, &mut context_ref);
                }
            });

        children
            .elements()
            .at_position(mouse.pos())
            .by_attribute("id", "cancel_button")
            .first(|_, _| {
                if mouse.left_up() {
                    self.cancel(state, &mut context_ref);
                }
            });
    }
}

#[derive(Default, Debug, State)]
pub struct NewHeader {
    pub name: Value<String>,
    pub value: Value<String>,
}
