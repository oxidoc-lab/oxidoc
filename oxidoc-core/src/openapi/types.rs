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
    pub schema_json: String,
}

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: String,
    pub description: String,
    pub content_type: Option<String>,
    pub schema_json: Option<String>,
}
