pub const SEARCH_DIALOG: &str = r#"/* Search dialog */
.oxidoc-search-overlay[hidden] {
    display: none !important;
}
.oxidoc-search-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 10vh;
    backdrop-filter: blur(2px);
}
.oxidoc-search-dialog {
    width: 100%;
    max-width: 600px;
    max-height: 70vh;
    background: var(--oxidoc-bg);
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.75rem;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
.oxidoc-search-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--oxidoc-border);
}
.oxidoc-search-icon {
    color: var(--oxidoc-primary);
    flex-shrink: 0;
}
.oxidoc-search-input {
    flex: 1;
    border: none;
    background: none;
    font-size: 1rem;
    color: var(--oxidoc-text);
    outline: none;
    font-family: inherit;
}
.oxidoc-search-input::placeholder {
    color: var(--oxidoc-text-secondary);
}
.oxidoc-search-clear {
    background: none;
    border: none;
    color: var(--oxidoc-primary);
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    white-space: nowrap;
}
.oxidoc-search-clear:hover {
    text-decoration: underline;
}
.oxidoc-search-close {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.25rem;
    color: var(--oxidoc-text-secondary);
    cursor: pointer;
    padding: 0.125rem;
    margin-left: 0.25rem;
}
.oxidoc-search-close:hover {
    color: var(--oxidoc-text);
    border-color: var(--oxidoc-text-secondary);
}
.oxidoc-search-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    min-height: 200px;
}
.oxidoc-search-empty,
.oxidoc-search-no-results {
    align-items: center;
    justify-content: center;
    height: 180px;
    color: var(--oxidoc-text-secondary);
    font-size: 0.9375rem;
}
.oxidoc-search-empty { display: flex; }
.oxidoc-search-no-results { display: none; }
.oxidoc-search-no-results.visible { display: flex; }
.oxidoc-search-empty.hidden { display: none; }
.oxidoc-search-results {
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.oxidoc-search-result {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.625rem 0.75rem;
    border-radius: 0.5rem;
    text-decoration: none;
    color: var(--oxidoc-text);
    transition: background 0.1s;
}
.oxidoc-search-result:hover,
.oxidoc-search-result.active {
    background: color-mix(in srgb, var(--oxidoc-primary) 10%, transparent);
}
.oxidoc-search-result-icon {
    color: var(--oxidoc-text-secondary);
    flex-shrink: 0;
    width: 18px;
    height: 18px;
    margin-top: 0.125rem;
}
.oxidoc-search-result-content {
    flex: 1;
    min-width: 0;
}
.oxidoc-search-result-title {
    font-size: 0.9375rem;
    font-weight: 500;
    line-height: 1.4;
}
.oxidoc-search-result-snippet {
    font-size: 0.8125rem;
    color: var(--oxidoc-text-secondary);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}
.oxidoc-search-result-page {
    font-size: 0.8125rem;
    color: var(--oxidoc-text-secondary);
    line-height: 1.4;
}
.oxidoc-search-result mark {
    background: color-mix(in srgb, var(--oxidoc-primary) 20%, transparent);
    color: var(--oxidoc-primary);
    border-radius: 2px;
    padding: 0 1px;
}
.oxidoc-search-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    border-top: 1px solid var(--oxidoc-border);
    font-size: 0.75rem;
    color: var(--oxidoc-text-secondary);
}
.oxidoc-search-keys {
    display: flex;
    gap: 1rem;
}
.oxidoc-search-keys kbd {
    display: inline-block;
    padding: 0 0.25rem;
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.25rem;
    font-family: inherit;
    font-size: 0.6875rem;
    line-height: 1.4;
    background: var(--oxidoc-bg-secondary);
}
@media (max-width: 640px) {
    .oxidoc-search-dialog {
        max-width: 100%;
        max-height: 100vh;
        border-radius: 0;
        height: 100%;
    }
    .oxidoc-search-overlay {
        padding-top: 0;
    }
    .oxidoc-search-footer {
        display: none;
    }
}"#;
