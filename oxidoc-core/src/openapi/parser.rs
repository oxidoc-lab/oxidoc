use super::{ApiEndpoint, ApiParameter, ApiRequestBody, ApiResponse};
use crate::error::{OxidocError, Result};
use openapiv3::OpenAPI;
use std::path::Path;

/// Load and parse an OpenAPI spec file (YAML or JSON).
pub fn load_openapi_spec(spec_path: &Path) -> Result<OpenAPI> {
    let content = std::fs::read_to_string(spec_path).map_err(|e| OxidocError::FileRead {
        path: spec_path.display().to_string(),
        source: e,
    })?;

    let spec: OpenAPI = if spec_path.extension().and_then(|e| e.to_str()) == Some("json") {
        serde_json::from_str(&content).map_err(|e| OxidocError::RdxParse {
            path: spec_path.display().to_string(),
            message: format!("Invalid OpenAPI JSON: {e}"),
        })?
    } else {
        serde_saphyr::from_str(&content).map_err(|e| OxidocError::RdxParse {
            path: spec_path.display().to_string(),
            message: format!("Invalid OpenAPI YAML: {e}"),
        })?
    };

    Ok(spec)
}

/// Extract all endpoints from an OpenAPI spec.
pub fn extract_endpoints(spec: &OpenAPI) -> Vec<ApiEndpoint> {
    let mut endpoints = Vec::new();

    for (path, method, operation) in spec.operations() {
        let parameters = extract_parameters(operation);
        let request_body = extract_request_body(operation, spec);
        let responses = extract_responses(operation, spec);

        endpoints.push(ApiEndpoint {
            path: path.to_string(),
            method: method.to_uppercase(),
            operation_id: operation.operation_id.clone(),
            summary: operation.summary.clone(),
            description: operation.description.clone(),
            tags: operation.tags.clone(),
            parameters,
            request_body,
            responses,
            deprecated: operation.deprecated,
        });
    }

    endpoints
}

fn extract_parameters(operation: &openapiv3::Operation) -> Vec<ApiParameter> {
    operation
        .parameters
        .iter()
        .filter_map(|p| {
            let openapiv3::ReferenceOr::Item(param) = p else {
                return None;
            };
            let (name, location, required, description, data) = match param {
                openapiv3::Parameter::Query { parameter_data, .. } => (
                    &parameter_data.name,
                    "query",
                    parameter_data.required,
                    &parameter_data.description,
                    parameter_data,
                ),
                openapiv3::Parameter::Path { parameter_data, .. } => (
                    &parameter_data.name,
                    "path",
                    true,
                    &parameter_data.description,
                    parameter_data,
                ),
                openapiv3::Parameter::Header { parameter_data, .. } => (
                    &parameter_data.name,
                    "header",
                    parameter_data.required,
                    &parameter_data.description,
                    parameter_data,
                ),
                openapiv3::Parameter::Cookie { parameter_data, .. } => (
                    &parameter_data.name,
                    "cookie",
                    parameter_data.required,
                    &parameter_data.description,
                    parameter_data,
                ),
            };

            let schema_type = schema_type_from_data(data);

            Some(ApiParameter {
                name: name.clone(),
                location: location.to_string(),
                required,
                description: description.clone(),
                schema_type,
            })
        })
        .collect()
}

fn schema_type_from_data(data: &openapiv3::ParameterData) -> String {
    match &data.format {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => schema_type_string(schema_ref),
        openapiv3::ParameterSchemaOrContent::Content(_) => "object".to_string(),
    }
}

fn schema_type_string(schema_ref: &openapiv3::ReferenceOr<openapiv3::Schema>) -> String {
    match schema_ref {
        openapiv3::ReferenceOr::Reference { reference } => {
            reference.rsplit('/').next().unwrap_or("object").to_string()
        }
        openapiv3::ReferenceOr::Item(schema) => match &schema.schema_kind {
            openapiv3::SchemaKind::Type(t) => match t {
                openapiv3::Type::String(_) => "string".to_string(),
                openapiv3::Type::Number(_) => "number".to_string(),
                openapiv3::Type::Integer(_) => "integer".to_string(),
                openapiv3::Type::Boolean(_) => "boolean".to_string(),
                openapiv3::Type::Array(_) => "array".to_string(),
                openapiv3::Type::Object(_) => "object".to_string(),
            },
            _ => "object".to_string(),
        },
    }
}

fn extract_request_body(
    operation: &openapiv3::Operation,
    spec: &OpenAPI,
) -> Option<ApiRequestBody> {
    let body_ref = operation.request_body.as_ref()?;
    let openapiv3::ReferenceOr::Item(body) = body_ref else {
        return None;
    };

    let (content_type, media) = body.content.iter().next()?;
    let (schema_type, fields) = media
        .schema
        .as_ref()
        .map(|s| extract_schema_fields(s, spec))
        .unwrap_or_else(|| ("object".to_string(), vec![]));

    Some(ApiRequestBody {
        required: body.required,
        description: body.description.clone(),
        content_type: content_type.clone(),
        fields,
        schema_type,
    })
}

