//! Code snippet generation for API requests in multiple languages.

/// Represents a prepared API request for code generation.
#[derive(Debug, Clone)]
pub struct CodegenRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl CodegenRequest {
    /// Generate a curl command for this request.
    pub fn to_curl(&self) -> String {
        let mut cmd = format!("curl -X {} '{}'", self.method, self.url);

        for (name, value) in &self.headers {
            cmd.push_str(&format!(
                " \\\n  -H '{}: {}'",
                escape_shell(name),
                escape_shell(value)
            ));
        }

        if let Some(ref body) = self.body {
            cmd.push_str(&format!(" \\\n  -d '{}'", escape_shell(body)));
        }

        cmd
    }

    /// Generate a Python (requests library) snippet for this request.
    pub fn to_python(&self) -> String {
        let mut code = String::from("import requests\n\n");

        code.push_str(&format!("url = '{}'\n", self.url));

        if !self.headers.is_empty() {
            code.push_str("headers = {\n");
            for (name, value) in &self.headers {
                code.push_str(&format!(
                    "    '{}': '{}',\n",
                    name,
                    escape_python_string(value)
                ));
            }
            code.push_str("}\n\n");
        } else {
            code.push_str("headers = {}\n\n");
        }

        if let Some(ref body) = self.body {
            code.push_str(&format!("data = '{}'\n\n", escape_python_string(body)));
            code.push_str(&format!(
                "response = requests.request('{}', url, headers=headers, data=data)\n",
                self.method
            ));
        } else {
            code.push_str(&format!(
                "response = requests.request('{}', url, headers=headers)\n",
                self.method
            ));
        }

        code.push_str("print(response.status_code)\n");
        code.push_str("print(response.text)");

        code
    }

    /// Generate a JavaScript (fetch API) snippet for this request.
    pub fn to_javascript(&self) -> String {
        let mut code = String::from("const url = '");
        code.push_str(&self.url);
        code.push_str("';\n\n");

        let mut options = String::from("const options = {\n");
        options.push_str(&format!("  method: '{}',\n", self.method));

        if !self.headers.is_empty() {
            options.push_str("  headers: {\n");
            for (name, value) in &self.headers {
                options.push_str(&format!(
                    "    '{}': '{}',\n",
                    name,
                    escape_quoted_string(value)
                ));
            }
            options.push_str("  },\n");
        }

        if let Some(ref body) = self.body {
            options.push_str(&format!("  body: '{}',\n", escape_quoted_string(body)));
        }

        options.push_str("};\n\n");

        code.push_str(&options);
        code.push_str("fetch(url, options)\n");
        code.push_str("  .then(response => response.json())\n");
        code.push_str("  .then(data => console.log(data))\n");
        code.push_str("  .catch(error => console.error('Error:', error));");

        code
    }

    /// Generate a Rust (reqwest) snippet for this request.
    pub fn to_rust(&self) -> String {
        let mut code = String::from("#[tokio::main]\n");
        code.push_str("async fn main() -> Result<(), Box<dyn std::error::Error>> {\n");
        code.push_str("    let client = reqwest::Client::new();\n\n");

        code.push_str(&format!("    let url = \"{}\";\n\n", self.url));

        code.push_str(&format!(
            "    let mut request = client.request(reqwest::Method::{}, url);\n",
            self.method
        ));

        for (name, value) in &self.headers {
            code.push_str(&format!(
                "    request = request.header(\"{}\", \"{}\");\n",
                name,
                escape_quoted_string(value)
            ));
        }

        if let Some(ref body) = self.body {
            code.push_str(&format!(
                "    request = request.body(\"{}\");\n",
                escape_quoted_string(body)
            ));
        }

        code.push_str("\n    let response = request.send().await?;\n");
        code.push_str("    let status = response.status();\n");
        code.push_str("    let body = response.text().await?;\n\n");
        code.push_str("    println!(\"Status: {}\", status);\n");
        code.push_str("    println!(\"Body: {}\", body);\n\n");
        code.push_str("    Ok(())\n");
        code.push_str("}\n");

        code
    }
}

fn escape_shell(s: &str) -> String {
    s.replace('\'', "'\\''")
}

fn escape_python_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn escape_quoted_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request() -> CodegenRequest {
        CodegenRequest {
            method: "GET".into(),
            url: "https://api.example.com/users/42".into(),
            headers: vec![
                ("Authorization".into(), "Bearer token123".into()),
                ("X-Request-ID".into(), "req-001".into()),
            ],
            body: None,
        }
    }

    #[test]
    fn test_curl_generation() {
        let req = test_request();
        let curl = req.to_curl();
        assert!(curl.contains("curl -X GET"));
        assert!(curl.contains("https://api.example.com/users/42"));
        assert!(curl.contains("Authorization"));
        assert!(curl.contains("Bearer token123"));
    }

    #[test]
    fn test_curl_with_body() {
        let req = CodegenRequest {
            method: "POST".into(),
            url: "https://api.example.com/users".into(),
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: Some("{\"name\": \"Alice\"}".into()),
        };
        let curl = req.to_curl();
        assert!(curl.contains("curl -X POST"));
        assert!(curl.contains("-d"));
        assert!(curl.contains("name"));
    }

    #[test]
    fn test_python_generation() {
        let req = test_request();
        let python = req.to_python();
        assert!(python.contains("import requests"));
        assert!(python.contains("https://api.example.com/users/42"));
        assert!(python.contains("requests.request"));
        assert!(python.contains("'GET'"));
    }

    #[test]
    fn test_javascript_generation() {
        let req = test_request();
        let js = req.to_javascript();
        assert!(js.contains("const url = 'https://api.example.com/users/42'"));
        assert!(js.contains("fetch(url"));
        assert!(js.contains(".then(response => response.json())"));
    }

    #[test]
    fn test_rust_generation() {
        let req = test_request();
        let rust = req.to_rust();
        assert!(rust.contains("#[tokio::main]"));
        assert!(rust.contains("reqwest::Client::new()"));
        assert!(rust.contains("Bearer token123"));
    }

    #[test]
    fn test_escape_shell() {
        assert_eq!(escape_shell("hello"), "hello");
        assert_eq!(escape_shell("it's"), "it'\\''s");
    }

    #[test]
    fn test_escape_quoted_string() {
        assert_eq!(escape_quoted_string("hello"), "hello");
        assert_eq!(escape_quoted_string("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_quoted_string("quote\"here"), "quote\\\"here");
    }
}
