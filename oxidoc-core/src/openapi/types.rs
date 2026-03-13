/// A parsed API endpoint with all its metadata.
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: String,
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub parameters: Vec<ApiParameter>,
    pub request_body: Option<ApiRequestBody>,
    pub responses: Vec<ApiResponse>,
    pub deprecated: bool,
}

#[derive(Debug, Clone)]
pub struct ApiParameter {
    pub name: String,
    pub location: String,
    pub required: bool,
    pub description: Option<String>,
    pub schema_type: String,
}

#[derive(Debug, Clone)]
pub struct ApiRequestBody {
    pub required: bool,
    pub description: Option<String>,
    pub content_type: String,
    pub fields: Vec<SchemaField>,
    pub schema_type: String,
}

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: String,
    pub description: String,
    pub content_type: Option<String>,
    pub fields: Vec<SchemaField>,
    pub schema_type: Option<String>,
}

/// A single field extracted from a JSON schema's `properties`.
#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: Option<String>,
    pub children: Vec<SchemaField>,
}
