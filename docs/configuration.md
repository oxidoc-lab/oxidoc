# Configuration Reference

Oxidoc is configured via an `oxidoc.toml` file at the root of your project. This guide documents every available configuration option with type, default value, and description.

## Project Configuration

### `[project]` (required)

| Field         | Type   | Default    | Description                                                                                     |
| :------------ | :----- | :--------- | :---------------------------------------------------------------------------------------------- |
| `name`        | string | (required) | The name of your documentation site. Displayed in headers and page titles.                      |
| `logo`        | string | `None`     | URL or path to your logo image. Used in the header navigation bar.                              |
| `base_url`    | string | `None`     | Base URL for your site (e.g., `https://docs.example.com`). Used for sitemap generation and SEO. |
| `description` | string | `None`     | Short description of your documentation. Used in the RSS feed and as fallback meta description. |

Example:

```toml
[project]
name = "My SDK Docs"
logo = "/assets/logo.svg"
base_url = "https://docs.example.com"
description = "Complete API documentation for My SDK"
```

## Theme Configuration

### `[theme]` (optional)

| Field        | Type   | Default     | Description                                                                               |
| :----------- | :----- | :---------- | :---------------------------------------------------------------------------------------- |
| `primary`    | string | `"#2563eb"` | Primary brand color (hex format). Used for buttons, links, and highlights.                |
| `accent`     | string | `"#f59e0b"` | Accent color (hex format). Used for secondary highlights.                                 |
| `dark_mode`  | string | `"system"`  | Dark mode behavior. Options: `"light"`, `"dark"`, or `"system"` (respects OS preference). |
| `custom_css` | array  | `[]`        | List of custom CSS file paths layered on top of the default styles.                       |
| `font`       | string | `None`      | Override the default sans-serif font family.                                              |
| `code_font`  | string | `None`      | Override the default monospace font family.                                               |

Example:

```toml
[theme]
primary = "#3b82f6"
accent = "#f59e0b"
dark_mode = "system"
custom_css = ["assets/custom.css"]
font = '"Inter", system-ui, sans-serif'
code_font = '"JetBrains Mono", monospace'
```

## Routing Configuration

### `[routing]` (optional)

Explicit sidebar ordering and navigation structure. If omitted, Oxidoc uses file-system routing (alphabetical or numbered file ordering).

| Field        | Type  | Default | Description                            |
| :----------- | :---- | :------ | :------------------------------------- |
| `navigation` | array | `[]`    | Array of navigation groups. See below. |

#### Navigation Groups

Each navigation group has:

| Field     | Type   | Default    | Description                                                                                               |
| :-------- | :----- | :--------- | :-------------------------------------------------------------------------------------------------------- |
| `group`   | string | (required) | Group heading displayed in the sidebar.                                                                   |
| `pages`   | array  | `[]`       | List of page slugs (without .rdx extension) to include in this group.                                     |
| `openapi` | string | `None`     | Path to an OpenAPI 3.x spec file (YAML or JSON). If set, the group contains generated API endpoint pages. |

Example:

```toml
[routing]
navigation = [
  { group = "Getting Started", pages = ["intro", "quickstart", "installation"] },
  { group = "Guides", pages = ["basics", "advanced", "best-practices"] },
  { group = "API Reference", openapi = "./openapi.yaml" }
]
```

## Versioning Configuration

### `[versioning]` (optional)

Support multiple documentation versions for different releases.

| Field      | Type   | Default | Description                                                  |
| :--------- | :----- | :------ | :----------------------------------------------------------- |
| `default`  | string | `None`  | Default version to display. Must be in the `versions` array. |
| `versions` | array  | `[]`    | List of available versions (e.g., `["v1.0", "v2.0"]`).       |

Example:

```toml
[versioning]
default = "v2.0"
versions = ["v1.0", "v1.5", "v2.0"]
```

## Search Configuration

### `[search]` (optional)

Configure the search backend.

| Field        | Type   | Default    | Description                                                                                            |
| :----------- | :----- | :--------- | :----------------------------------------------------------------------------------------------------- |
| `provider`   | string | `"oxidoc"` | Search provider. Options: `"oxidoc"`, `"algolia"`, `"typesense"`, `"meilisearch"`, or `"custom"`.      |
| `semantic`   | bool   | `false`    | Enable hybrid semantic search. Adds build time due to embedding generation but improves search recall. |
| `model_path` | string | `None`     | Path to a custom GGUF embedding model. Uses the bundled model if not set.                              |
| `app_id`     | string | `None`     | Algolia application ID (required for Algolia provider).                                                |
| `api_key`    | string | `None`     | Algolia search API key (required for Algolia provider).                                                |
| `index_name` | string | `None`     | Algolia index name (required for Algolia provider).                                                    |

