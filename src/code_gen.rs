use std::fs;

use crate::{
    fs::{get_app_dir, get_documents_dir},
    projects::{Header, PersistedProject},
};

const JS_METHOD_TEMPLATE: &str = "
export async function [NAME]([BODY_VAR]) {
  const response = await fetch(\"[URL]\", {
    headers: { [HEADERS] },[BODY]
    method: \"[METHOD]\",
  })

  return await response.[RESPONSE_FUNC]()
}
";

const TS_METHOD_TEMPLATE: &str = "
export async function [NAME][RETURN_GENERIC]([BODY_VAR]): Promise<[RETURN_TYPE]> {
  const response = await fetch(\"[URL]\", {
    headers: { [HEADERS] },[BODY]
    method: \"[METHOD]\",
  })

  return await response.[RESPONSE_FUNC]() [RETURN_GENERIC_CAST]
}
";

const RUST_METHOD_TEMPLATE: &str = "
pub fn [NAME]([ARGS]) -> Result<String, Box<dyn Error>> {
    let mut request = ureq::request(\"[METHOD]\", \"[URL]\");

[HEADERS]

    [CALL]
}
";

pub enum WebType {
    JavaScript,
    TypeScript,
}

pub fn generate_rust(project: PersistedProject) -> anyhow::Result<()> {
    let mut module = "use ureq::Response;
use std::error::Error;
"
    .to_string();

    let rust_method_template = get_rust_method_template()?;

    project.endpoints.iter().for_each(|endpoint| {
        let mut headers: Vec<String> = vec![];
        endpoint.headers.iter().for_each(|header| {
            headers.push(format!(
                "    request = request.set(\"{}\", \"{}\");",
                header.name, header.value
            ));
        });
        let headers = headers.join("\n");

        let call = match endpoint.body.is_empty() {
            true => "request.call()",
            false => "request.send_string(body)",
        };

        let args = match endpoint.body.is_empty() {
            true => "",
            false => "body: &str",
        };

        let method_code = rust_method_template
            .replace("[NAME]", &endpoint.name.replace(" ", "_"))
            .replace("[ARGS]", args)
            .replace("[METHOD]", &endpoint.method)
            .replace("[URL]", &endpoint.url)
            .replace("[HEADERS]", &headers)
            .replace("[CALL]", call);

        module.push_str(&method_code);
    });

    write_code(&project.name, &module, "rs", &project.name)
}

pub fn generate_web(project: PersistedProject, web_type: WebType) -> anyhow::Result<()> {
    let mut module = String::new();
    let method_template = get_method_template(&web_type)?;

    project.endpoints.iter().for_each(|endpoint| {
        let content_type = endpoint
            .headers
            .iter()
            .find(|header| header.name == "content-type");

        let default_header = Header::default();
        let content_type = content_type.unwrap_or(&default_header);

        let mut return_generic = "";
        let mut return_type = "string";
        let mut response_func = "text";
        let mut return_generic_cast = "";

        if content_type.value == "application/json" {
            return_generic = "<T>";
            return_type = "T";
            response_func = "json";
            return_generic_cast = "as T";
        }

        let mut body_var = "";
        let mut body = "";

        if !endpoint.body.is_empty() {
            body_var = match web_type {
                WebType::JavaScript => "body",
                WebType::TypeScript => "body: BodyInit",
            };

            body = "\n    body,";
        }

        let mut headers: Vec<String> = vec![];
        endpoint.headers.iter().for_each(|h| {
            let header = format!("\"{}\": \"{}\"", h.name, h.value);
            headers.push(header);
        });
        let headers = headers.join(", ");

        let method_code = method_template
            .replace("[NAME]", &endpoint.name.replace(" ", "_"))
            .replace("[RETURN_GENERIC]", return_generic)
            .replace("[BODY_VAR]", body_var)
            .replace("[RETURN_TYPE]", return_type)
            .replace("[URL]", &endpoint.url)
            .replace("[HEADERS]", &headers)
            .replace("[BODY]", body)
            .replace("[METHOD]", &endpoint.method)
            .replace("[RESPONSE_FUNC]", response_func)
            .replace("[RETURN_GENERIC_CAST]", return_generic_cast);

        module.push_str(&method_code);
    });

    let extension = match web_type {
        WebType::JavaScript => "js",
        WebType::TypeScript => "ts",
    };

    write_code(&project.name, &module, extension, &project.name)
}

fn get_rust_method_template() -> anyhow::Result<String> {
    let mut app_dir = get_app_dir("code_templates")?;
    app_dir.push("rust_method_template.txt");

    match fs::read_to_string(app_dir.clone()) {
        Ok(template) => Ok(template),
        Err(_) => {
            fs::write(app_dir, RUST_METHOD_TEMPLATE)?;

            Ok(RUST_METHOD_TEMPLATE.to_string())
        }
    }
}

fn get_method_template(web_type: &WebType) -> anyhow::Result<String> {
    let mut app_dir = get_app_dir("code_templates")?;

    let template = match web_type {
        WebType::JavaScript => JS_METHOD_TEMPLATE,
        WebType::TypeScript => TS_METHOD_TEMPLATE,
    };

    let template_file_name = match web_type {
        WebType::JavaScript => "javascript_method_template.txt",
        WebType::TypeScript => "typescript_method_template.txt",
    };

    app_dir.push(template_file_name);

    match fs::read_to_string(app_dir.clone()) {
        Ok(template) => Ok(template),
        Err(_) => {
            fs::write(app_dir, template)?;

            Ok(template.to_string())
        }
    }
}

fn write_code(name: &str, module: &str, extension: &str, project: &str) -> anyhow::Result<()> {
    let mut docs_dir = get_documents_dir()?;
    let module_name = name.replace(" ", "_");

    docs_dir.push(project);
    let _ = fs::create_dir_all(&docs_dir);

    let docs_dir_exists = fs::exists(&docs_dir)?;

    if !docs_dir_exists {
        return Err(anyhow::Error::msg(
            "Couldn't create project directory for code generation",
        ));
    }

    docs_dir.push(format!("{module_name}.{extension}"));
    fs::write(docs_dir, module)?;

    Ok(())
}