fn extract_responses(operation: &openapiv3::Operation, spec: &OpenAPI) -> Vec<ApiResponse> {
    let mut responses = Vec::new();

    if let Some(openapiv3::ReferenceOr::Item(resp)) = operation.responses.default.as_ref() {
        responses.push(response_from_item("default", resp, spec));
    }

    for (status, resp_ref) in &operation.responses.responses {
        if let openapiv3::ReferenceOr::Item(resp) = resp_ref {
            responses.push(response_from_item(&status.to_string(), resp, spec));
        }
    }

    responses
}

fn response_from_item(status: &str, resp: &openapiv3::Response, spec: &OpenAPI) -> ApiResponse {
    let (content_type, schema_type, fields) = resp
        .content
        .iter()
        .next()
        .map(|(ct, media)| {
            let (st, f) = media
                .schema
                .as_ref()
                .map(|s| extract_schema_fields(s, spec))
                .unwrap_or_else(|| ("object".to_string(), vec![]));
            (Some(ct.clone()), Some(st), f)
        })
        .unwrap_or((None, None, vec![]));

    ApiResponse {
        status: status.to_string(),
        description: resp.description.clone(),
        content_type,
        fields,
        schema_type,
    }
}

/// Resolve a `$ref` like `#/components/schemas/Post` to a Schema from the spec.
fn resolve_schema_ref<'a>(reference: &str, spec: &'a OpenAPI) -> Option<&'a openapiv3::Schema> {
    let name = reference.rsplit('/').next()?;
    let schema_ref = spec.components.as_ref()?.schemas.get(name)?;
    match schema_ref {
        openapiv3::ReferenceOr::Item(schema) => Some(schema),
        _ => None,
    }
}

/// Extract fields from a schema reference, returning (type_name, fields).
fn extract_schema_fields(
    schema_ref: &openapiv3::ReferenceOr<openapiv3::Schema>,
    spec: &OpenAPI,
) -> (String, Vec<super::SchemaField>) {
    match schema_ref {
        openapiv3::ReferenceOr::Reference { reference } => {
            let name = reference.rsplit('/').next().unwrap_or("object");
            // Resolve the $ref
            if let Some(schema) = resolve_schema_ref(reference, spec) {
                let (_, fields) = extract_schema_from_schema(schema, spec);
                (name.to_string(), fields)
            } else {
                (name.to_string(), vec![])
            }
        }
        openapiv3::ReferenceOr::Item(schema) => extract_schema_from_schema(schema, spec),
    }
}

fn extract_schema_fields_boxed(
    schema_ref: &openapiv3::ReferenceOr<Box<openapiv3::Schema>>,
    spec: &OpenAPI,
) -> (String, Vec<super::SchemaField>) {
    match schema_ref {
        openapiv3::ReferenceOr::Reference { reference } => {
            let name = reference.rsplit('/').next().unwrap_or("object");
            if let Some(schema) = resolve_schema_ref(reference, spec) {
                let (_, fields) = extract_schema_from_schema(schema, spec);
                (name.to_string(), fields)
            } else {
                (name.to_string(), vec![])
            }
        }
        openapiv3::ReferenceOr::Item(schema) => extract_schema_from_schema(schema, spec),
    }
}

fn extract_schema_from_schema(
    schema: &openapiv3::Schema,
    spec: &OpenAPI,
) -> (String, Vec<super::SchemaField>) {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(t) => match t {
            openapiv3::Type::Object(obj) => {
                let required_set: std::collections::HashSet<&str> =
                    obj.required.iter().map(|s| s.as_str()).collect();
                let fields = obj
                    .properties
                    .iter()
                    .map(|(name, prop_ref)| {
                        let (field_type, desc, children) = match prop_ref {
                            openapiv3::ReferenceOr::Reference { reference } => {
                                let t = reference.rsplit('/').next().unwrap_or("object");
                                let ch = resolve_schema_ref(reference, spec)
                                    .map(|s| extract_schema_from_schema(s, spec).1)
                                    .unwrap_or_default();
                                (t.to_string(), None, ch)
                            }
                            openapiv3::ReferenceOr::Item(prop) => {
                                let t = prop_type_string(prop);
                                let ch = match &prop.schema_kind {
                                    openapiv3::SchemaKind::Type(openapiv3::Type::Object(_)) => {
                                        extract_schema_from_schema(prop, spec).1
                                    }
                                    openapiv3::SchemaKind::Type(openapiv3::Type::Array(arr)) => arr
                                        .items
                                        .as_ref()
                                        .map(|i| extract_schema_fields_boxed(i, spec).1)
                                        .unwrap_or_default(),
                                    _ => vec![],
                                };
                                (t, prop.schema_data.description.clone(), ch)
                            }
                        };
                        super::SchemaField {
                            name: name.clone(),
                            field_type,
                            required: required_set.contains(name.as_str()),
                            description: desc,
                            children,
                        }
                    })
                    .collect();
                ("object".to_string(), fields)
            }
            openapiv3::Type::Array(arr) => {
                let (item_type, inner_fields) = arr
                    .items
                    .as_ref()
                    .map(|i| extract_schema_fields_boxed(i, spec))
                    .unwrap_or_else(|| ("object".to_string(), vec![]));
                (format!("array<{item_type}>"), inner_fields)
            }
            openapiv3::Type::String(_) => ("string".to_string(), vec![]),
            openapiv3::Type::Number(_) => ("number".to_string(), vec![]),
            openapiv3::Type::Integer(_) => ("integer".to_string(), vec![]),
            openapiv3::Type::Boolean(_) => ("boolean".to_string(), vec![]),
        },
        _ => ("object".to_string(), vec![]),
    }
}

