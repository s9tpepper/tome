use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use anathema::{
    component::{self, Component, ComponentId, KeyCode},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, State, Value},
    widgets::Elements,
};
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
}

#[derive(Default, State)]
pub struct AddHeaderWindowState {
    header: Value<NewHeader>,

    app_theme: Value<AppTheme>,

    #[state_ignore]
    current_names: Vec<String>,

    #[state_ignore]
    current_name: String,

    unique_name_error: Value<String>,
}

impl AddHeaderWindowState {
    pub fn new() -> Self {
        let app_theme = get_app_theme();

        AddHeaderWindowState {
            app_theme: app_theme.into(),

            current_name: "".to_string(),
            current_names: vec![],

            header: NewHeader::default().into(),
            unique_name_error: "".to_string().into(),
        }
    }
}

impl DashboardMessageHandler for AddHeaderWindow {
    fn handle_message(
        value: component::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::dashboard::DashboardState,
        mut context: anathema::prelude::Context<'_, super::dashboard::DashboardState>,
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

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.update_app_theme(state);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        #[allow(clippy::single_match)]
        match message.as_str() {
            "open" => {
                context.set_focus("id", "header_name_input");
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
        _elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
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
        key: component::KeyEvent,
        _state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            KeyCode::Esc => {
                context.publish("add_header__cancel", |state| &state.header);
            }

            KeyCode::Char(char) => {
                match char {
                    's' => context.publish("add_header__submit", |state| &state.header),

                    'c' => context.publish("add_header__cancel", |state| &state.header),

                    // Sets focus to header name text input
                    'n' => context.set_focus("id", "header_name_input"),

                    // Sets focus to header value text input
                    'v' => context.set_focus("id", "header_value_input"),

                    _ => {}
                }
            }

            _ => {}
        }
    }

    fn accept_focus(&self) -> bool {
        true
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

impl ::anathema::state::State for NewHeader {
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
