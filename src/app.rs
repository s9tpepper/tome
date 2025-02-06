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
        app_section::{AppSection, AppSectionState},
        confirm_action_window::ConfirmActionWindow,
        dashboard::DashboardComponent,
        edit_header_selector::EditHeaderSelector,
        edit_input::EditInput,
        floating_windows::{
            add_project_variable::AddProjectVariable,
            app_theme_selector::AppThemeSelector,
            body_mode_selector::{BodyModeSelector, BodyModeSelectorState},
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
        menu_item::{MenuItem, MenuItemState},
        method_selector::{MethodSelector, MethodSelectorState},
        options::OptionsView,
        project_window::ProjectWindow,
        request_headers_editor::{RequestHeadersEditor, RequestHeadersEditorState},
        response_renderer::ResponseRenderer,
        row::{Row, RowState},
        textarea::{TextArea, TextAreaInputState},
        textinput::{InputState, TextInput, TEXTINPUT_TEMPLATE},
    },
    templates::template,
    theme::get_app_theme,
};

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

        #[cfg(feature = "static_templates")]
        let tui = TuiBackend::builder()
            .enable_alt_screen()
            .enable_raw_mode()
            .enable_mouse()
            .hide_cursor()
            .finish();

        #[cfg(feature = "runtime_templates")]
        let tui = TuiBackend::builder()
            .enable_raw_mode()
            .enable_mouse()
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
            template("templates/textarea"),
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
            template("templates/method_selector"),
            || MethodSelector,
            MethodSelectorState::new,
        )?;

        builder.register_prototype(
            "body_mode_selector",
            template("floating_windows/templates/body_mode_selector"),
            || BodyModeSelector,
            BodyModeSelectorState::new,
        )?;

        builder.register_prototype(
            "menu_item",
            template("templates/menu_item"),
            || MenuItem,
            MenuItemState::new,
        )?;

        builder.register_prototype(
            "request_headers_editor",
            template("templates/request_headers_editor"),
            || RequestHeadersEditor,
            RequestHeadersEditorState::new,
        )?;

        builder.register_prototype(
            "app_section",
            template("templates/app_section"),
            || AppSection,
            AppSectionState::new,
        )?;

        builder.register_prototype("row", template("templates/row"), || Row, RowState::new)?;

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
            template("templates/url_input"),
        )?;

        TextArea::register(
            &self.component_ids,
            builder,
            "request_body_input".to_string(),
            template("templates/textarea"),
            Some("endpoint_request_body".to_string()),
            vec!["dashboard".to_string()],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "edit_endpoint_name_input",
            template("templates/edit_input"),
            None,
            vec![],
        )?;
        EditInput::register(
            &self.component_ids,
            builder,
            "edit_project_name_input",
            template("templates/edit_input"),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "add_project_variable_name",
            template("templates/edit_input"),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "add_project_variable_public_value",
            template("templates/edit_input"),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "add_project_variable_private_value",
            template("templates/edit_input"),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "response_filter_input",
            template("templates/response_filter_input"),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "url_text_input",
            template("templates/no_border_input"),
            Some(String::from("endpoint_url_input")),
            vec![String::from("dashboard")],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "edit_header_name_input",
            template("templates/textinput"),
            None,
            vec![],
        )?;
        EditInput::register(
            &self.component_ids,
            builder,
            "edit_header_value_input",
            template("templates/textinput"),
            None,
            vec![],
        )?;

        EditInput::register(
            &self.component_ids,
            builder,
            "headernameinput",
            template("templates/textinput"),
            None,
            vec![],
        )?;
        EditInput::register(
            &self.component_ids,
            builder,
            "headervalueinput",
            template("templates/textinput"),
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
        EditHeaderSelector::register(&self.component_ids, builder)?;

        TextArea::register(
            &self.component_ids,
            builder,
            "response_body_area".to_string(),
            template("templates/textarea"),
            None,
            vec![],
        )?;

        FocusableSection::register(
            &self.component_ids,
            builder,
            "request_body_section",
            template("templates/request_body_section"),
        )?;

        Ok(())
    }
}
