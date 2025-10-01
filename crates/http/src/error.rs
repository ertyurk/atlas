//! Error handling for ATLAS HTTP layer

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

/// Standard error response format for all HTTP errors
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub details: Vec<serde_json::Value>,
    pub message: String,
    pub code: String,
    pub trace_id: String,
    pub timestamp: String,
}

/// Application error types that map to HTTP responses
#[derive(Error, Debug)]
pub enum AppError {
    #[error("validation error: {message}")]
    Validation {
        details: Vec<serde_json::Value>,
        code: String,
        message: String,
    },

    #[error("conflict: {message}")]
    Conflict {
        details: Vec<serde_json::Value>,
        code: String,
        message: String,
    },

    #[error("not found: {message}")]
    NotFound { message: String, code: String },

    #[error("unauthorized: {message}")]
    Unauthorized { message: String, code: String },

    #[error("forbidden: {message}")]
    Forbidden { message: String, code: String },

    #[error("bad request: {message}")]
    BadRequest { message: String, code: String },

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Create a validation error
    pub fn validation(details: Vec<serde_json::Value>, message: impl Into<String>) -> Self {
        Self::Validation {
            details,
            code: "validation_error".to_string(),
            message: message.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict(details: Vec<serde_json::Value>, message: impl Into<String>) -> Self {
        Self::Conflict {
            details,
            code: "conflict".to_string(),
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
            code: "not_found".to_string(),
        }
    }

    /// Create an unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
            code: "unauthorized".to_string(),
        }
    }

    /// Create a forbidden error
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
            code: "forbidden".to_string(),
        }
    }

    /// Create a bad request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
            code: "bad_request".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_id = Uuid::new_v4();
        let timestamp = OffsetDateTime::now_utc().to_string();

        let (status, error_code, message, details) = match self {
            AppError::Validation {
                details,
                code,
                message,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                code,
                message,
                Some(details),
            ),
            AppError::Conflict {
                details,
                code,
                message,
            } => (StatusCode::CONFLICT, code, message, Some(details)),
            AppError::NotFound { message, code } => (StatusCode::NOT_FOUND, code, message, None),
            AppError::Unauthorized { message, code } => {
                (StatusCode::UNAUTHORIZED, code, message, None)
            }
            AppError::Forbidden { message, code } => (StatusCode::FORBIDDEN, code, message, None),
            AppError::BadRequest { message, code } => {
                (StatusCode::BAD_REQUEST, code, message, None)
            }
            AppError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error".to_string(),
                e.to_string(),
                None,
            ),
        };

        tracing::error!(
            error_id = %error_id,
            error_code = %error_code,
            status_code = %status.as_u16(),
            "Request error"
        );

        // In production, we might want to hide internal error details
        let message = if cfg!(not(debug_assertions)) && status == StatusCode::INTERNAL_SERVER_ERROR
        {
            "An internal server error occurred".to_string()
        } else {
            message
        };

        let error_response = json!({
            "error": {
                "code": error_code,
                "message": message,
                "details": details.unwrap_or_default(),
                "trace_id": error_id.to_string(),
                "timestamp": timestamp
            }
        });

        (status, Json(error_response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_validation_error() {
        let details = vec![serde_json::json!({"field": "slug", "error": "required"})];
        let error = AppError::validation(details.clone(), "Validation failed");

        match error {
            AppError::Validation {
                details: d,
                code,
                message,
            } => {
                assert_eq!(d, details);
                assert_eq!(code, "validation_error");
                assert_eq!(message, "Validation failed");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_error_response_mapping() {
        let error = AppError::not_found("Resource not found");
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_internal_error_mapping() {
        let internal_error = anyhow::anyhow!("Database connection failed");
        let error = AppError::Internal(internal_error);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_format() {
        let error = AppError::not_found("Test resource not found");
        let response = error.into_response();

        // The response should include the new error format with trace_id and timestamp
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // In a real test, you would extract and verify the JSON body contains:
        // - error.code
        // - error.message
        // - error.details
        // - error.trace_id (UUID format)
        // - error.timestamp (ISO 8601 format)
    }
}
