//! Layout, theme, and structural CSS for the API Playground.

pub const CSS: &str = r#"
:host {
  --oxidoc-api-primary: #3b82f6;
  --oxidoc-api-success: #10b981;
  --oxidoc-api-warning: #f59e0b;
  --oxidoc-api-error: #ef4444;
  --oxidoc-api-info: #06b6d4;
  --oxidoc-api-patch: #a855f7;
  --oxidoc-api-delete: #ef4444;
  --oxidoc-api-bg: #ffffff;
  --oxidoc-api-text: #1f2937;
  --oxidoc-api-border: #e5e7eb;
  --oxidoc-api-shadow: rgba(0, 0, 0, 0.1);
}

@media (prefers-color-scheme: dark) {
  :host {
    --oxidoc-api-bg: #1f2937;
    --oxidoc-api-text: #f3f4f6;
    --oxidoc-api-border: #4b5563;
    --oxidoc-api-shadow: rgba(0, 0, 0, 0.3);
  }
}

.api-playground {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  color: var(--oxidoc-api-text);
  background: var(--oxidoc-api-bg);
  border: 1px solid var(--oxidoc-api-border);
  border-radius: 8px;
  padding: 24px;
  margin: 16px 0;
  box-shadow: 0 1px 3px var(--oxidoc-api-shadow);
}

.api-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--oxidoc-api-border);
}

.api-method {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: white;
}

.api-method-get { background-color: var(--oxidoc-api-primary); }
.api-method-post { background-color: var(--oxidoc-api-success); }
.api-method-put { background-color: var(--oxidoc-api-warning); }
.api-method-patch { background-color: var(--oxidoc-api-patch); }
.api-method-delete { background-color: var(--oxidoc-api-error); }
.api-method-options { background-color: var(--oxidoc-api-info); }

.api-path {
  font-family: "Monaco", "Courier New", monospace;
  font-size: 14px;
  font-weight: 500;
  word-break: break-all;
}

.api-section {
  margin-bottom: 24px;
}

.api-section-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
}

.api-section-title:hover { opacity: 0.8; }

.api-section-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  transition: transform 0.2s ease;
}

.api-section-toggle.collapsed { transform: rotate(-90deg); }

.api-param-group { display: grid; gap: 12px; }

.api-param-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.api-param-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  font-weight: 500;
}

.api-param-required {
  color: var(--oxidoc-api-error);
  font-weight: 600;
}

.api-param-type {
  font-size: 12px;
  color: var(--oxidoc-api-text);
  opacity: 0.7;
  font-family: "Monaco", "Courier New", monospace;
}

.api-response-section {
  border: 1px solid var(--oxidoc-api-border);
  border-radius: 4px;
  overflow: hidden;
}

.api-response-header {
  padding: 12px;
  background-color: rgba(0, 0, 0, 0.02);
  border-bottom: 1px solid var(--oxidoc-api-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.api-status {
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.api-status-2xx { color: var(--oxidoc-api-success); }
.api-status-3xx { color: var(--oxidoc-api-info); }
.api-status-4xx { color: var(--oxidoc-api-warning); }
.api-status-5xx { color: var(--oxidoc-api-error); }

.api-duration {
  font-size: 12px;
  opacity: 0.7;
}

.api-response-body {
  padding: 12px;
  background-color: var(--oxidoc-api-bg);
  max-height: 400px;
  overflow-y: auto;
  font-family: "Monaco", "Courier New", monospace;
  font-size: 12px;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.api-error-message {
  padding: 12px;
  background-color: rgba(239, 68, 68, 0.1);
  color: var(--oxidoc-api-error);
  border: 1px solid var(--oxidoc-api-error);
  border-radius: 4px;
  font-size: 14px;
}

.api-empty-state {
  padding: 24px;
  text-align: center;
  color: var(--oxidoc-api-text);
  opacity: 0.6;
}
"#;
