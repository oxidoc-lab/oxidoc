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
        let request_body = extract_request_body(operation);
        let responses = extract_responses(operation);

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

fn extract_request_body(operation: &openapiv3::Operation) -> Option<ApiRequestBody> {
    let body_ref = operation.request_body.as_ref()?;
    let openapiv3::ReferenceOr::Item(body) = body_ref else {
        return None;
    };

    let (content_type, media) = body.content.iter().next()?;
    let schema_json = media
        .schema
        .as_ref()
        .map(|s| serde_json::to_string_pretty(s).unwrap_or_default())
        .unwrap_or_default();

    Some(ApiRequestBody {
        required: body.required,
        description: body.description.clone(),
        content_type: content_type.clone(),
        schema_json,
    })
}

fn extract_responses(operation: &openapiv3::Operation) -> Vec<ApiResponse> {
    let mut responses = Vec::new();

    if let Some(openapiv3::ReferenceOr::Item(resp)) = operation.responses.default.as_ref() {
        responses.push(response_from_item("default", resp));
    }

    for (status, resp_ref) in &operation.responses.responses {
        if let openapiv3::ReferenceOr::Item(resp) = resp_ref {
            responses.push(response_from_item(&status.to_string(), resp));
        }
    }

    responses
}

fn response_from_item(status: &str, resp: &openapiv3::Response) -> ApiResponse {
    let (content_type, schema_json) = resp
        .content
        .iter()
        .next()
        .map(|(ct, media)| {
            let schema = media
                .schema
                .as_ref()
                .map(|s| serde_json::to_string_pretty(s).unwrap_or_default());
            (Some(ct.clone()), schema)
        })
        .unwrap_or((None, None));

    ApiResponse {
        status: status.to_string(),
        description: resp.description.clone(),
        content_type,
        schema_json,
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
}
