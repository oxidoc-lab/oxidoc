//! Type definitions for the API Playground component.

use serde::{Deserialize, Serialize};

/// Props for the API Playground component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPlaygroundProps {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub parameters: Vec<ParameterDef>,
    #[serde(default)]
    pub request_body_schema: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

/// Definition of an API parameter from the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    #[serde(rename = "type", default = "default_type")]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
}

fn default_type() -> String {
    "string".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_def_default_type() {
        let param = ParameterDef {
            name: "test".into(),
            location: "query".into(),
            param_type: default_type(),
            required: false,
        };
        assert_eq!(param.param_type, "string");
    }
}
