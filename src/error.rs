use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Forbidden,
    BadRequest(String),
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Internal(other.to_string()),
        }
    }
}

impl From<minijinja::Error> for AppError {
    fn from(err: minijinja::Error) -> Self {
        AppError::Internal(format!("template: {err:#}"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "We could not find that page.".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Your account cannot open this page.".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => {
                tracing::error!("internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong on our side.".to_string())
            }
        };
        let body = format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>Dispatchly</title>\
             <link rel=\"stylesheet\" href=\"/static/css/app.css\"></head><body><main class=\"page narrow\">\
             <h1>{}</h1><p>{}</p><p><a href=\"/\">Back to the board</a></p></main></body></html>",
            status.as_u16(),
            html_escape(&message)
        );
        (status, Html(body)).into_response()
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
