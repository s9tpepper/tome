use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::{Children, Component, ComponentId, Context},
    runtime::Builder,
    state::{State, Value},
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::templates::template;

#[derive(Debug, Deserialize, Serialize)]
pub enum AppLayoutMessages {
    OpenOptions,
    OpenDashboard,
    OptionsMounted,
    DashboardMounted,
}

#[derive(Debug)]
enum AppDisplay {
    Dashboard,
    Options,
}

impl State for AppDisplay {
    fn type_info(&self) -> anathema::state::Type {
        anathema::state::Type::String
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            AppDisplay::Dashboard => Some("Dashboard"),
            AppDisplay::Options => Some("Options"),
        }
    }
}

#[derive(anathema::state::State)]
pub struct AppLayoutState {
    display: Value<AppDisplay>,

    #[state_ignore]
    dashboard_mounted: bool,

    #[state_ignore]
    options_mounted: bool,
}

pub struct AppLayoutComponent {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl AppLayoutComponent {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut Builder<()>,
    ) -> anyhow::Result<()> {
        let app_id = builder.component(
            "app",
            template("templates/app_layout"),
            AppLayoutComponent {
                component_ids: ids.clone(),
            },
            AppLayoutState {
                display: AppDisplay::Dashboard.into(),
                dashboard_mounted: false,
                options_mounted: false,
            },
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("app"), app_id);

        Ok(())
    }
}

impl Component for AppLayoutComponent {
    type State = AppLayoutState;
    type Message = String;

    fn on_focus(
        &mut self,
        _state: &mut Self::State,
        mut _elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        context.components.by_attribute("id", "app").focus();
    }

    fn on_unmount(
        &mut self,
        _: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        info!("----- Dashboard unmounted");
    }

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        let Ok(app_layout_message) = serde_json::from_str::<AppLayoutMessages>(&message) else {
            return;
        };

        match app_layout_message {
            AppLayoutMessages::OpenOptions => {
                state.display.set(AppDisplay::Options);

                if state.options_mounted {
                    context.components.by_attribute("id", "options").focus();
                }
            }
            AppLayoutMessages::OpenDashboard => {
                state.display.set(AppDisplay::Dashboard);

                if state.dashboard_mounted {
                    context.components.by_name("dashboard").focus();
                }
            }

            AppLayoutMessages::OptionsMounted => {
                info!("AppLayoutMessages::OptionsMounted");
                context.components.by_attribute("id", "options").focus();
            }

            AppLayoutMessages::DashboardMounted => {
                context.components.by_name("dashboard").focus();
            }
        }
    }
}
