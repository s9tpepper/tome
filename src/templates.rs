// #[cfg(feature = "runtime_templates")]
// #[allow(unused)]
// use anathema::prelude::ToSourceKind;

#[cfg(feature = "runtime_templates")]
pub fn template(name: &str) -> String {
    format!("src/components/{}.aml", name)
}

// macro_rules! template {
//     ($name:expr) => {
//         format!("src/components/{}.aml", $name)
//     };
// }

#[cfg(feature = "static_templates")]
use std::{collections::HashMap, sync::LazyLock};

#[cfg(feature = "static_templates")]
pub static TEMPLATE_MAP: LazyLock<HashMap<&str, &str>> =
    LazyLock::<HashMap<&str, &str>>::new(|| {
        let mut theme_map: HashMap<&str, &str> = HashMap::new();

        theme_map.insert(
            "templates/button",
            include_str!("components/templates/button.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/button_style_selector",
            include_str!("components/floating_windows/templates/button_style_selector.aml"),
        );

        theme_map.insert(
            "templates/request_body_section",
            include_str!("components/templates/request_body_section.aml"),
        );
        theme_map.insert(
            "floating_windows/templates/file_selector",
            include_str!("components/floating_windows/templates/file_selector.aml"),
        );
        theme_map.insert(
            "floating_windows/templates/project_variables",
            include_str!("components/floating_windows/templates/project_variables.aml"),
        );
        theme_map.insert(
            "floating_windows/templates/add_project_variable",
            include_str!("components/floating_windows/templates/add_project_variable.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/code_gen",
            include_str!("components/floating_windows/templates/code_gen.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/commands",
            include_str!("components/floating_windows/templates/commands.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/app_theme_selector",
            include_str!("components/floating_windows/templates/app_theme_selector.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/syntax_theme_selector",
            include_str!("components/floating_windows/templates/syntax_theme_selector.aml"),
        );

        theme_map.insert(
            "templates/options",
            include_str!("components/templates/options.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/edit_project_name",
            include_str!("components/floating_windows/templates/edit_project_name.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/edit_endpoint_name",
            include_str!("components/floating_windows/templates/edit_endpoint_name.aml"),
        );

        theme_map.insert(
            "templates/dashboard",
            include_str!("components/templates/dashboard.aml"),
        );

        theme_map.insert(
            "templates/confirm_action_window",
            include_str!("components/templates/confirm_action_window.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/endpoints_selector",
            include_str!("components/floating_windows/templates/endpoints_selector.aml"),
        );

        theme_map.insert(
            "templates/project_window",
            include_str!("components/templates/project_window.aml"),
        );

        theme_map.insert(
            "templates/app_layout",
            include_str!("components/templates/app_layout.aml"),
        );

        theme_map.insert(
            "templates/response_renderer",
            include_str!("components/templates/response_renderer.aml"),
        );

        theme_map.insert(
            "templates/syntax_highlighter_renderer",
            include_str!("components/templates/syntax_highlighter_renderer.aml"),
        );

        theme_map.insert(
            "templates/url_input",
            include_str!("components/templates/url_input.aml"),
        );

        theme_map.insert(
            "templates/add_header_window",
            include_str!("components/templates/add_header_window.aml"),
        );

        theme_map.insert(
            "templates/row",
            include_str!("components/templates/row.aml"),
        );

        theme_map.insert(
            "templates/edit_header_selector",
            include_str!("components/templates/edit_header_selector.aml"),
        );

        theme_map.insert(
            "templates/app_section",
            include_str!("components/templates/app_section.aml"),
        );

        theme_map.insert(
            "templates/request_headers_editor",
            include_str!("components/templates/request_headers_editor.aml"),
        );

        theme_map.insert(
            "templates/menu_item",
            include_str!("components/templates/menu_item.aml"),
        );

        theme_map.insert(
            "floating_windows/templates/body_mode_selector",
            include_str!("components/floating_windows/templates/body_mode_selector.aml"),
        );

        theme_map.insert(
            "templates/method_selector",
            include_str!("components/templates/method_selector.aml"),
        );

        theme_map.insert(
            "templates/textarea",
            include_str!("components/templates/textarea.aml"),
        );

        theme_map.insert(
            "templates/edit_input",
            include_str!("components/templates/edit_input.aml"),
        );

        theme_map.insert(
            "templates/response_filter_input",
            include_str!("components/templates/response_filter_input.aml"),
        );

        theme_map.insert(
            "templates/no_border_input",
            include_str!("components/templates/no_border_input.aml"),
        );

        theme_map.insert(
            "templates/textinput",
            include_str!("components/templates/textinput.aml"),
        );

        theme_map
    });

#[cfg(feature = "static_templates")]
use anathema::prelude::SourceKind;

#[cfg(feature = "static_templates")]
pub fn template(name: &str) -> SourceKind {
    use anathema::prelude::ToSourceKind;

    let tpl = TEMPLATE_MAP.get(name);

    let s = tpl.expect(&format!("Requested a missing template: {name}"));

    s.to_template()
}

// macro_rules! template {
//     ($name:expr) => {
//         let x = include_str!(concat!("components/", $name, ".aml"))
//
//         <x as ::anathema::prelude::ToSourceKind>::to_template()
//     };
// }

// pub(crate) use template;
