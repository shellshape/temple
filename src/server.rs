use anyhow::Result;
use axum::body::Body;
use axum::extract::{OriginalUri, State};
use axum::http::{Response, StatusCode};
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Sse};
use axum::routing::get;
use futures_util::stream::Stream;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::net::ToSocketAddrs;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

const INJECT_SCRIPT: &str = r#"
<script>
const eventSource = new EventSource("/_sse");
eventSource.onmessage = (event) => {
    if (event.data === "reload") {
        window.location.reload();
    }
}
</script>
"#;

#[derive(Clone)]
struct AppState {
    asset_dir: PathBuf,
    tx: Arc<broadcast::Sender<()>>,
}

pub async fn run_dev_server<P, A>(
    address: A,
    asset_dir: P,
    sender: broadcast::Sender<()>,
) -> Result<()>
where
    P: Into<PathBuf>,
    A: ToSocketAddrs,
{
    let listener = tokio::net::TcpListener::bind(address).await?;

    let tx = Arc::new(sender);

    let state = AppState {
        asset_dir: asset_dir.into(),
        tx: tx.clone(),
    };

    let app = axum::Router::new()
        .route("/_sse", get(sse_handler))
        .route("/", get(serve_handler))
        .route("/{*path}", get(serve_handler))
        .with_state(state);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream =
        BroadcastStream::new(rx).map(|_| std::result::Result::Ok(Event::default().data("reload")));
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("keep-alive"),
    )
}

async fn serve_handler(
    OriginalUri(uri): OriginalUri,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let path_str = uri.path();

    let mut file_path = PathBuf::from(state.asset_dir);
    file_path.push(&path_str.trim_start_matches('/'));

    if file_path.is_dir() {
        file_path.push("index.html");
    }

    match fs::read(&file_path).await {
        Ok(bytes) => {
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("html") {
                    let mut content = String::from_utf8_lossy(&bytes).to_string();
                    content.push_str(INJECT_SCRIPT);

                    return Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(Body::from(content))
                        .expect("response");
                }
            }

            let mime = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime)
                .body(Body::from(bytes))
                .expect("response")
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            format!("File not found: {}", file_path.display()),
        )
            .into_response(),
    }
}
