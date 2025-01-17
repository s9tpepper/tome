use std::{cell::RefCell, collections::HashMap, fs::File, rc::Rc};

use anathema::{
    component::ComponentId,
    prelude::{Document, ToSourceKind, TuiBackend},
    runtime::{Runtime, RuntimeBuilder},
};
use log::{info, LevelFilter};
use simplelog::{Config, WriteLogger};

use crate::{
    components::{
        add_header_window::AddHeaderWindow,
        app_layout::AppLayoutComponent,
        app_section::{AppSection, AppSectionState, APP_SECTION_TEMPLATE},
        confirm_action_window::ConfirmActionWindow,
        dashboard::DashboardComponent,
        edit_header_selector::{
            EditHeaderSelector, EditHeaderSelectorState, EDIT_HEADER_SELECTOR_TEMPLATE,
        },
        edit_header_window::EditHeaderWindow,
        edit_input::EditInput,
        floating_windows::{
            add_project_variable::AddProjectVariable,
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
            project_variables::ProjectVariables,
            syntax_theme_selector::SyntaxThemeSelector,
        },
        focusable_section::FocusableSection,
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
    },
    theme::get_app_theme,
};

const RESPONSE_FILTER_INPUT: &str =
    include_str!("./components/templates/response_filter_input.aml");
const NO_BORDER_INPUT: &str = include_str!("./components/templates/no_border_input.aml");

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
        let _ = WriteLogger::init(
            LevelFilter::Info,
            Config::default(),
            File::create("my_rust_binary.log").unwrap(),
        );

        info!("Logging has been enabled");
    }

    pub fn new() -> Self {
        info!("App::new()");
        App {
            component_ids: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.logger();

        info!("App::run()");

        let doc = Document::new("@app");
        info!("App::run() doc");

        let tui = TuiBackend::builder()
            .enable_alt_screen()
            .enable_raw_mode()
            .hide_cursor()
            .finish();

        info!("Made tui");

        if let Err(ref error) = tui {
            info!("Error making tui");
            eprintln!("[ERROR] Could not start terminal interface");
            eprintln!("{error:?}");
        }

        let backend = tui.unwrap();
        let mut runtime_builder = Runtime::builder(doc, backend);

        info!("Registering components...");
        self.register_components(&mut runtime_builder)?;

        let runtime = runtime_builder.finish();
        info!("Started runtime...");

        if let Ok(mut runtime) = runtime {
            let _emitter = runtime.emitter();

            info!("Running runtime...");
            runtime.run();
        } else if let Err(error) = runtime {
            eprintln!("{:?}", error);
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
            TEXTINPUT_TEMPLATE.to_template(),
            move || TextInput {
                component_ids: component_ids.clone(),
                listeners: vec!["dashboard".to_string()],
            },
            InputState::new,
        )?;

        component_ids = self.component_ids.clone();
        builder.register_prototype(
            "response_body_area",
            TEXTAREA_TEMPLATE.to_template(),
            move || {
                let app_theme = get_app_theme();
                TextArea {
                    component_ids: component_ids.clone(),
                    listeners: vec![],
                    input_for: None,
                    app_theme,
                }
            },
            || {
                let app_theme = get_app_theme();
                let x = app_theme.foreground.to_ref().to_string();
                let y = app_theme.background.to_ref().to_string();
                let fg = x.as_str();
                let bg = y.as_str();

                TextAreaInputState::new(fg, bg)
            },
        )?;

        builder.register_prototype(
            "method_selector",
            METHOD_SELECTOR_TEMPLATE.to_template(),
            || MethodSelector,
            MethodSelectorState::new,
        )?;

        builder.register_prototype(
            "body_mode_selector",
            BODY_MODE_SELECTOR_TEMPLATE.to_template(),
            || BodyModeSelector,
            BodyModeSelectorState::new,
        )?;

        builder.register_prototype(
            "menu_item",
            MENU_ITEM_TEMPLATE.to_template(),
            || MenuItem,
            MenuItemState::new,
        )?;

        builder.register_prototype(
            "request_headers_editor",
            REQUEST_HEADERS_EDITOR_TEMPLATE.to_template(),
            || RequestHeadersEditor,
            RequestHeadersEditorState::new,
        )?;

        builder.register_prototype(
            "app_section",
            APP_SECTION_TEMPLATE.to_template(),
            || AppSection,
            AppSectionState::new,
        )?;

        builder.register_prototype(
            "edit_header_selector",
            EDIT_HEADER_SELECTOR_TEMPLATE.to_template(),
            || EditHeaderSelector,
            EditHeaderSelectorState::new,
        )?;

        builder.register_prototype("row", ROW_TEMPLATE.to_template(), || Row, RowState::new)?;

        Ok(())
    }

    fn register_components(
        &mut self,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
    ) -> anyhow::Result<()> {
        self.register_prototypes(builder)?;

        AddHeaderWindow::register(&self.component_ids, builder)?;

        FocusableSection::register(
            &self.component_ids,
            builder,
            "url_input",
            include_str!("./components/templates/url_input.aml").to_template(),
        )?;

        TextArea::register(
            &self.component_ids,
            builder,
            "request_body_input",
            Some(TEXTAREA_TEMPLATE.to_template()),
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
            "add_project_variable_name",
            None,
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "add_project_variable_public_value",
            None,
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "add_project_variable_private_value",
            None,
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "response_filter_input",
            Some(RESPONSE_FILTER_INPUT.to_template()),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "url_text_input",
            Some(NO_BORDER_INPUT.to_template()),
            Some(String::from("endpoint_url_input")),
            vec![String::from("dashboard")],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "edit_header_name_input",
            Some(TEXTINPUT_TEMPLATE.to_template()),
            None,
            vec![],
        )?;
        EditInput::register(
            &self.component_ids,
            builder,
            "edit_header_value_input",
            Some(TEXTINPUT_TEMPLATE.to_template()),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "headernameinput",
            Some(TEXTINPUT_TEMPLATE.to_template()),
            None,
            vec![],
        )?;
        EditInput::register(
            &self.component_ids,
            builder,
            "headervalueinput",
            Some(TEXTINPUT_TEMPLATE.to_template()),
            None,
            vec![],
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
        AddProjectVariable::register(&self.component_ids, builder)?;
        ProjectVariables::register(&self.component_ids, builder)?;
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
            REQUEST_BODY_SECTION_TEMPLATE.to_template(),
        )?;

        // dbg!(&self.component_ids);

        Ok(())
    }
}
