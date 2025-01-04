use std::{fs, time::SystemTime};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    fs::get_documents_dir,
    projects::{save_project, Header, PersistedEndpoint, PersistedProject},
};

const POSTMAN_JSON_SCHEMA: &str =
    "https://schema.getpostman.com/json/collection/v2.1.0/collection.json";

pub fn export_postman(project: PersistedProject) -> anyhow::Result<()> {
    let postman_json: PostmanJson = project.into();

    let mut docs_dir = get_documents_dir()?;
    docs_dir.push(format!("{}.json", postman_json.info.name));

    let json = serde_json::to_string_pretty(&postman_json)?;

    fs::write(docs_dir, json)?;

    Ok(())
}

pub fn import_postman(postman: PostmanJson) -> anyhow::Result<()> {
    save_project(postman.into())
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct PostmanJson {
    info: PostmanInformation,
    item: Vec<PostmanItem>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct PostmanInformation {
    name: String,
    description: String,
    schema: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct PostmanItem {
    id: String,
    name: String,
    request: PostmanRequest,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct PostmanRequest {
    description: String,
    url: String,
    method: String,
    header: Vec<PostmanKV>,
    body: Option<PostmanBody>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct PostmanKV {
    key: String,
    value: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
enum PostmanBodyMode {
    #[serde(rename = "raw")]
    #[default]
    Raw,

    // #[serde(rename = "urlencoded", skip_serializing_if = "Option::is_none")]
    #[serde(rename = "urlencoded")]
    UrlEncoded,

    #[serde(rename = "formdata")]
    FormData,

    #[serde(rename = "file")]
    File,

    #[serde(rename = "graphql")]
    GraphQL,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct GraphQL;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct PostmanBody {
    mode: PostmanBodyMode,

    #[serde(skip_serializing_if = "Option::is_none")]
    urlencoded: Option<Vec<PostmanKV>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    graphql: Option<GraphQL>,
    // form_data: Option<FormData>,
    // file: Option<File>
}

fn create_uuid(seed: &str) -> String {
    let uid = Uuid::new_v5(&Uuid::NAMESPACE_URL, seed.as_bytes());

    uid.to_string()
}

// TODO: Update this when we add support for urlencoded, formdata, file, and graphql
impl From<PostmanBody> for String {
    fn from(postman_body: PostmanBody) -> Self {
        match postman_body.mode {
            PostmanBodyMode::Raw => postman_body.raw.unwrap_or("".to_string()),
            PostmanBodyMode::UrlEncoded => todo!(),
            PostmanBodyMode::FormData => todo!(),
            _ => "".to_string(), // PostmanBodyMode::File => todo!(),
                                 // PostmanBodyMode::GraphQL => todo!(),
        }
    }
}

impl From<PostmanJson> for PersistedProject {
    fn from(postman_json: PostmanJson) -> Self {
        let PostmanJson { info, item } = postman_json;
        let endpoints = item
            .iter()
            .map(|postman_item| PersistedEndpoint {
                name: postman_item.name.clone(),
                url: postman_item.request.url.clone(),
                method: postman_item.request.method.clone(),
                headers: postman_item
                    .request
                    .header
                    .iter()
                    .map(|postman_kv| Header {
                        name: postman_kv.key.clone(),
                        value: postman_kv.value.clone(),
                    })
                    .collect(),
                body: postman_item.request.body.clone().unwrap_or_default().into(),
            })
            .collect();

        PersistedProject {
            name: info.name,
            endpoints,
        }
    }
}

impl From<PersistedProject> for PostmanJson {
    fn from(project: PersistedProject) -> Self {
        let info = PostmanInformation {
            name: project.name,
            description: format!(
                "Postman collection exported from Tome on: {:?}",
                SystemTime::now()
            ),
            schema: POSTMAN_JSON_SCHEMA.to_string(),
        };

        let item: Vec<PostmanItem> = project
            .endpoints
            .iter()
            .map(|endpoint| {
                let id = create_uuid(&endpoint.name);

                let mut content_type = String::from("text/plain");

                let header: Vec<PostmanKV> = endpoint
                    .headers
                    .iter()
                    .map(|header| {
                        if header.name.to_lowercase() == "content-type" {
                            content_type = header.value.clone();
                        }

                        PostmanKV {
                            key: header.name.clone(),
                            value: header.value.clone(),
                        }
                    })
                    .collect();

                let body = match content_type.as_str() {
                    "multipart/x-form-data" => {
                        todo!()
                    }

                    "urlencoded" => {
                        todo!()
                    }

                    endpoint_body => match endpoint_body {
                        "" => None,
                        _ => Some(PostmanBody {
                            mode: PostmanBodyMode::Raw,
                            urlencoded: None,
                            raw: Some(endpoint.body.clone()),
                            graphql: None,
                        }),
                    },
                };

                let request = PostmanRequest {
                    url: endpoint.url.clone(),
                    // TODO: Add descriptiong field/input in endpoint creation
                    description: "".to_string(),
                    method: endpoint.method.clone(),
                    header,
                    body,
                };

                PostmanItem {
                    id,
                    request,
                    name: endpoint.name.clone(),
                }
            })
            .collect();

        PostmanJson { info, item }
    }
}
