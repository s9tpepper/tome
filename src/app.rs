use std::{cell::RefCell, collections::HashMap, fs::File, rc::Rc};

use anathema::{
    component::ComponentId,
    prelude::{Document, TuiBackend},
    runtime::{Runtime, RuntimeBuilder},
};
use log::{info, LevelFilter};
use simplelog::{Config, WriteLogger};

use crate::components::{
    add_header_window::{AddHeaderWindow, AddHeaderWindowState, ADD_HEADER_WINDOW_TEMPLATE},
    app_layout::AppLayoutComponent,
    app_section::{AppSection, AppSectionState, APP_SECTION_TEMPLATE},
    confirm_action_window::ConfirmActionWindow,
    dashboard::DashboardComponent,
    edit_header_selector::{
        EditHeaderSelector, EditHeaderSelectorState, EDIT_HEADER_SELECTOR_TEMPLATE,
    },
    edit_header_window::EditHeaderWindow,
    edit_input::EditInput,
    edit_name_textinput::EditNameTextInput,
    edit_value_textinput::EditValueTextInput,
    floating_windows::{
        app_theme_selector::AppThemeSelector,
        body_mode_selector::{
            BodyModeSelector, BodyModeSelectorState, BODY_MODE_SELECTOR_TEMPLATE,
        },
        code_gen::CodeGen,
        commands::Commands,
        edit_endpoint_name::EditEndpointName,
        edit_project_name::EditProjectName,
        endpoints_selector::EndpointsSelector,
        file_selector::FileSelector,
        syntax_theme_selector::SyntaxThemeSelector,
    },
    focusable_section::FocusableSection,
    header_name_textinput::HeaderNameTextInput,
    header_value_textinput::HeaderValueTextInput,
    menu_item::{MenuItem, MenuItemState, MENU_ITEM_TEMPLATE},
    method_selector::{MethodSelector, MethodSelectorState, METHOD_SELECTOR_TEMPLATE},
    options::OptionsView,
    project_window::ProjectWindow,
    request_body_section::REQUEST_BODY_SECTION_TEMPLATE,
    request_headers_editor::{
        RequestHeadersEditor, RequestHeadersEditorState, REQUEST_HEADERS_EDITOR_TEMPLATE,
    },
    response_renderer::ResponseRenderer,
    row::{Row, RowState, ROW_TEMPLATE},
    textarea::{TextArea, TextAreaInputState, TEXTAREA_TEMPLATE},
    textinput::{InputState, TextInput, TEXTINPUT_TEMPLATE},
};

const RESPONSE_FILTER_INPUT: &str = "./src/components/templates/response_filter_input.aml";
const NO_BORDER_INPUT: &str = "./src/components/templates/no_border_input.aml";

pub fn app() -> anyhow::Result<()> {
    App::new().run()?;

    Ok(())
}

struct App {
    component_ids: Rc<RefCell<HashMap<String, ComponentId<String>>>>,
}

impl App {
    fn logger(&self) {
        // TODO: Move this log file into the application directory
        // TODO: Enable this block with an env var
        // let _ = WriteLogger::init(
        //     LevelFilter::Info,
        //     Config::default(),
        //     File::create("my_rust_binary.log").unwrap(),
        // );

        info!("Logging has been enabled");
    }

