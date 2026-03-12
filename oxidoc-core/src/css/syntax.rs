/// Syntax highlighting token styles for code blocks.
/// Supports both light and dark color schemes via CSS variables.
pub const SYNTAX: &str = r##"/* Syntax Highlighting Tokens */
.tok-keyword {
    color: var(--oxidoc-tok-keyword, #2563eb);
    font-weight: 600;
}
.tok-string {
    color: var(--oxidoc-tok-string, #047857);
}
.tok-comment {
    color: var(--oxidoc-tok-comment, #6b7280);
    font-style: italic;
}
.tok-number {
    color: var(--oxidoc-tok-number, #b45309);
}
.tok-function {
    color: var(--oxidoc-tok-function, #6d28d9);
    font-weight: 500;
}
.tok-type {
    color: var(--oxidoc-tok-type, #9333ea);
    font-weight: 500;
}
.tok-operator {
    color: var(--oxidoc-tok-operator, #4338ca);
}
.tok-punctuation {
    color: var(--oxidoc-tok-punctuation, #64748b);
}
.tok-property {
    color: var(--oxidoc-tok-property, #0e7490);
}
.tok-variable {
    color: var(--oxidoc-tok-variable, #f3f4f6);
}
.tok-builtin {
    color: var(--oxidoc-tok-builtin, #1d4ed8);
    font-weight: 500;
}
.tok-attr {
    color: var(--oxidoc-tok-attr, #b45309);
}

@media (prefers-color-scheme: light) {
    :root {
        --oxidoc-tok-keyword: #2563eb;
        --oxidoc-tok-string: #047857;
        --oxidoc-tok-comment: #6b7280;
        --oxidoc-tok-number: #b45309;
        --oxidoc-tok-function: #6d28d9;
        --oxidoc-tok-type: #9333ea;
        --oxidoc-tok-operator: #4338ca;
        --oxidoc-tok-punctuation: #64748b;
        --oxidoc-tok-property: #0e7490;
        --oxidoc-tok-variable: #1e293b;
        --oxidoc-tok-builtin: #1d4ed8;
        --oxidoc-tok-attr: #b45309;
    }
}

@media (prefers-color-scheme: dark) {
    :root {
        --oxidoc-tok-keyword: #38bdf8;
        --oxidoc-tok-string: #34d399;
        --oxidoc-tok-comment: #9ca3af;
        --oxidoc-tok-number: #fbbf24;
        --oxidoc-tok-function: #c084fc;
        --oxidoc-tok-type: #f472b6;
        --oxidoc-tok-operator: #818cf8;
        --oxidoc-tok-punctuation: #cbd5e1;
        --oxidoc-tok-property: #22d3ee;
        --oxidoc-tok-variable: #f1f5f9;
        --oxidoc-tok-builtin: #60a5fa;
        --oxidoc-tok-attr: #fcd34d;
    }
}

html[data-theme="light"] {
    --oxidoc-tok-keyword: #0ea5e9;
    --oxidoc-tok-string: #10b981;
    --oxidoc-tok-comment: #6b7280;
    --oxidoc-tok-number: #d97706;
    --oxidoc-tok-function: #7c3aed;
    --oxidoc-tok-type: #c026d3;
    --oxidoc-tok-operator: #4f46e5;
    --oxidoc-tok-punctuation: #64748b;
    --oxidoc-tok-property: #0891b2;
    --oxidoc-tok-variable: #1e293b;
    --oxidoc-tok-builtin: #2563eb;
    --oxidoc-tok-attr: #d97706;
}

html[data-theme="dark"] {
    --oxidoc-tok-keyword: #38bdf8;
    --oxidoc-tok-string: #34d399;
    --oxidoc-tok-comment: #9ca3af;
    --oxidoc-tok-number: #fbbf24;
    --oxidoc-tok-function: #c084fc;
    --oxidoc-tok-type: #f472b6;
    --oxidoc-tok-operator: #818cf8;
    --oxidoc-tok-punctuation: #cbd5e1;
    --oxidoc-tok-property: #22d3ee;
    --oxidoc-tok-variable: #f1f5f9;
    --oxidoc-tok-builtin: #60a5fa;
    --oxidoc-tok-attr: #fcd34d;
}
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_contains_all_token_classes() {
        assert!(SYNTAX.contains(".tok-keyword"));
        assert!(SYNTAX.contains(".tok-string"));
        assert!(SYNTAX.contains(".tok-comment"));
        assert!(SYNTAX.contains(".tok-number"));
        assert!(SYNTAX.contains(".tok-function"));
        assert!(SYNTAX.contains(".tok-type"));
        assert!(SYNTAX.contains(".tok-operator"));
        assert!(SYNTAX.contains(".tok-punctuation"));
        assert!(SYNTAX.contains(".tok-property"));
        assert!(SYNTAX.contains(".tok-variable"));
        assert!(SYNTAX.contains(".tok-builtin"));
        assert!(SYNTAX.contains(".tok-attr"));
    }

    #[test]
    fn syntax_has_light_theme() {
        assert!(SYNTAX.contains("prefers-color-scheme: light"));
    }

    #[test]
    fn syntax_has_dark_theme() {
        assert!(SYNTAX.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn syntax_has_manual_theme_override() {
        assert!(SYNTAX.contains("html[data-theme=\"light\"]"));
        assert!(SYNTAX.contains("html[data-theme=\"dark\"]"));
    }
}