fn prop_type_string(schema: &openapiv3::Schema) -> String {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(pt) => match pt {
            openapiv3::Type::String(_) => "string".to_string(),
            openapiv3::Type::Number(_) => "number".to_string(),
            openapiv3::Type::Integer(_) => "integer".to_string(),
            openapiv3::Type::Boolean(_) => "boolean".to_string(),
            openapiv3::Type::Array(_) => "array".to_string(),
            openapiv3::Type::Object(_) => "object".to_string(),
        },
        _ => "object".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi::test_helpers::sample_spec;

    #[test]
    fn extract_endpoints_from_spec() {
        let spec = sample_spec();
        let endpoints = extract_endpoints(&spec);
        assert_eq!(endpoints.len(), 3);

        let get_pets = endpoints
            .iter()
            .find(|e| e.operation_id.as_deref() == Some("listPets"))
            .unwrap();
        assert_eq!(get_pets.method, "GET");
        assert_eq!(get_pets.path, "/pets");
        assert_eq!(get_pets.parameters.len(), 1);
        assert_eq!(get_pets.parameters[0].name, "limit");
        assert_eq!(get_pets.parameters[0].location, "query");
        assert!(!get_pets.parameters[0].required);

        let create_pet = endpoints
            .iter()
            .find(|e| e.operation_id.as_deref() == Some("createPet"))
            .unwrap();
        assert_eq!(create_pet.method, "POST");
        assert!(create_pet.request_body.is_some());
        assert!(create_pet.request_body.as_ref().unwrap().required);
    }

    #[test]
    fn parse_empty_spec() {
        let yaml = r#"
openapi: "3.0.0"
info:
  title: Empty API
  version: "1.0.0"
paths: {}
"#;
        let spec: openapiv3::OpenAPI = serde_saphyr::from_str(yaml).unwrap();
        let endpoints = extract_endpoints(&spec);
        assert_eq!(endpoints.len(), 0);
    }

    #[test]
    fn parse_spec_with_references() {
        let yaml = r##"
openapi: "3.0.0"
info:
  title: API with Refs
  version: "1.0.0"
paths:
  /users:
    get:
      operationId: listUsers
      parameters:
        - $ref: "#/components/parameters/PageParam"
        - name: limit
          in: query
          required: false
          schema:
            type: integer
      responses:
        '200':
          description: Users
"##;
        let spec: openapiv3::OpenAPI = serde_saphyr::from_str(yaml).unwrap();
        let endpoints = extract_endpoints(&spec);
        assert_eq!(endpoints.len(), 1);
        let endpoint = &endpoints[0];
        // $ref parameters should be filtered out
        assert_eq!(endpoint.parameters.len(), 1);
        assert_eq!(endpoint.parameters[0].name, "limit");
    }

    #[test]
    fn deprecated_endpoint_flag() {
        let yaml = r#"
openapi: "3.0.0"
info:
  title: API with Deprecated
  version: "1.0.0"
paths:
  /old-endpoint:
    get:
      operationId: oldOp
      deprecated: true
      responses:
        '200':
          description: Response
"#;
        let spec: openapiv3::OpenAPI = serde_saphyr::from_str(yaml).unwrap();
        let endpoints = extract_endpoints(&spec);
        assert_eq!(endpoints.len(), 1);
        assert!(endpoints[0].deprecated);
    }

    #[test]
    fn endpoint_without_tags() {
        let yaml = r#"
openapi: "3.0.0"
info:
  title: API
  version: "1.0.0"
paths:
  /items:
    get:
      operationId: getItems
      responses:
        '200':
          description: Items
"#;
        let spec: openapiv3::OpenAPI = serde_saphyr::from_str(yaml).unwrap();
        let endpoints = extract_endpoints(&spec);
        assert_eq!(endpoints.len(), 1);
        assert!(endpoints[0].tags.is_empty());
    }

    #[test]
    fn multiple_response_codes() {
        let yaml = r#"
openapi: "3.0.0"
info:
  title: API
  version: "1.0.0"
paths:
  /resource:
    post:
      operationId: createResource
      responses:
        '201':
          description: Created
        '400':
          description: Bad request
        '500':
          description: Server error
"#;
        let spec: openapiv3::OpenAPI = serde_saphyr::from_str(yaml).unwrap();
        let endpoints = extract_endpoints(&spec);
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].responses.len(), 3);
        let statuses: Vec<_> = endpoints[0]
            .responses
            .iter()
            .map(|r| r.status.as_str())
            .collect();
        assert!(statuses.contains(&"201"));
        assert!(statuses.contains(&"400"));
        assert!(statuses.contains(&"500"));
    }
}
