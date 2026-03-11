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

| Field       | Type   | Default     | Description                                                                               |
| :---------- | :----- | :---------- | :---------------------------------------------------------------------------------------- |
| `primary`   | string | `"#2563eb"` | Primary brand color (hex format). Used for buttons, links, and highlights.                |
| `dark_mode` | string | `"system"`  | Dark mode behavior. Options: `"light"`, `"dark"`, or `"system"` (respects OS preference). |

Example:

```toml
[theme]
primary = "#3b82f6"
dark_mode = "system"
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

## Internationalization (i18n)

### `[i18n]` (optional)

Multi-language documentation support.

| Field            | Type   | Default | Description                                                           |
| :--------------- | :----- | :------ | :-------------------------------------------------------------------- |
| `default_locale` | string | `"en"`  | Default language locale (e.g., `"en"`, `"es"`, `"ja"`).               |
| `locales`        | array  | `[]`    | List of available locales. If empty, only the default locale is used. |

Example:

```toml
[i18n]
default_locale = "en"
locales = ["en", "es", "ja"]
```

## Search Configuration

### `[search]` (optional)

Configure the search backend.

| Field      | Type   | Default           | Description                                                                      |
| :--------- | :----- | :---------------- | :------------------------------------------------------------------------------- |
| `provider` | string | `"oxidoc-boostr"` | Search provider. Options: `"oxidoc-boostr"`, `"oxidoc-tantivy"`, or `"algolia"`. |

Example:

```toml
[search]
provider = "oxidoc-boostr"
```

## Components Configuration

### `[components.custom]` (optional)

Register custom Vanilla Web Components (HTML5 Custom Elements) as an escape hatch for rapid UI iteration without Rust.

Maps custom tag names to JavaScript file paths.

Example:

```toml
[components.custom]
PromoBanner = "assets/js/promo-banner.js"
FeedbackWidget = "assets/js/feedback-widget.js"
Timeline = "assets/js/timeline.js"
```

When Oxidoc encounters a component tag matching an entry (e.g., `<PromoBanner>`), it bypasses the Wasm pipeline and renders the custom element directly.

## Footer Configuration

### `[footer]` (optional)

Configure the site footer.

| Field       | Type   | Default | Description                               |
| :---------- | :----- | :------ | :---------------------------------------- |
| `copyright` | string | `None`  | Copyright notice displayed in the footer. |
| `links`     | array  | `[]`    | Array of footer links (see below).        |

#### Footer Links

Each footer link has:

| Field   | Type   | Description                   |
| :------ | :----- | :---------------------------- |
| `label` | string | Link text displayed to users. |
| `href`  | string | URL target for the link.      |

Example:

```toml
[footer]
copyright = "Copyright 2024 My Company. All rights reserved."

[[footer.links]]
label = "GitHub"
href = "https://github.com/mycompany/my-sdk"

[[footer.links]]
label = "Issues"
href = "https://github.com/mycompany/my-sdk/issues"

[[footer.links]]
label = "License"
href = "/LICENSE"
```

## Redirects Configuration

### `[redirects]` (optional)

Define URL redirects for moved or renamed pages.

| Field       | Type  | Description                          |
| :---------- | :---- | :----------------------------------- |
| `redirects` | array | Array of redirect rules (see below). |

#### Redirect Entries

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

[i18n]
default_locale = "en"
locales = ["en", "es", "fr", "ja"]

[search]
provider = "oxidoc-boostr"

[components.custom]
HighlightBox = "assets/js/highlight-box.js"
DemoPlayer = "assets/js/demo-player.js"

[footer]
copyright = "Copyright 2024 Acme Inc. All rights reserved."

[[footer.links]]
label = "GitHub Repository"
href = "https://github.com/acme/sdk"

[[footer.links]]
label = "Issue Tracker"
href = "https://github.com/acme/sdk/issues"

[[footer.links]]
label = "Community Forum"
href = "https://forum.acme.com"

[[redirects.redirects]]
from = "/v1-docs"
to = "/"

[[redirects.redirects]]
from = "/old-authentication"
to = "/authentication"

[analytics]
google_analytics = "G-1234567890"
script = "<script async src=\"https://cdn.example.com/custom-analytics.js\"></script>"
```

## Notes

- All configuration is optional except for `[project].name`.
- File-system routing (without explicit `[routing]` config) orders pages alphabetically or by numeric prefix (e.g., `1-intro.rdx`, `2-basics.rdx`).
- OpenAPI specs are automatically parsed and generate interactive API documentation pages.
- Custom Web Components specified in `[components.custom]` are loaded asynchronously and do not block page rendering.
- Analytics scripts are inserted into the document head; ensure they follow security best practices and comply with user privacy policies.
