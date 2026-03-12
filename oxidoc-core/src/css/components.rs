pub const COMPONENTS: &str = r#"/* Component: Callout */
.oxidoc-callout {
    border: 1px solid var(--oxidoc-border);
    border-left: 3px solid var(--oxidoc-primary);
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    margin: 1rem 0;
    background: var(--oxidoc-bg-secondary);
}
.oxidoc-callout-warning { border-left-color: #f59e0b; }
.oxidoc-callout-error, .oxidoc-callout-danger { border-left-color: #ef4444; }
.oxidoc-callout-tip, .oxidoc-callout-success { border-left-color: #10b981; }
.oxidoc-callout-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
    font-size: 0.875rem;
}
.oxidoc-callout-body { margin-top: 0.5rem; font-size: 0.875rem; }

/* Component: Tabs */
.oxidoc-tabs {
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.5rem;
    margin: 1rem 0;
    overflow: hidden;
}
.oxidoc-tabs-list {
    display: flex;
    border-bottom: 1px solid var(--oxidoc-border);
    background: var(--oxidoc-bg-secondary);
    gap: 0;
}
.oxidoc-tab {
    padding: 0.5rem 1rem;
    border: none;
    background: none;
    color: var(--oxidoc-text-secondary);
    cursor: pointer;
    font-size: 0.875rem;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
}
.oxidoc-tab:hover { color: var(--oxidoc-text); }
.oxidoc-tab.active {
    color: var(--oxidoc-primary);
    border-bottom-color: var(--oxidoc-primary);
    font-weight: 500;
}
.oxidoc-tab-panel { padding: 1rem; }

/* Component: CodeBlock */
.oxidoc-codeblock {
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.5rem;
    margin: 1rem 0;
    overflow: hidden;
}
.oxidoc-codeblock-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.375rem 0.75rem;
    background: var(--oxidoc-bg-secondary);
    border-bottom: 1px solid var(--oxidoc-border);
    font-size: 0.75rem;
    color: var(--oxidoc-text-secondary);
}
.oxidoc-codeblock-body { position: relative; }
.oxidoc-codeblock-body pre {
    margin: 0;
    border: none;
    border-radius: 0;
    background: transparent;
}
.oxidoc-copy-btn {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.25rem;
    background: var(--oxidoc-bg);
    color: var(--oxidoc-text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s;
}
.oxidoc-codeblock:hover .oxidoc-copy-btn { opacity: 1; }
.oxidoc-copy-btn.copied { color: #10b981; }
.oxidoc-line {
    display: block;
    margin-left: -1rem;
    margin-right: -1rem;
    padding-left: 1rem;
    padding-right: 1rem;
}
.oxidoc-line.highlighted {
    background: color-mix(in srgb, var(--oxidoc-primary) 10%, transparent);
    border-left: 3px solid var(--oxidoc-primary);
    padding-left: calc(1rem - 3px);
}
.oxidoc-line-number {
    display: inline-block;
    width: 2.5rem;
    text-align: right;
    padding-right: 0.75rem;
    color: var(--oxidoc-text-secondary);
    user-select: none;
}

/* Component: Accordion */
.oxidoc-accordion { margin: 1rem 0; }
.oxidoc-accordion-item {
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.375rem;
    margin-bottom: 0.5rem;
    overflow: hidden;
}
.oxidoc-accordion-trigger {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 0.75rem 1rem;
    border: none;
    background: var(--oxidoc-bg-secondary);
    color: var(--oxidoc-text);
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    text-align: left;
}
.oxidoc-accordion-trigger:hover { background: var(--oxidoc-border); }
.oxidoc-accordion-chevron {
    font-size: 0.625rem;
    transition: transform 0.2s;
}
.oxidoc-accordion-content {
    padding: 0.75rem 1rem;
    font-size: 0.875rem;
}

/* Component: CardGrid */
.oxidoc-card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
    margin: 1rem 0;
}
.oxidoc-card {
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.5rem;
    padding: 1.25rem;
    text-decoration: none;
    color: var(--oxidoc-text);
    transition: border-color 0.15s, box-shadow 0.15s;
    display: flex;
    flex-direction: column;
}
a.oxidoc-card:hover {
    border-color: var(--oxidoc-primary);
    box-shadow: 0 2px 8px color-mix(in srgb, var(--oxidoc-primary) 15%, transparent);
}
.oxidoc-card-icon { font-size: 1.5rem; margin-bottom: 0.5rem; display: block; }
.oxidoc-card-title { font-weight: 600; margin: 0 0 0.25rem 0; font-size: 1rem; }
.oxidoc-card-body { font-size: 0.875rem; color: var(--oxidoc-text-secondary); }
.oxidoc-card-body p { margin: 0; }
.oxidoc-card-desc { font-size: 0.8125rem; color: var(--oxidoc-text-secondary); }"#;

pub const TYPOGRAPHY: &str = r#"/* Typography */
article h1 { font-size: 2rem; font-weight: 700; margin-bottom: 1rem; line-height: 1.2; }
article h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-top: 2.5rem;
    margin-bottom: 0.75rem;
    padding-bottom: 0.375rem;
    border-bottom: 1px solid var(--oxidoc-border);
    line-height: 1.3;
}
article h3 { font-size: 1.25rem; font-weight: 600; margin-top: 2rem; margin-bottom: 0.5rem; }
article h4 { font-size: 1.0625rem; font-weight: 600; margin-top: 1.5rem; margin-bottom: 0.5rem; }
article h5, article h6 { font-size: 1rem; font-weight: 600; margin-top: 1.25rem; margin-bottom: 0.5rem; }

article p { margin-bottom: 1rem; }

article a {
    color: var(--oxidoc-primary);
    text-decoration: underline;
    text-decoration-color: color-mix(in srgb, var(--oxidoc-primary) 40%, transparent);
    text-underline-offset: 2px;
    transition: text-decoration-color 0.15s;
}
article a:hover { text-decoration-color: var(--oxidoc-primary); }

article strong { font-weight: 600; }
article em { font-style: italic; }

article ul, article ol { padding-left: 1.5rem; margin-bottom: 1rem; }
article li { margin-bottom: 0.25rem; }
article li > p { margin-bottom: 0.5rem; }

article blockquote {
    border-left: 3px solid var(--oxidoc-primary);
    padding: 0.5rem 1rem;
    margin: 1rem 0;
    color: var(--oxidoc-text-secondary);
    background: var(--oxidoc-bg-secondary);
    border-radius: 0 0.375rem 0.375rem 0;
}

article code {
    font-family: var(--oxidoc-font-mono);
    font-size: 0.875em;
    background: var(--oxidoc-code-bg);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
}
article pre {
    background: var(--oxidoc-code-bg);
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.5rem;
    padding: 1rem;
    margin: 1rem 0;
    overflow-x: auto;
    font-size: 0.875rem;
    line-height: 1.6;
    position: relative;
}
article pre code {
    background: none;
    padding: 0;
    border-radius: 0;
}
article pre .oxidoc-copy-code {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.25rem;
    background: var(--oxidoc-bg);
    color: var(--oxidoc-text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s;
}
article pre:hover .oxidoc-copy-code { opacity: 1; }
article pre .oxidoc-copy-code.copied { color: #10b981; }

article table {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
    font-size: 0.875rem;
}
article th, article td {
    padding: 0.625rem 0.75rem;
    border: 1px solid var(--oxidoc-border);
    text-align: left;
}
article th {
    background: var(--oxidoc-bg-secondary);
    font-weight: 600;
}

article hr {
    border: none;
    border-top: 1px solid var(--oxidoc-border);
    margin: 2rem 0;
}

article img {
    max-width: 100%;
    height: auto;
    border-radius: 0.5rem;
}

/* Heading anchors */
article h1, article h2, article h3, article h4, article h5, article h6 {
    scroll-margin-top: calc(var(--oxidoc-header-height) + 1rem);
}"#;

pub const API: &str = r#"/* API endpoint styles */
.oxidoc-api-method {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    color: #fff;
    margin-right: 0.5rem;
    vertical-align: middle;
}
.oxidoc-api-method-get { background: #10b981; }
.oxidoc-api-method-post { background: #3b82f6; }
.oxidoc-api-method-put { background: #f59e0b; }
.oxidoc-api-method-patch { background: #8b5cf6; }
.oxidoc-api-method-delete { background: #ef4444; }"#;