    pub fn new() -> Self {
        App {
            component_ids: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.logger();

        let doc = Document::new("@app");

        let tui = TuiBackend::builder()
            // TODO: Enable this line for releases
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

    fn register_prototypes(
        &self,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        let mut component_ids = self.component_ids.clone();

        builder.register_prototype(
            "textinput",
            TEXTINPUT_TEMPLATE,
            move || TextInput {
                component_ids: component_ids.clone(),
                listeners: vec!["dashboard".to_string()],
            },
            InputState::new,
        )?;

        component_ids = self.component_ids.clone();
        builder.register_prototype(
            "response_body_area",
            TEXTAREA_TEMPLATE,
            move || TextArea {
                component_ids: component_ids.clone(),
                listeners: vec![],
                input_for: None,
            },
            TextAreaInputState::new,
        )?;

        component_ids = self.component_ids.clone();
        builder.register_prototype(
            "textarea",
            TEXTAREA_TEMPLATE,
            move || TextArea {
                component_ids: component_ids.clone(),
                listeners: vec![],
                input_for: None,
            },
            TextAreaInputState::new,
        )?;

        builder.register_prototype(
            "method_selector",
            METHOD_SELECTOR_TEMPLATE,
            || MethodSelector,
            MethodSelectorState::new,
        )?;

        builder.register_prototype(
            "body_mode_selector",
            BODY_MODE_SELECTOR_TEMPLATE,
            || BodyModeSelector,
            BodyModeSelectorState::new,
        )?;

        builder.register_prototype(
            "menu_item",
            MENU_ITEM_TEMPLATE,
            || MenuItem,
            MenuItemState::new,
        )?;

        builder.register_prototype(
            "request_headers_editor",
            REQUEST_HEADERS_EDITOR_TEMPLATE,
            || RequestHeadersEditor,
            RequestHeadersEditorState::new,
        )?;

        builder.register_prototype(
            "app_section",
            APP_SECTION_TEMPLATE,
            || AppSection,
            AppSectionState::new,
        )?;

        builder.register_prototype(
            "add_header_window",
            ADD_HEADER_WINDOW_TEMPLATE,
            || AddHeaderWindow,
            AddHeaderWindowState::new,
        )?;

        builder.register_prototype(
            "edit_header_selector",
            EDIT_HEADER_SELECTOR_TEMPLATE,
            || EditHeaderSelector,
            EditHeaderSelectorState::new,
        )?;

        builder.register_prototype("row", ROW_TEMPLATE, || Row, RowState::new)?;

        Ok(())
    }

    fn register_components(
        &mut self,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        self.register_prototypes(builder)?;

        FocusableSection::register(
            &self.component_ids,
            builder,
            "url_input",
            "./src/components/templates/url_input.aml",
        )?;

        TextArea::register(
            &self.component_ids,
            builder,
            "request_body_input",
            Some(TEXTAREA_TEMPLATE),
            Some("endpoint_request_body".to_string()),
            vec!["dashboard".to_string()],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "edit_endpoint_name_input",
            None,
            None,
            vec![],
        )?;
        EditInput::register(
            &self.component_ids,
            builder,
            "edit_project_name_input",
            None,
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "response_filter_input",
            Some(RESPONSE_FILTER_INPUT),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "url_text_input",
            Some(NO_BORDER_INPUT),
            Some(String::from("endpoint_url_input")),
            vec![String::from("dashboard")],
        )?;

        ResponseRenderer::register(
            &self.component_ids,
            builder,
            "response_renderer".to_string(),
        )?;
        ResponseRenderer::register(
            &self.component_ids,
            builder,
            "code_sample_renderer".to_string(),
        )?;
        AppLayoutComponent::register(&self.component_ids, builder)?;
        EditHeaderWindow::register(&self.component_ids, builder)?;
        HeaderNameTextInput::register(&self.component_ids, builder)?;
        HeaderValueTextInput::register(&self.component_ids, builder)?;
        EditNameTextInput::register(&self.component_ids, builder)?;
        EditValueTextInput::register(&self.component_ids, builder)?;

        ProjectWindow::register(&self.component_ids, builder)?;
        EndpointsSelector::register(&self.component_ids, builder)?;

        ConfirmActionWindow::register(&self.component_ids, builder)?;
        DashboardComponent::register(&self.component_ids, builder)?;
        EditEndpointName::register(&self.component_ids, builder)?;
        EditProjectName::register(&self.component_ids, builder)?;
        OptionsView::register(&self.component_ids, builder)?;
        SyntaxThemeSelector::register(&self.component_ids, builder)?;
        AppThemeSelector::register(&self.component_ids, builder)?;
        Commands::register(&self.component_ids, builder)?;
        CodeGen::register(&self.component_ids, builder)?;
        FileSelector::register("postman_file_selector", &self.component_ids, builder)?;

        TextArea::register(
            &self.component_ids,
            builder,
            "response_body_area",
            None,
            None,
            vec![],
        )?;

        FocusableSection::register(
            &self.component_ids,
            builder,
            "request_body_section",
            REQUEST_BODY_SECTION_TEMPLATE,
        )?;

        // dbg!(&self.component_ids);

        Ok(())
    }
}
