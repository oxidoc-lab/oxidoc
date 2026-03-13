use axum::Router;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{Html, IntoResponse};
use futures_util::stream::Stream;
use notify::{Event, RecursiveMode, Watcher};
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

/// Start the development server with file watching and live reload.
pub async fn run_dev_server(project_root: PathBuf, port: u16) -> miette::Result<()> {
    let output_dir = project_root.join(".oxidoc-dev");

    // Build wasm once before first build
    super::build_wasm_once(&output_dir);

    // Initial build
    do_build(&project_root, &output_dir, "Build complete")?;

    let (reload_tx, _) = broadcast::channel::<()>(16);
    let reload_tx = Arc::new(reload_tx);

    // File watcher
    let watch_root = project_root.clone();
    let watch_output = output_dir.clone();
    let watch_tx = reload_tx.clone();
    let _watcher = spawn_watcher(watch_root, watch_output, watch_tx)?;

    // Axum server
    let reload_tx_sse = reload_tx.clone();

    let html_redirect = axum::middleware::from_fn(redirect_html_to_clean_url);

    let app = Router::new()
        .route(
            "/__oxidoc_reload",
            axum::routing::get(move || async move {
                let rx = reload_tx_sse.subscribe();
                Sse::new(reload_stream(rx)).keep_alive(KeepAlive::default())
            }),
        )
        .fallback_service(ServeDir::new(&output_dir).fallback(axum::routing::get({
            let output_dir_clone = output_dir.clone();
            move |uri: axum::http::Uri| {
                let output_dir = output_dir_clone.clone();
                async move {
                    let path = uri.path().trim_start_matches('/');

                    // Clean URLs: /intro → intro.html
                    let html_path = output_dir.join(format!("{path}.html"));
                    if html_path.is_file()
                        && let Ok(content) = tokio::fs::read_to_string(&html_path).await
                    {
                        return Html(content).into_response();
                    }

                    // Try path/index.html
                    let index_path = output_dir.join(path).join("index.html");
                    if index_path.is_file()
                        && let Ok(content) = tokio::fs::read_to_string(&index_path).await
                    {
                        return Html(content).into_response();
                    }

                    // 404
                    let not_found = output_dir.join("404.html");
                    let body = tokio::fs::read_to_string(&not_found)
                        .await
                        .unwrap_or_else(|_| "404 Not Found".to_string());
                    (axum::http::StatusCode::NOT_FOUND, Html(body)).into_response()
                }
            }
        })))
        .layer(html_redirect);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Dev server running at http://localhost:{port}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| miette::miette!("Failed to bind to port {port}: {e}"))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| miette::miette!("Server error: {e}"))?;

    Ok(())
}

/// Redirect `*.html` URLs to clean URLs (e.g. `/intro.html` → `/intro`).
async fn redirect_html_to_clean_url(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let path = req.uri().path();
    if path.ends_with(".html") && path != "/404.html" {
        let clean = path.trim_end_matches(".html");
        let clean = if clean.ends_with("/index") {
            clean.trim_end_matches("/index")
        } else {
            clean
        };
        let clean = if clean.is_empty() { "/" } else { clean };
        return axum::response::Redirect::permanent(clean).into_response();
    }
    next.run(req).await.into_response()
}

fn reload_stream(
    mut rx: broadcast::Receiver<()>,
) -> impl Stream<Item = Result<SseEvent, Infallible>> {
    async_stream::stream! {
        while let Ok(()) = rx.recv().await {
            yield Ok(SseEvent::default().event("reload").data(""));
        }
    }
}

fn do_build(project_root: &Path, output_dir: &Path, label: &str) -> miette::Result<()> {
    let start = std::time::Instant::now();
    let result = oxidoc_core::builder::build_site_with_model(
        project_root,
        output_dir,
        Some(super::BUNDLED_SEARCH_MODEL),
    )?;

    inject_reload_script(output_dir);

    tracing::info!(
        pages = result.pages_rendered,
        elapsed_ms = start.elapsed().as_millis() as u64,
        "{label}"
    );
    Ok(())
}

/// Inject a small live-reload polling script into all HTML files in the output directory.
fn inject_reload_script(output_dir: &Path) {
    let script = r#"<script>
(function(){var a=true,s=new EventSource("/__oxidoc_reload");s.addEventListener("reload",function(){if(a)location.reload()});document.addEventListener("visibilitychange",function(){if(document.hidden){a=false;s.close()}});window.addEventListener("pagehide",function(){a=false;s.close()})})();
</script>"#;

    let entries = match glob_html_files(output_dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Failed to scan for HTML files: {e}");
            return;
        }
    };
    for path in entries {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let injected = content.replace("</body>", &format!("{script}\n</body>"));
                if let Err(e) = std::fs::write(&path, injected) {
                    tracing::warn!(
                        "Failed to inject reload script into {}: {e}",
                        path.display()
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read {}: {e}", path.display());
            }
        }
    }
}

fn glob_html_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(glob_html_files(&path)?);
        } else if path.extension().and_then(|e| e.to_str()) == Some("html") {
            files.push(path);
        }
    }
    Ok(files)
}

fn spawn_watcher(
    project_root: PathBuf,
    output_dir: PathBuf,
    reload_tx: Arc<broadcast::Sender<()>>,
) -> miette::Result<notify::RecommendedWatcher> {
    let docs_dir = project_root.join("docs");
    let config_path = project_root.join("oxidoc.toml");

    let root_clone = project_root.clone();
    let last_rebuild = std::sync::Arc::new(std::sync::Mutex::new(std::time::Instant::now()));

    let mut watcher =
        notify::recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
            let Ok(event) = res else { return };

            // Filter: ignore hidden files and directories
            let has_hidden = event.paths.iter().any(|p| {
                p.components().any(|c| {
                    if let std::path::Component::Normal(n) = c {
                        n.to_string_lossy().starts_with('.')
                    } else {
                        false
                    }
                })
            });
            if has_hidden {
                return;
            }

            // Only rebuild on file modifications/creates/deletes in docs/ or oxidoc.toml
            let dominated = event
                .paths
                .iter()
                .any(|p| p.starts_with(&docs_dir) || p == &config_path);
            if !dominated {
                return;
            }

            let dominated = matches!(
                event.kind,
                notify::EventKind::Modify(_)
                    | notify::EventKind::Create(_)
                    | notify::EventKind::Remove(_)
            );
            if !dominated {
                return;
            }

            // Debounce: only rebuild if 100ms have passed since last rebuild
            let now = std::time::Instant::now();
            let mut last = last_rebuild.lock().unwrap_or_else(|e| {
                tracing::warn!("Rebuild mutex was poisoned, resetting");
                e.into_inner()
            });
            if now.duration_since(*last).as_millis() < 100 {
                return;
            }
            *last = now;

            let triggered_file = event
                .paths
                .first()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(unknown)".to_string());

            tracing::info!(file = %triggered_file, "File changed, rebuilding...");
            match do_build(&root_clone, &output_dir, "Rebuild complete") {
                Ok(_) => {
                    let _ = reload_tx.send(());
                }
                Err(e) => {
                    tracing::error!("Rebuild failed: {e}");
                }
            }
        })
        .map_err(|e| miette::miette!("Failed to create file watcher: {e}"))?;

    watcher
        .watch(&project_root, RecursiveMode::Recursive)
        .map_err(|e| miette::miette!("Failed to watch directory: {e}"))?;

    Ok(watcher)
}
