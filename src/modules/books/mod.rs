pub mod models;

use async_trait::async_trait;
use atlas_kernel::{InitCtx, Migration, Module};
use axum::{routing::get, Router};
use serde_json::json;

/// Books module implementation for testing the ATLAS module lifecycle
pub struct BooksModule;

impl BooksModule {
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Module for BooksModule {
    fn name(&self) -> &'static str {
        "books"
    }

    async fn init(&self, ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        tracing::info!(
            module = self.name(),
            environment = ?ctx.settings.environment,
            "books module initialized"
        );
        Ok(())
    }

    fn routes(&self) -> Router {
        Router::new()
            .route("/", get(list_books))
            .route("/health", get(health_check))
            .route("/error-test", get(error_test))
    }

    fn openapi(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "paths": {
                "/": {
                    "get": {
                        "summary": "List books",
                        "tags": ["Books"],
                        "responses": {
                            "200": {
                                "description": "List of books",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "array",
                                            "items": {
                                                "$ref": "#/components/schemas/Book"
                                            }
                                        }
                                    }
                                }
                            },
                            "500": {
                                "description": "Internal server error",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/ErrorResponse"
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/health": {
                    "get": {
                        "summary": "Books health check",
                        "tags": ["Books"],
                        "responses": {
                            "200": {
                                "description": "OK",
                                "content": {
                                    "text/plain": {
                                        "schema": {
                                            "type": "string"
                                        }
                                    }
                                }
                            },
                            "500": {
                                "description": "Internal server error",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/ErrorResponse"
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/error-test": {
                    "get": {
                        "summary": "Error test endpoint",
                        "tags": ["Books"],
                        "responses": {
                            "422": {
                                "description": "Validation error",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/ErrorResponse"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "Book": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique identifier for the book"
                            },
                            "title": {
                                "type": "string",
                                "description": "Title of the book"
                            },
                            "author": {
                                "type": "string",
                                "description": "Author of the book"
                            },
                            "slug": {
                                "type": "string",
                                "description": "URL-friendly slug for the book"
                            }
                        },
                        "required": ["id", "title", "author", "slug"]
                    },
                    "CreateBook": {
                        "type": "object",
                        "properties": {
                            "title": {
                                "type": "string",
                                "description": "Title of the book"
                            },
                            "author": {
                                "type": "string",
                                "description": "Author of the book"
                            },
                            "slug": {
                                "type": "string",
                                "description": "URL-friendly slug for the book"
                            }
                        },
                        "required": ["title", "author", "slug"]
                    }
                }
            }
        }))
    }

    fn migrations(&self) -> Vec<Migration> {
        vec![Migration {
            id: "001_init",
            up: r#"
                DEFINE TABLE book SCHEMAFULL;
                DEFINE FIELD title  ON book TYPE string ASSERT $value != "";
                DEFINE FIELD author ON book TYPE string ASSERT $value != "";
                DEFINE FIELD slug   ON book TYPE string ASSERT $value != "";
                DEFINE INDEX book_slug_unique ON book FIELDS slug UNIQUE;
                "#,
        }]
    }

    async fn start(&self, _ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        tracing::info!(module = self.name(), "books module started");
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        tracing::info!(module = self.name(), "books module stopped");
        Ok(())
    }
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "books module is healthy"
}

/// List books endpoint (stub implementation)
async fn list_books() -> axum::Json<Vec<models::Book>> {
    let books = vec![
        models::Book {
            id: "book-1".to_string(),
            title: "The Rust Programming Language".to_string(),
            author: "Steve Klabnik".to_string(),
            slug: "rust-programming-language".to_string(),
        },
        models::Book {
            id: "book-2".to_string(),
            title: "Programming Rust".to_string(),
            author: "Jim Blandy".to_string(),
            slug: "programming-rust".to_string(),
        },
    ];

    axum::Json(books)
}

/// Error test endpoint to demonstrate the new error format
async fn error_test() -> Result<axum::Json<serde_json::Value>, atlas_http::error::AppError> {
    // Return a validation error to demonstrate the new error format
    Err(atlas_http::error::AppError::validation(
        vec![json!({"field": "slug", "error": "required"})],
        "This is a test validation error to demonstrate the new error format with trace_id and timestamp"
    ))
}

/// Create a new instance of the books module
pub fn create_module() -> std::sync::Arc<dyn Module> {
    std::sync::Arc::new(BooksModule::new())
}