Example:

```toml
[search]
provider = "oxidoc"
semantic = true
```

## Components Configuration

### `[components.custom]` (optional)

Register custom Vanilla Web Components (HTML5 Custom Elements) as an escape hatch for rapid UI iteration without Rust. Maps component tag names to JavaScript file paths.

Example:

```toml
[components.custom]
PromoBanner = "assets/js/promo-banner.js"
FeedbackWidget = "assets/js/feedback-widget.js"
Timeline = "assets/js/timeline.js"
```

When Oxidoc encounters a component tag matching an entry (e.g., `<PromoBanner>`), it bypasses the Wasm pipeline and renders the custom element directly with a `<script type="module">` tag pointing to the JS file.

## Footer Configuration

### `[footer]` (optional)

Configure the site footer.

| Field                 | Type   | Default | Description                                                        |
| :-------------------- | :----- | :------ | :----------------------------------------------------------------- |
| `copyright_owner`     | string | `None`  | Copyright owner name. Auto-generates "Copyright © {year} {owner}." |
| `copyright_owner_url` | string | `None`  | Optional URL for the copyright owner name.                         |
| `links`               | array  | `[]`    | Array of footer links (see below).                                 |

#### Footer Links

Each footer link has:

| Field   | Type   | Description                   |
| :------ | :----- | :---------------------------- |
| `label` | string | Link text displayed to users. |
| `href`  | string | URL target for the link.      |

Example:

```toml
[footer]
copyright_owner = "My Company"
copyright_owner_url = "https://example.com"

[[footer.links]]
label = "GitHub"
href = "https://github.com/mycompany/my-sdk"

[[footer.links]]
label = "Issues"
href = "https://github.com/mycompany/my-sdk/issues"
```

## Redirects Configuration

### `[redirects]` (optional)

Define URL redirects for moved or renamed pages.

Each redirect has:

| Field  | Type   | Description                                       |
| :----- | :----- | :------------------------------------------------ |
| `from` | string | Old URL path (e.g., `/old-page`).                 |
| `to`   | string | New URL path (e.g., `/new-page`) or external URL. |

Example:

```toml
[[redirects.redirects]]
from = "/old-api-reference"
to = "/api-reference"

[[redirects.redirects]]
from = "/legacy-guide"
to = "/guides/migration"
```

## Analytics Configuration

### `[analytics]` (optional)

Integrate analytics services.

| Field              | Type   | Default | Description                                                                     |
| :----------------- | :----- | :------ | :------------------------------------------------------------------------------ |
| `script`           | string | `None`  | Custom analytics script tag (e.g., Plausible, Fathom, or custom tracking code). |
| `google_analytics` | string | `None`  | Google Analytics measurement ID (e.g., `"G-XXXXXXXXXX"`).                       |

Example:

```toml
[analytics]
google_analytics = "G-XXXXXXXXXX"
script = "<script async src=\"https://cdn.example.com/analytics.js\"></script>"
```

## Complete Example

```toml
[project]
name = "Acme SDK Documentation"
logo = "/assets/acme-logo.svg"
base_url = "https://docs.acme.com"
description = "Complete API documentation and guides for the Acme SDK"

[theme]
primary = "#ff6b35"
dark_mode = "system"

[routing]
navigation = [
  { group = "Getting Started", pages = ["intro", "quickstart", "installation", "authentication"] },
  { group = "Guides", pages = ["basics", "advanced", "best-practices", "troubleshooting"] },
  { group = "API Reference", openapi = "./openapi.yaml" },
  { group = "Webhooks", pages = ["webhooks-intro", "webhook-events"] }
]

[versioning]
default = "v2.0"
versions = ["v1.0", "v1.5", "v2.0"]

[search]
provider = "oxidoc"

[footer]
copyright_owner = "Acme Inc."
copyright_owner_url = "https://acme.com"

[[footer.links]]
label = "GitHub Repository"
href = "https://github.com/acme/sdk"

[[footer.links]]
label = "Issue Tracker"
href = "https://github.com/acme/sdk/issues"

[[redirects.redirects]]
from = "/v1-docs"
to = "/"

[analytics]
google_analytics = "G-1234567890"
```

## Notes

- All configuration is optional except for `[project].name`.
- File-system routing (without explicit `[routing]` config) orders pages alphabetically or by numeric prefix (e.g., `1-intro.rdx`, `2-basics.rdx`).
- OpenAPI specs are automatically parsed and generate interactive API documentation pages.
- Analytics scripts are inserted into the document head; ensure they follow security best practices and comply with user privacy policies.
