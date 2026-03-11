//! Interactive control CSS for form inputs, buttons, code snippets, and responsive layout.

pub const CSS: &str = r#"
.api-input {
  padding: 8px 12px;
  font-size: 14px;
  border: 1px solid var(--oxidoc-api-border);
  border-radius: 4px;
  background-color: var(--oxidoc-api-bg);
  color: var(--oxidoc-api-text);
  font-family: "Monaco", "Courier New", monospace;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.api-input:focus {
  outline: none;
  border-color: var(--oxidoc-api-primary);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.api-textarea {
  padding: 12px;
  font-size: 13px;
  border: 1px solid var(--oxidoc-api-border);
  border-radius: 4px;
  background-color: var(--oxidoc-api-bg);
  color: var(--oxidoc-api-text);
  font-family: "Monaco", "Courier New", monospace;
  min-height: 150px;
  resize: vertical;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.api-textarea:focus {
  outline: none;
  border-color: var(--oxidoc-api-primary);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.api-select {
  padding: 8px 12px;
  font-size: 14px;
  border: 1px solid var(--oxidoc-api-border);
  border-radius: 4px;
  background-color: var(--oxidoc-api-bg);
  color: var(--oxidoc-api-text);
  cursor: pointer;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.api-select:focus {
  outline: none;
  border-color: var(--oxidoc-api-primary);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.api-button {
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 600;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s ease, opacity 0.2s ease;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.api-button-primary {
  background-color: var(--oxidoc-api-primary);
  color: white;
}

.api-button-primary:hover:not(:disabled) { opacity: 0.9; }
.api-button-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.api-button-secondary {
  background-color: transparent;
  color: var(--oxidoc-api-primary);
  border: 1px solid var(--oxidoc-api-primary);
}

.api-button-secondary:hover:not(:disabled) {
  background-color: rgba(59, 130, 246, 0.1);
}

.api-button-secondary:disabled { opacity: 0.5; cursor: not-allowed; }

.api-loading-spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(59, 130, 246, 0.2);
  border-top-color: var(--oxidoc-api-primary);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.api-code-snippet {
  border: 1px solid var(--oxidoc-api-border);
  border-radius: 4px;
  overflow: hidden;
}

.api-code-tabs {
  display: flex;
  border-bottom: 1px solid var(--oxidoc-api-border);
  background-color: rgba(0, 0, 0, 0.02);
}

.api-code-tab {
  padding: 12px;
  font-size: 13px;
  font-weight: 500;
  background: transparent;
  border: none;
  color: var(--oxidoc-api-text);
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 0.2s ease, background-color 0.2s ease;
}

.api-code-tab:hover {
  opacity: 1;
  background-color: rgba(59, 130, 246, 0.1);
}

.api-code-tab.active {
  opacity: 1;
  color: var(--oxidoc-api-primary);
  border-bottom: 2px solid var(--oxidoc-api-primary);
}

.api-code-block {
  position: relative;
  background-color: var(--oxidoc-api-bg);
  padding: 12px;
}

.api-code-content {
  font-family: "Monaco", "Courier New", monospace;
  font-size: 12px;
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
  max-height: 300px;
  overflow-y: auto;
}

.api-copy-button {
  position: absolute;
  top: 8px;
  right: 8px;
  padding: 6px 12px;
  font-size: 12px;
  background-color: var(--oxidoc-api-primary);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  opacity: 0.8;
  transition: opacity 0.2s ease;
}

.api-copy-button:hover { opacity: 1; }
.api-copy-button.copied { background-color: var(--oxidoc-api-success); }

.api-auth-inputs { display: grid; gap: 12px; }

@media (max-width: 640px) {
  .api-playground { padding: 16px; }

  .api-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .api-response-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .api-code-tabs { flex-wrap: wrap; }

  .api-code-tab {
    flex: 1 1 auto;
    text-align: center;
  }
}
"#;
