use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use ::anathema::state::State;
use anathema::{
    component::{self, Component, ComponentId, KeyCode, KeyEvent, MouseEvent},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, Value},
    widgets::Elements,
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    app::GlobalEventHandler,
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
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
    ) -> anyhow::Result<()> {
        let name: String = "add_header_window".to_string();

        let app_id = builder.register_component(
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
        context: Context<'_, AddHeaderWindowState>,
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
        context: &mut RefCell<Context<'_, AddHeaderWindowState>>,
    ) {
        context
            .borrow_mut()
            .publish("add_header__submit", |state| &state.header);

        state.active = false;
    }

    pub fn cancel(
        &self,
        state: &mut AddHeaderWindowState,
        context: &mut RefCell<Context<'_, AddHeaderWindowState>>,
    ) {
        context
            .borrow_mut()
            .publish("add_header__cancel", |state| &state.header);

        state.active = false;
    }
}

#[derive(Default, State)]
pub struct AddHeaderWindowState {
    #[state_ignore]
    active: bool,

    header: Value<NewHeader>,

    app_theme: Value<AppTheme>,

    #[state_ignore]
    current_names: Vec<String>,

    #[state_ignore]
    current_name: String,

    unique_name_error: Value<String>,

    success_button_color: Value<String>,
    cancel_button_color: Value<String>,

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

        AddHeaderWindowState {
            active: false,
            app_theme: app_theme.into(),

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
        value: component::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::dashboard::DashboardState,
        mut context: Context<'_, super::dashboard::DashboardState>,
        _: Elements<'_, '_>,
        _component_ids: Ref<'_, HashMap<String, ComponentId<String>>>,
    ) {
        let event: String = ident.into();
        match event.as_str() {
            "add_header__name_update" => {
                let Ok(new_header) = serde_json::from_str::<Header>(&value.to_string()) else {
                    return;
                };

                state.new_header_name.set(new_header.name);
            }
            "add_header__value_update" => {
                let Ok(new_header) = serde_json::from_str::<Header>(&value.to_string()) else {
                    return;
                };

                state.new_header_value.set(new_header.value);
            }
            "add_header__submit" => {
                // TODO: Update to check if its a edit of existing header or new header being added

                let header_name = state.new_header_name.to_ref().to_string();
                let header_value = state.new_header_value.to_ref().to_string();

                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "app");

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
                context.set_focus("id", "app");
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
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
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
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);

        state
            .cancel_button_color
            .set(state.success_color_focused.clone());

        state
            .success_button_color
            .set(state.cancel_color_focused.clone());
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match message.as_str() {
            "open" => {
                context.set_focus("id", "header_name_input");

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
                            common: String::from(""),
                        });

                        self.set_values_for_inputs(state, context);
                    }
                }
            }
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        match ident {
            "header_name_update" => {
                state.header.to_mut().name.set(value.to_string());
                state.header.to_mut().update_common();

                context.publish("add_header__name_update", |state| &state.header)
            }

            "header_value_update" => {
                state.header.to_mut().value.set(value.to_string());
                state.header.to_mut().update_common();

                context.publish("add_header__value_update", |state| &state.header)
            }

            "name_input_focus" | "value_input_focus" => {
                context.set_focus("id", "add_header_window");
            }

            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        _elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        match key.code {
            KeyCode::Esc => {
                context.publish("add_header__cancel", |state| &state.header);
            }

            KeyCode::Char(char) => {
                let mut context = RefCell::new(context);

                match char {
                    's' => self.submit(state, &mut context),

                    'c' => self.cancel(state, &mut context),

                    // Sets focus to header name text input
                    'n' => context.borrow_mut().set_focus("id", "header_name_input"),

                    // Sets focus to header value text input
                    'v' => context.borrow_mut().set_focus("id", "header_value_input"),

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
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        // TODO: Remove this state.active after Anathema update
        if !state.active {
            return;
        }

        let mut context_ref = RefCell::new(context);

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "submit_button")
            .first(|_, _| {
                if mouse.lsb_up() {
                    self.submit(state, &mut context_ref);
                }
            });

        elements
            .at_position(mouse.pos())
            .by_attribute("id", "cancel_button")
            .first(|_, _| {
                if mouse.lsb_up() {
                    self.cancel(state, &mut context_ref);
                }
            });
    }
}

#[derive(Default, Debug)]
pub struct NewHeader {
    pub name: Value<String>,
    pub value: Value<String>,

    pub common: String,
}

impl NewHeader {
    pub fn update_common(&mut self) {
        let header = Header {
            name: self.name.to_ref().to_string(),
            value: self.value.to_ref().to_string(),
        };

        let Ok(common_val_str) = serde_json::to_string(&header) else {
            return;
        };

        self.common = common_val_str;
    }
}

impl State for NewHeader {
    fn state_get(
        &self,
        path: ::anathema::state::Path<'_>,
        sub: ::anathema::state::Subscriber,
    ) -> ::core::prelude::v1::Option<::anathema::state::ValueRef> {
        let ::anathema::state::Path::Key(key) = path else {
            return ::core::prelude::v1::None;
        };
        match key {
            "name" => ::core::prelude::v1::Some(self.name.value_ref(sub)),
            "value" => ::core::prelude::v1::Some(self.value.value_ref(sub)),
            _ => ::core::prelude::v1::None,
        }
    }

    fn state_lookup(
        &self,
        path: ::anathema::state::Path<'_>,
    ) -> ::core::prelude::v1::Option<::anathema::state::PendingValue> {
        let ::anathema::state::Path::Key(key) = path else {
            return ::core::prelude::v1::None;
        };
        match key {
            "name" => ::core::prelude::v1::Some(self.name.to_pending()),
            "value" => ::core::prelude::v1::Some(self.value.to_pending()),
            _ => ::core::prelude::v1::None,
        }
    }

    fn to_common(&self) -> ::core::prelude::v1::Option<::anathema::state::CommonVal<'_>> {
        Some(CommonVal::Str(&self.common))
    }
}
