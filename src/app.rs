use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anathema::{
    component::ComponentId,
    prelude::{Document, TuiBackend},
    runtime::{Runtime, RuntimeBuilder},
};

use crate::components::{
    add_header_window::{AddHeaderWindow, AddHeaderWindowState, ADD_HEADER_WINDOW_TEMPLATE},
    app_layout::{AppLayoutComponent, AppLayoutState, APP_LAYOUT_TEMPLATE},
    app_section::{AppSection, AppSectionState, APP_SECTION_TEMPLATE},
    confirm_action_window::ConfirmActionWindow,
    dashboard::DashboardComponent,
    edit_header_selector::{
        EditHeaderSelector, EditHeaderSelectorState, EDIT_HEADER_SELECTOR_TEMPLATE,
    },
    edit_header_window::EditHeaderWindow,
    edit_name_textinput::EditNameTextInput,
    edit_value_textinput::EditValueTextInput,
    focusable_section::{FocusableSection, FocusableSectionState},
    header_name_textinput::HeaderNameTextInput,
    header_value_textinput::HeaderValueTextInput,
    menu_item::{MenuItem, MenuItemState, MENU_ITEM_TEMPLATE},
    method_selector::{MethodSelector, MethodSelectorState, METHOD_SELECTOR_TEMPLATE},
    project_window::ProjectWindow,
    request_body_section::REQUEST_BODY_SECTION_TEMPLATE,
    request_headers_editor::{
        RequestHeadersEditor, RequestHeadersEditorState, REQUEST_HEADERS_EDITOR_TEMPLATE,
    },
    row::{Row, RowState, ROW_TEMPLATE},
    textarea::{TextArea, TextAreaInputState, TEXTAREA_TEMPLATE},
    textinput::{InputState, TextInput, TEXTINPUT_TEMPLATE},
};

pub fn app() -> anyhow::Result<()> {
    App::new().run()?;

    Ok(())
}

struct App {
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl App {
    pub fn new() -> Self {
        App {
            component_ids: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let doc = Document::new("@app");

        let tui = TuiBackend::builder()
            // .enable_alt_screen()
            .enable_raw_mode()
            .hide_cursor()
            .finish();

        if let Err(ref error) = tui {
            println!("[ERROR] Could not start terminal interface");
            println!("{error:?}");
        }

        let backend = tui.unwrap();
        let mut runtime_builder = Runtime::builder(doc, backend);
        self.register_components(&mut runtime_builder)?;

        let runtime = runtime_builder.finish();
        if let Ok(mut runtime) = runtime {
            let _emitter = runtime.emitter();

            runtime.run();
        } else if let Err(error) = runtime {
            println!("{:?}", error);
        }

        Ok(())
    }

    fn register_prototypes(&self, builder: &mut RuntimeBuilder<TuiBackend, ()>) {
        let _ = builder.register_prototype(
            "url_input",
            "./src/components/templates/url_input.aml",
            || FocusableSection,
            FocusableSectionState::new,
        );

        let _ = builder.register_prototype(
            "textinput",
            TEXTINPUT_TEMPLATE,
            || TextInput,
            InputState::new,
        );

        let _ = builder.register_prototype(
            "textarea",
            TEXTAREA_TEMPLATE,
            || TextArea,
            TextAreaInputState::new,
        );

        let _ = builder.register_prototype(
            "method_selector",
            METHOD_SELECTOR_TEMPLATE,
            || MethodSelector,
            MethodSelectorState::new,
        );

        let _ = builder.register_prototype(
            "menu_item",
            MENU_ITEM_TEMPLATE,
            || MenuItem,
            MenuItemState::new,
        );

        let _ = builder.register_prototype(
            "request_headers_editor",
            REQUEST_HEADERS_EDITOR_TEMPLATE,
            || RequestHeadersEditor,
            RequestHeadersEditorState::new,
        );

        let _ = builder.register_prototype(
            "app_section",
            APP_SECTION_TEMPLATE,
            || AppSection,
            AppSectionState::new,
        );

        let _ = builder.register_prototype(
            "request_body_section",
            REQUEST_BODY_SECTION_TEMPLATE,
            || FocusableSection,
            FocusableSectionState::new,
        );

        let _ = builder.register_prototype(
            "add_header_window",
            ADD_HEADER_WINDOW_TEMPLATE,
            || AddHeaderWindow,
            AddHeaderWindowState::new,
        );

        let _ = builder.register_prototype(
            "edit_header_selector",
            EDIT_HEADER_SELECTOR_TEMPLATE,
            || EditHeaderSelector,
            EditHeaderSelectorState::new,
        );

        let _ = builder.register_prototype("row", ROW_TEMPLATE, || Row, RowState::new);
    }

    fn register_components(
        &mut self,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        self.register_prototypes(builder);

        let _ = builder.register_component(
            "app",
            APP_LAYOUT_TEMPLATE,
            AppLayoutComponent,
            AppLayoutState {},
        );

        EditHeaderWindow::register(&self.component_ids, builder)?;
        HeaderNameTextInput::register(&self.component_ids, builder)?;
        HeaderValueTextInput::register(&self.component_ids, builder)?;
        EditNameTextInput::register(&self.component_ids, builder)?;
        EditValueTextInput::register(&self.component_ids, builder)?;
        ProjectWindow::register(&self.component_ids, builder)?;
        ConfirmActionWindow::register(&self.component_ids, builder)?;
        DashboardComponent::register(&self.component_ids, builder)?;

        Ok(())
    }
}
