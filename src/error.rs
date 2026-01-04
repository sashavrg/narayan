use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("FFmpeg conversion failed: {0}")]
    ConversionError(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Invalid file type")]
    InvalidFileType,

    #[error("File too large")]
    FileTooLarge,

    #[error("Too many files in job (max: {0})")]
    TooManyFiles(usize),

    #[error("No files provided")]
    NoFilesProvided,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("FFmpeg not found")]
    FFmpegNotFound,

    #[error("Job creation failed: {0}")]
    JobCreationFailed(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("ZIP creation failed: {0}")]
    ZipError(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidFileType => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::FileTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            AppError::TooManyFiles(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NoFilesProvided => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::JobNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::FFmpegNotFound => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "FFmpeg is not installed on the server".to_string(),
            ),
            AppError::ConversionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::JobCreationFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::WebSocketError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::ZipError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
