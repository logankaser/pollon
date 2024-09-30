use axum::{
    extract::{FromRequest, Path as UrlPath, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use axum_embed::ServeEmbed;
use dotenv::dotenv;
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde::Serialize;
use std::{
    env, fs,
    path::{Component as PathComponent, Path, PathBuf},
};
use tokio::io::AsyncWriteExt;

// Response Conversion:

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
struct ApiJson<T>(T);

impl<T> IntoResponse for ApiJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

enum ApiError {
    Simple(String),
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    FromUtf8(std::string::FromUtf8Error),
    Var(std::env::VarError),
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<std::str::Utf8Error> for ApiError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::Utf8(error)
    }
}
impl From<std::string::FromUtf8Error> for ApiError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8(error)
    }
}
impl From<std::env::VarError> for ApiError {
    fn from(error: std::env::VarError) -> Self {
        Self::Var(error)
    }
}

// Custom Error formating
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let (status, message) = match self {
            ApiError::Simple(error) => (StatusCode::INTERNAL_SERVER_ERROR, error),
            ApiError::Io(error) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)),
            ApiError::Utf8(error) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)),
            ApiError::FromUtf8(error) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)),
            ApiError::Var(error) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)),
        };
        (status, ApiJson(ErrorResponse { message })).into_response()
    }
}

#[derive(Clone)]
struct AppState {
    library: PathBuf,
}

impl AppState {
    fn new() -> Self {
        let library_raw = env::var("LIBRARY").unwrap_or(env::var("PWD").unwrap());
        AppState {
            library: fs::canonicalize(library_raw).unwrap_or(".".into()),
        }
    }
}

#[derive(RustEmbed, Clone)]
#[folder = "client/"]
struct ClientAssets;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let state = AppState::new();
    println!("Serving {:?}", state.library);
    let api = Router::new()
        .route("/:document", get(document).post(document_append))
        .route(
            "/:document/:node",
            get(node_get).put(node_replace).delete(node_delete),
        )
        .with_state(state);
    let client_assets = ServeEmbed::<ClientAssets>::new();
    let app = Router::new()
        .nest("/", api)
        .nest_service("/client", client_assets);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// Append to a path if new_component is
/// a normal component, so no .. or . or ../../
fn path_append_normal<'a>(
    path: &'a mut PathBuf,
    new_component: &str,
) -> Result<&'a PathBuf, ApiError> {
    // We're going to pass this path to an OS API,
    // from a user input, so lets do some sanitization.
    // TODO hopefully there is a better way to do this.
    match Path::new(&new_component).components().next() {
        Some(PathComponent::Normal(raw)) => {
            path.push(raw);
            Ok(path)
        }
        Some(com) => Err(ApiError::Simple(format!(
            "`{}` contains invalid component `{:#?}`",
            new_component, com
        ))),
        None => Err(ApiError::Simple("Unknown Error".to_string())),
    }
}

#[derive(Deserialize)]
struct NodeSet {
    nodes: String,
}

async fn document(
    UrlPath(doc_raw): UrlPath<String>,
    nodes: Option<Query<NodeSet>>,
    State(state): State<AppState>,
) -> Result<Html<String>, ApiError> {
    let mut rendered = String::new();
    let mut paths = vec![];
    let mut doc_path = state.library.clone();
    let doc_path = path_append_normal(&mut doc_path, &doc_raw)?;
    if let Some(nodes) = nodes {
        let nodes = nodes.nodes.split(",");
        for node in nodes {
            let node = format!("{}.html", node);
            let mut path = doc_path.clone();
            let path = path_append_normal(&mut path, &node)?;
            if let Some(path) = path.as_path().to_str() {
                paths.push(path.to_string());
            }
        }
    } else {
        let mut dir = tokio::fs::read_dir(doc_path).await?;
        while let Some(ent) = dir.next_entry().await? {
            if let Some(path) = ent.path().as_path().to_str() {
                paths.push(path.to_string());
            }
        }
        paths.sort();
    }
    for path in paths {
        // TODO use spawn blocking to reduce thread spam.
        let file = tokio::fs::read(path).await?;
        rendered.push_str(std::str::from_utf8(&file)?);
    }
    Ok(Html(rendered))
}

async fn node_get(
    UrlPath((doc_raw, node_raw)): UrlPath<(String, String)>,
    State(state): State<AppState>,
) -> Result<Html<String>, ApiError> {
    let mut node_path = state.library.clone();
    path_append_normal(&mut node_path, &doc_raw)?;
    let node_file = format!("{}.html", node_raw);
    path_append_normal(&mut node_path, &node_file)?;
    let file = tokio::fs::read(node_path).await?;
    Ok(Html(String::from_utf8(file)?))
}

async fn node_replace(
    UrlPath((doc_raw, node_raw)): UrlPath<(String, String)>,
    State(state): State<AppState>,
    node_body: String,
) -> Result<StatusCode, ApiError> {
    let mut node_path = state.library.clone();
    path_append_normal(&mut node_path, &doc_raw)?;
    let node_file = format!("{}.html", node_raw);
    path_append_normal(&mut node_path, &node_file)?;
    tokio::fs::write(node_path, node_body).await?;
    Ok(StatusCode::RESET_CONTENT)
}

async fn node_delete(
    UrlPath((doc_raw, node_raw)): UrlPath<(String, String)>,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    let mut node_path = state.library.clone();
    path_append_normal(&mut node_path, &doc_raw)?;
    let node_file = format!("{}.html", node_raw);
    path_append_normal(&mut node_path, &node_file)?;
    tokio::fs::remove_file(node_path).await?;
    Ok(StatusCode::RESET_CONTENT)
}

async fn document_append(
    UrlPath(doc_raw): UrlPath<String>,
    State(state): State<AppState>,
    node_body: String,
) -> Result<StatusCode, ApiError> {
    let mut doc_path = state.library.clone();
    let doc_path = path_append_normal(&mut doc_path, &doc_raw)?;
    let mut dir = tokio::fs::read_dir(doc_path).await?;
    let mut new_canidate: u32 = 0;
    while let Some(ent) = dir.next_entry().await? {
        let path = ent.file_name();
        if !ent.file_type().await?.is_file() {
            continue;
        }
        let Some(path) = path.to_str() else {
            continue;
        };
        let path = path.strip_suffix(".html").unwrap_or(path);
        let Ok(value) = path.parse::<u32>() else {
            continue;
        };
        if value >= new_canidate {
            new_canidate = value + 1;
        }
    }
    let node_raw = format!("{}.html", new_canidate);
    let mut node_path = doc_path.clone();
    path_append_normal(&mut node_path, &node_raw)?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(node_path)
        .await?;
    file.write_all(node_body.as_bytes()).await?;
    file.flush().await?;
    Ok(StatusCode::OK)
}
