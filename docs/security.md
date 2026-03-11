# Security Guide

This guide documents recommended security practices for Oxidoc documentation sites, with a focus on Content-Security-Policy (CSP) headers for WebAssembly applications.

## Content-Security-Policy (CSP) Headers

Oxidoc generates static HTML with embedded WebAssembly islands. A properly configured CSP header protects against XSS attacks while allowing legitimate Wasm execution and component hydration.

### Recommended CSP Header

```
script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self' data:; default-src 'none'
```

### Header Breakdown

| Directive | Value | Reason |
|:---|:---|:---|
| `script-src` | `'self' 'wasm-unsafe-eval'` | Allows scripts from your origin and WebAssembly execution. `wasm-unsafe-eval` is required for Wasm to function. |
| `style-src` | `'self'` | Styles from your origin only. Prevents CSS injection. |
| `connect-src` | `'self'` | XHR/fetch requests to your origin only. Used for API calls from Wasm islands. |
| `img-src` | `'self' data:` | Images from your origin or data URIs. Prevents image-based tracking. |
| `default-src` | `'none'` | Deny everything not explicitly allowed. Fail-safe default. |

## Platform-Specific Configuration

### Vercel (vercel.json)

Add to your `vercel.json`:

```json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        {
          "key": "Content-Security-Policy",
          "value": "script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self' data:; default-src 'none'"
        }
      ]
    }
  ]
}
```

### Netlify (_headers)

Create a `_headers` file in your `dist/` directory:

```
/*
  Content-Security-Policy: script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self' data:; default-src 'none'
```

Or configure in `netlify.toml`:

```toml
[[headers]]
for = "/*"
[headers.values]
Content-Security-Policy = "script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self' data:; default-src 'none'"
```

### Cloudflare Pages (_headers)

Create a `_headers` file in your output directory:

```
/*
  Content-Security-Policy: script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self' data:; default-src 'none'
```

### Nginx

Add to your server block in `nginx.conf`:

```nginx
server {
  listen 443 ssl http2;
  server_name docs.example.com;

  # ... SSL configuration ...

  add_header Content-Security-Policy "script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self' data:; default-src 'none'" always;

  location / {
    root /path/to/oxidoc/dist;
    try_files $uri $uri/ /404.html;
  }
}
```

### Apache

Add to your `.htaccess` or VirtualHost:

```apache
<IfModule mod_headers.c>
  Header set Content-Security-Policy "script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self' data:; default-src 'none'"
</IfModule>
```

## Subresource Integrity (SRI)

Oxidoc automatically generates Subresource Integrity (SRI) hashes for CSS and JavaScript assets. These hashes are embedded in the HTML as `integrity` attributes:

```html
<link rel="stylesheet" href="/oxidoc.abc123.css" integrity="sha384-..." />
<script src="/oxidoc-loader.def456.js" integrity="sha384-..."></script>
```

SRI protects against unauthorized modifications of your assets. When combined with CSP, it provides defense-in-depth.

### Why CSP + SRI Work Together

- **CSP** prevents inline scripts and untrusted external resources
- **SRI** verifies the exact content of allowed resources
- Together, they ensure code integrity across your documentation site

## Extended CSP for Custom Features

If you use custom Web Components or third-party services, you may need to extend the CSP:

### With Google Analytics

```
script-src 'self' 'wasm-unsafe-eval' https://www.googletagmanager.com https://www.google-analytics.com;
connect-src 'self' https://www.google-analytics.com https://www.googletagmanager.com;
```

### With Custom Web Components

If you register custom Web Components that use external fonts or CDNs, add those origins:

```
script-src 'self' 'wasm-unsafe-eval' https://your-cdn.example.com;
style-src 'self' https://fonts.googleapis.com;
font-src 'self' https://fonts.gstatic.com;
connect-src 'self' https://api.example.com;
```

### With Search or Analytics APIs

If your Wasm islands make API calls to external services:

```
connect-src 'self' https://search-api.example.com https://analytics-api.example.com;
```

## Security Best Practices

1. **Always use HTTPS**: CSP headers are only effective over encrypted connections.

2. **Minimize exceptions**: Avoid `'unsafe-inline'` and `'unsafe-eval'` except for the Wasm requirement.

3. **Regularly audit content**: Monitor CSP violation reports via the `report-uri` or `report-to` directive (optional):
   ```
   report-uri https://your-csp-reporting-service.example.com/report
   ```

4. **Test before deploying**: Use CSP in report-only mode first:
   ```
   Content-Security-Policy-Report-Only: ...
   ```
   Monitor the `Content-Security-Policy-Report-Only` header in your server logs, then switch to the enforcement header when confident.

5. **Document custom components**: If you use `[components.custom]` with external JavaScript, document its CSP requirements.

6. **Keep dependencies updated**: Regularly update Oxidoc and Rust dependencies to receive security patches.

## Wasm-Specific Security Notes

- `'wasm-unsafe-eval'` is a necessary exception for WebAssembly execution. There is no safer alternative in current CSP specifications.
- Oxidoc pre-compiles Wasm binaries during the build phase. No code generation or dynamic evaluation occurs at runtime.
- Wasm modules are sandboxed by the browser and cannot directly access the DOM. All DOM interaction goes through explicitly exported JS functions.

## Vulnerability Reporting

If you discover a security vulnerability in Oxidoc, please report it responsibly to the maintainers rather than opening a public issue. Check the repository for security contact information.

## Additional Resources

- [Mozilla CSP Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [OWASP Content Security Policy](https://owasp.org/www-community/attacks/Content_Security_Policy)
- [WebAssembly Security Considerations](https://webassembly.org/docs/security/)
