use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri};
use axum::response::Response;
use axum::{response::IntoResponse, Router};
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::{env, process};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // initialize tracing
    tracing_subscriber::fmt::init();
    // let j = serde_json::to_string_pretty(&files).unwrap();
    // println!("{j}");
    // let mut file = OpenOptions::new()
    //     .write(true)
    //     .open("./out.json")
    //     .await
    //     .unwrap();
    // file.write_all(j.as_bytes()).await.unwrap();
    // file.flush().await.unwrap();
    //println!("file saved!");

    let state = AppState {
        base_path: PathBuf::from(env::current_dir().unwrap()),
    };
    println!("{state:?}");
    let app = Router::new().fallback(fallback_handler).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!(
        "listening on http://{}",
        listener.local_addr().unwrap().to_string()
    );
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[tracing::instrument]
#[axum::debug_handler]
async fn fallback_handler(uri: Uri, State(s): State<AppState>) -> Result<Response, AppError> {
    let path = s.base_path.join(uri.path().trim_matches('/'));
    //println!("{path:?}");
    if !path.exists() {
        return Ok((StatusCode::NOT_FOUND, "not found!").into_response());
    }
    if path.is_file() {
        return Ok("thisss is a fileee and download not implemented".into_response());
    }
    if path.is_dir() {
        let mut reader = fs::read_dir(&path).await?;
        let mut entries = Vec::new();
        while let Some(entry) = reader.next_entry().await? {
            let m = entry.metadata().await?;
            let attr = m.file_attributes();
            let file_name = entry
                .file_name()
                .into_string()
                .unwrap_or("Unknown (ERROR 01: failed to convert Os String to String)".to_string());
            if (attr & 0x2) != 0 || file_name.starts_with('.') {
                continue;
            }
            entries.push(file_name)
        }
        let mut headers = HeaderMap::new();
        headers.append(CONTENT_TYPE, HeaderValue::from_static("text/html"));
        return Ok((
            headers,
            entries
                .iter()
                .map(|n| {
                    let mut p = uri.path().to_string() + n + "/";

                    format!("<a href='{p}'>{n}</a><br>")
                })
                .collect::<String>(),
        )
            .into_response());
    }
    Ok(path.to_str().unwrap().to_string().into_response())
}

#[derive(Debug, Clone)]
struct AppState {
    base_path: PathBuf,
}

// fn extract_file_name(p: &Path) -> String {
//     p.file_name()
//         .and_then(|x| x.to_str())
//         .map(|x| x.to_string())
//         .unwrap_or(String::from("Unknown"))
// }

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
