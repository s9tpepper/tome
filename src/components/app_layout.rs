use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::ComponentId,
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, State, Value},
    widgets::Elements,
};
use serde::{Deserialize, Serialize};

use crate::{app::GlobalEventHandler, templates::template};

#[derive(Debug, Deserialize, Serialize)]
pub enum AppLayoutMessages {
    OpenOptions,
    OpenDashboard,
}

enum AppDisplay {
    Dashboard,
    Options,
}

impl State for AppDisplay {
    fn to_common(&self) -> Option<anathema::state::CommonVal<'_>> {
        match self {
            AppDisplay::Dashboard => Some(CommonVal::Str("Dashboard")),
            AppDisplay::Options => Some(CommonVal::Str("Options")),
        }
    }
}

#[derive(anathema::state::State)]
pub struct AppLayoutState {
    display: Value<AppDisplay>,
}

pub struct AppLayoutComponent {
    #[allow(dead_code)]
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl AppLayoutComponent {
    pub fn register(
        ids: &Rc<RefCell<HashMap<String, ComponentId<String>>>>,
        builder: &mut RuntimeBuilder<TuiBackend, GlobalEventHandler>,
    ) -> anyhow::Result<()> {
        let app_id = builder.register_component(
            "app",
            template("templates/app_layout"),
            AppLayoutComponent {
                component_ids: ids.clone(),
            },
            AppLayoutState {
                display: AppDisplay::Dashboard.into(),
            },
        )?;

        let mut ids_ref = ids.borrow_mut();
        ids_ref.insert(String::from("app"), app_id);

        Ok(())
    }
}

impl anathema::component::Component for AppLayoutComponent {
    type State = AppLayoutState;
    type Message = String;

    fn on_focus(
        &mut self,
        _state: &mut Self::State,
        mut _elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        context.set_focus("id", "app");
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        let Ok(app_layout_message) = serde_json::from_str::<AppLayoutMessages>(&message) else {
            return;
        };

        match app_layout_message {
            AppLayoutMessages::OpenOptions => {
                state.display.set(AppDisplay::Options);
                context.set_focus("id", "options");
            }

            AppLayoutMessages::OpenDashboard => {
                state.display.set(AppDisplay::Dashboard);
                context.set_focus("id", "app");
            }
        }
    }
}
