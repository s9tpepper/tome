use std::{fmt, fs, marker::PhantomData, str::FromStr, time::SystemTime};

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use uuid::Uuid;
use void::Void;

use crate::{
    fs::get_documents_dir,
    projects::{Header, PersistedEndpoint, PersistedProject, PersistedVariable, VariableType},
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

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct PostmanJson {
    info: PostmanInformation,
    item: Vec<PostmanItem>,
    variable: Option<Vec<PostmanVariable>>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
enum PostmanVariableType {
    #[default]
    #[serde(rename = "string")]
    String,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "number")]
    Number,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct PostmanVariable {
    id: Option<String>,
    key: Option<String>,
    value: Option<String>,
    r#type: Option<PostmanVariableType>,
    name: Option<String>,
    system: Option<bool>,
    disabled: Option<bool>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct PostmanInformation {
    name: String,
    description: String,
    schema: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct PostmanItem {
    id: Option<String>,
    name: String,
    request: PostmanRequest,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct PostmanRequestUrl {
    raw: String,
}

impl FromStr for PostmanRequestUrl {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PostmanRequestUrl { raw: s.to_string() })
    }
}

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct PostmanRequest {
    description: Option<String>,
    #[serde(deserialize_with = "string_or_struct")]
    url: PostmanRequestUrl,

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
struct PostmanFormData {
    key: String,
    value: String,
    r#type: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
enum PostmanBodyMode {
    #[serde(rename = "raw")]
    #[default]
    Raw,

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
struct GraphQL {
    query: String,
    variables: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct RawOptions {
    language: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct PostmanBodyOptions {
    raw: RawOptions,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
struct PostmanBody {
    options: Option<PostmanBodyOptions>,

    mode: PostmanBodyMode,

    #[serde(skip_serializing_if = "Option::is_none")]
    urlencoded: Option<Vec<PostmanKV>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    graphql: Option<GraphQL>,

    #[serde(skip_serializing_if = "Option::is_none")]
    formdata: Option<Vec<PostmanFormData>>,
    // file: Option<File>,
}

impl From<PostmanFormData> for String {
    fn from(formdata: PostmanFormData) -> Self {
        format!("{}=\"{}\"", formdata.key, formdata.value)
    }
}

impl From<PostmanKV> for String {
    fn from(postman_kv: PostmanKV) -> Self {
        format!("{}={}", postman_kv.key, postman_kv.value)
    }
}

impl From<GraphQL> for String {
    fn from(graphql: GraphQL) -> Self {
        match serde_json::to_string(&graphql) {
            Ok(json) => json,
            Err(_) => "".to_string(),
        }
    }
}

fn create_uuid(seed: &str) -> String {
    let uid = Uuid::new_v5(&Uuid::NAMESPACE_URL, seed.as_bytes());

    uid.to_string()
}

// TODO: Fix the cloning if possible in this from implementation
impl From<PostmanJson> for PersistedProject {
    fn from(postman_json: PostmanJson) -> Self {
        let PostmanJson { info, item, .. } = postman_json;
        let endpoints = item
            .iter()
            .map(|postman_item| {
                let (body_mode, body) = match &postman_item.request.body {
                    Some(postman_body) => match postman_body.mode {
                        PostmanBodyMode::Raw => {
                            let raw = postman_body.raw.clone();

                            ("raw", raw.unwrap_or_default())
                        }

                        PostmanBodyMode::UrlEncoded => match &postman_body.urlencoded {
                            Some(urlencoded) => (
                                "urlencoded",
                                urlencoded.iter().fold(String::new(), |mut acc, kv| {
                                    let encoded: String = (*kv).clone().into();
                                    acc.push_str(&encoded);
                                    acc
                                }),
                            ),
                            None => ("urlencoded", "".to_string()),
                        },

                        PostmanBodyMode::FormData => match &postman_body.formdata {
                            Some(formdata) => {
                                let fd: Vec<String> =
                                    formdata.iter().map(|pfd| (*pfd).clone().into()).collect();

                                ("formdata", fd.join("\n"))
                            }
                            None => ("formdata", "".to_string()),
                        },

                        PostmanBodyMode::GraphQL => match &postman_body.graphql {
                            Some(graphql) => ("graphql", (*graphql).clone().into()),
                            None => ("graphql", "".to_string()),
                        },

                        // PostmanBodyMode::File => BodyMode::Binary,
                        _ => ("raw", "".to_string()),
                    },

                    None => ("raw", "".to_string()),
                };

                let raw_type = match &postman_item.request.body {
                    // Some(body) => body.options.raw.language.to_string(),
                    Some(body) => match &body.options {
                        Some(options) => options.raw.language.to_string(),
                        None => "".to_string(),
                    },
                    None => "Text".to_string(),
                };

                PersistedEndpoint {
                    name: postman_item.name.clone(),
                    url: postman_item.request.url.raw.clone(),
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
                    body,
                    body_mode: body_mode.to_string(),
                    raw_type,
                }
            })
            .collect();

        let variable = match postman_json.variable {
            Some(postman_variables) => postman_variables
                .iter()
                .map(|postman_variable| PersistedVariable {
                    id: postman_variable.id.clone(),
                    key: postman_variable.key.clone(),
                    value: postman_variable.value.clone(),
                    r#type: match &postman_variable.r#type {
                        Some(pvt) => match pvt {
                            PostmanVariableType::String => Some(VariableType::String),
                            PostmanVariableType::Boolean => Some(VariableType::Boolean),
                            PostmanVariableType::Any => Some(VariableType::Any),
                            PostmanVariableType::Number => Some(VariableType::Number),
                        },
                        None => None,
                    },
                    name: postman_variable.name.clone(),
                    system: postman_variable.system,
                    disabled: postman_variable.disabled,
                })
                .collect(),
            None => vec![],
        };

        PersistedProject {
            name: info.name,
            endpoints,
            variable,
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
                            formdata: None,
                            options: Some(PostmanBodyOptions {
                                raw: RawOptions {
                                    language: "Text".to_string(),
                                },
                            }),
                        }),
                    },
                };

                let request = PostmanRequest {
                    url: PostmanRequestUrl {
                        raw: endpoint.url.clone(),
                    },
                    // TODO: Add descriptiong field/input in endpoint creation
                    description: Some("".to_string()),
                    method: endpoint.method.clone(),
                    header,
                    body,
                };

                PostmanItem {
                    id: Some(id),
                    request,
                    name: endpoint.name.clone(),
                }
            })
            .collect();

        let variable = project
            .variable
            .iter()
            .map(|var| {
                Some(PostmanVariable {
                    id: var.id.clone(),
                    key: var.key.clone(),
                    value: var.value.clone(),
                    r#type: match &var.r#type {
                        Some(variable_type) => match variable_type {
                            crate::projects::VariableType::String => {
                                Some(PostmanVariableType::String)
                            }
                            crate::projects::VariableType::Boolean => {
                                Some(PostmanVariableType::Boolean)
                            }
                            crate::projects::VariableType::Any => Some(PostmanVariableType::Any),
                            crate::projects::VariableType::Number => {
                                Some(PostmanVariableType::Number)
                            }
                        },
                        None => None,
                    },
                    name: var.name.clone(),
                    system: var.system,
                    disabled: var.disabled,
                })
            })
            .collect();

        PostmanJson {
            info,
            item,
            variable,
        }
    }
}
