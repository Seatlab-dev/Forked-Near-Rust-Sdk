//! Adapted from the rocket_okapi code.

use super::Method;
use okapi::{
    openapi3::{
        Components, MediaType, OpenApi, Operation, PathItem, RefOr, RequestBody, Response,
        Responses, Tag,
    },
    MapEntry,
};
use schemars::{
    gen::{SchemaGenerator, SchemaSettings},
    schema::SchemaObject,
    Map,
};
use serde_json::json;

pub trait OperationAdd {
    fn operation_add(gen: &mut OpenApiGenerator, tags: &[impl AsRef<str>]);
}
impl<T> OperationAdd for T
where
    T: Method,
    <T as Method>::Input: schemars::JsonSchema,
    <T as Method>::Output: schemars::JsonSchema,
{
    fn operation_add(gen: &mut OpenApiGenerator, tags: &[impl AsRef<str>]) {
        let op_id = T::NAME.to_string();
        let responses = gen
            .create_or_insert_json_schema::<T::Output>()
            .into_json_responses(T::RESPONSE_DESCRIPTION, T::NO_RETURN);

        let request_body =
            gen.create_or_insert_json_schema::<T::Input>().into_json_request_body(T::NO_ARGS);

        let summary = T::NAME.to_string();
        let description = T::DESCRIPTION.to_string();
        let tags = tags.iter().map(AsRef::as_ref).map(Into::into).collect();
        let method = T::NEAR_METHOD;

        gen.add_operation(OperationInfo {
            path: format!("/{}", T::NAME),
            method,
            operation: okapi::openapi3::Operation {
                operation_id: Some(op_id),
                responses,
                request_body: Some(request_body.into()),
                summary: Some(summary),
                description: Some(description),
                tags,
                ..Default::default()
            },
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NearMethod {
    /// Near changing calls.
    ///
    /// Represented as the http `POST` variant.
    Regular,
    /// Near view/non-changing invocations.
    ///
    /// Represented as the http `GET` variant.
    View,
    /// Near changing calls (contract init).
    ///
    /// Represented as the http `POST` variant.
    Init,
    /// Near changing calls (contract init (ignore state)).
    ///
    /// Represented as the http `POST` variant.
    InitIgnoreState,
}

pub type Path = String;

#[derive(Debug, Clone)]
pub struct OpenApiGenerator {
    // TODO: This tag should be removed in the future and settings should be used.
    // This `allow` is just added in the mean time to make sure alle other test
    // will be finished correctly.
    #[allow(dead_code)]
    pub settings: SchemaSettings,
    pub schema_generator: SchemaGenerator,
    pub operations: Map<Path, (NearMethod, Operation)>,
}

/// Contains information about an endpoint.
pub struct OperationInfo {
    /// The path of the endpoint
    pub path: Path,
    /// Whether this is a Call or a View endpoint.
    pub method: NearMethod,
    /// Contains information to be showed in the documentation about this endpoint.
    pub operation: okapi::openapi3::Operation,
}

impl OpenApiGenerator {
    #[must_use]
    pub fn new(settings: &SchemaSettings) -> Self {
        OpenApiGenerator {
            schema_generator: settings.clone().into_generator(),
            settings: settings.clone(),
            operations: Map::default(),
        }
    }

    /// Generates a new schema.
    ///
    /// For primitive types, that schema is simply returned.  
    /// For more structured types, that schema is stored in the generator, and a
    /// reference to it is returned.
    pub fn create_or_insert_json_schema<T>(&mut self) -> SchemaObject
    where
        T: schemars::JsonSchema,
    {
        self.schema_generator.subschema_for::<T>().into()
    }

    /// Add a new `HTTP Method` to the collection of endpoints in the
    /// `OpenApiGenerator`.
    pub fn add_operation(&mut self, mut op: OperationInfo) {
        if let Some(op_id) = op.operation.operation_id {
            // TODO do this outside add_operation
            op.operation.operation_id = Some(op_id.trim_start_matches(':').replace("::", "_"));
        }
        match self.operations.entry(op.path.clone()) {
            MapEntry::Occupied(_e) => {
                panic!("repeated path: {}", op.path);
            }
            MapEntry::Vacant(e) => {
                e.insert((op.method, op.operation));
            }
        };
    }

    pub fn into_openapi_with_tags(self, tags: &[impl AsRef<str>]) -> OpenApi {
        let (inputs, models): (Vec<_>, Vec<_>) = self
            .schema_generator
            .definitions()
            .iter()
            .map(|(name, _schema)| name)
            .cloned()
            .partition(|name| name.ends_with(".Input"));
        let schema_ref = |name: &String| {
            format!(
                "## {name}\n<SchemaDefinition schemaRef=\"#/components/schemas/{name}\" />\n\n",
                name = name
            )
        };

        let mut spec = self.into_openapi();

        let mut tags = tags
            .iter()
            .map(AsRef::as_ref)
            .map(|tag| Tag {
                name: tag.into(),
                extensions: [("x-displayName".to_string(), json!(tag.to_uppercase()))].into(),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        // add models
        tags.push(Tag {
            name: "_all_models".into(),
            description: Some(format!(
                "{models}\n# Inputs\n\n{inputs}",
                models = models.iter().map(schema_ref).collect::<String>(),
                inputs = inputs.iter().map(schema_ref).collect::<String>()
            )),
            external_docs: None,
            extensions: [("x-displayName".to_string(), json!("Models"))].into(),
        });

        spec.tags = tags;
        spec
    }

    /// Generate an `OpenApi` specification for all added operations.
    #[must_use]
    pub fn into_openapi(self) -> OpenApi {
        let mut schema_generator = self.schema_generator;
        let mut schemas = schema_generator.take_definitions();

        for visitor in schema_generator.visitors_mut() {
            for schema in schemas.values_mut() {
                visitor.visit_schema(schema)
            }
        }

        OpenApi {
            openapi: "3.1.0".to_owned(),
            paths: {
                let mut paths = Map::<Path, PathItem>::new();
                for (path, (method, op)) in self.operations {
                    let path_item = paths.entry(path.clone()).or_default();
                    assert!(path_item.operation_mut(method).replace(op).is_none());
                }
                paths
            },
            components: Some(Components {
                schemas: schemas.into_iter().map(|(k, v)| (k, v.into())).collect(),
                security_schemes: Map::new(),
                ..Default::default()
            }),
            ..OpenApi::default()
        }
    }
}

pub trait PathItemExt {
    fn operation_mut(&mut self, method: NearMethod) -> &mut Option<Operation>;
}
impl PathItemExt for PathItem {
    fn operation_mut(&mut self, method: NearMethod) -> &mut Option<Operation> {
        match method {
            NearMethod::Regular => &mut self.post,
            NearMethod::View => &mut self.get,
            NearMethod::Init => &mut self.post,
            NearMethod::InitIgnoreState => &mut self.post,
            // NearMethod::Call => &mut self.post,
            // NearMethod::View => &mut self.patch,
        }
    }
}

pub trait SchemaObjectExt {
    fn into_json_responses(self, description: impl AsRef<str>, no_return: bool) -> Responses;
    fn into_json_request_body(self, no_args: bool) -> RequestBody;
    fn accept_either_schema(s1: Option<Self>, s2: Option<Self>) -> Option<Self>
    where
        Self: Sized;
}
impl SchemaObjectExt for SchemaObject {
    fn into_json_responses(self, description: impl AsRef<str>, no_return: bool) -> Responses {
        let mut responses = Responses::default();

        let description = description.as_ref();

        if no_return {
            let status = 204_u16;
            let description = if description == "()" { "" } else { description };

            let _response = responses.responses.entry(status.to_string()).or_insert_with(|| {
                Response { description: description.into(), ..Default::default() }.into()
            });
            responses
        } else {
            let media = MediaType { schema: Some(self), ..MediaType::default() };

            let status = 200_u16;
            let content_type = "application/json";

            let response = responses.responses.entry(status.to_string()).or_insert_with(|| {
                Response { description: description.into(), ..Default::default() }.into()
            });
            let response = match response {
                RefOr::Ref(_) => panic!("Altering Ref responses is not supported."),
                RefOr::Object(o) => o,
            };

            response.merge_content_media(content_type, media);
            responses
        }
    }

    fn into_json_request_body(self, no_args: bool) -> RequestBody {
        if no_args {
            RequestBody::default()
        } else {
            RequestBody {
                content: {
                    let mut map = Map::new();
                    map.insert(
                        "application/json".to_owned(),
                        MediaType { schema: Some(self), ..MediaType::default() },
                    );
                    map
                },
                required: true,
                ..okapi::openapi3::RequestBody::default()
            }
        }
    }

    fn accept_either_schema(
        s1: Option<SchemaObject>,
        s2: Option<SchemaObject>,
    ) -> Option<SchemaObject> {
        let (s1, s2) = match (s1, s2) {
            (Some(s1), Some(s2)) => (s1, s2),
            (Some(s), None) | (None, Some(s)) => return Some(s),
            (None, None) => return None,
        };
        let mut schema = SchemaObject::default();
        schema.subschemas().any_of = Some(vec![s1.into(), s2.into()]);
        Some(schema)
    }
}

trait ResponseExt {
    fn merge_content_media(&mut self, content_type: &str, media: MediaType);
}
impl ResponseExt for Response {
    fn merge_content_media(&mut self, content_type: &str, media: MediaType) {
        use indexmap::map::Entry;
        match self.content.entry(content_type.to_string()) {
            Entry::Occupied(mut e) => {
                e.get_mut().merge_with(media);
            }
            Entry::Vacant(e) => {
                e.insert(media);
            }
        };
    }
}

pub trait MediaTypeExt {
    fn merge_with(&mut self, other: Self);
}

impl MediaTypeExt for MediaType {
    fn merge_with(&mut self, other: Self) {
        *self = MediaType {
            schema: SchemaObject::accept_either_schema(self.schema.clone(), other.schema),
            example: self.example.clone().or(other.example),
            examples: match (self.examples.clone(), other.examples) {
                (None, None) => None,
                (mt1e, mt2e) => Some(mt1e.into_iter().chain(mt2e).flatten().collect()),
            },
            encoding: self.encoding.clone().into_iter().chain(other.encoding).collect(),
            extensions: self.extensions.clone().into_iter().chain(other.extensions).collect(),
        };
    }
}
