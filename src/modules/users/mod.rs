use async_trait::async_trait;
use atlas_kernel::{InitCtx, Migration, Module};
use axum::{routing::get, Router};
use serde_json::json;

/// Users module implementation for testing dynamic OpenAPI collection
pub struct UsersModule;

impl UsersModule {
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Module for UsersModule {
    fn name(&self) -> &'static str {
        "users"
    }

    async fn init(&self, ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        tracing::info!(
            module = self.name(),
            environment = ?ctx.settings.environment,
            "users module initialized"
        );
        Ok(())
    }

    fn routes(&self) -> Router {
        Router::new()
            .route("/", get(list_users))
            .route("/health", get(health_check))
            .route("/profile", get(get_profile))
    }

    fn openapi(&self) -> Option<serde_json::Value> {
        Some(json!({
            "paths": {
                "/": {
                    "get": {
                        "summary": "List users",
                        "tags": ["Users"],
                        "responses": {
                            "200": {
                                "description": "List of users",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "array",
                                            "items": {
                                                "$ref": "#/components/schemas/User"
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
                        "summary": "Users health check",
                        "tags": ["Users"],
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
                            }
                        }
                    }
                },
                "/profile": {
                    "get": {
                        "summary": "Get user profile",
                        "tags": ["Users"],
                        "responses": {
                            "200": {
                                "description": "User profile",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/UserProfile"
                                        }
                                    }
                                }
                            },
                            "404": {
                                "description": "User not found",
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
                    "User": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique identifier for the user"
                            },
                            "email": {
                                "type": "string",
                                "format": "email",
                                "description": "User's email address"
                            },
                            "name": {
                                "type": "string",
                                "description": "User's full name"
                            },
                            "created_at": {
                                "type": "string",
                                "format": "date-time",
                                "description": "When the user was created"
                            }
                        },
                        "required": ["id", "email", "name", "created_at"]
                    },
                    "UserProfile": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique identifier for the user"
                            },
                            "email": {
                                "type": "string",
                                "format": "email",
                                "description": "User's email address"
                            },
                            "name": {
                                "type": "string",
                                "description": "User's full name"
                            },
                            "bio": {
                                "type": "string",
                                "description": "User's biography"
                            },
                            "avatar_url": {
                                "type": "string",
                                "format": "uri",
                                "description": "URL to user's avatar image"
                            },
                            "created_at": {
                                "type": "string",
                                "format": "date-time",
                                "description": "When the user was created"
                            },
                            "updated_at": {
                                "type": "string",
                                "format": "date-time",
                                "description": "When the user was last updated"
                            }
                        },
                        "required": ["id", "email", "name", "created_at"]
                    }
                }
            }
        }))
    }

    fn migrations(&self) -> Vec<Migration> {
        vec![Migration {
            id: "001_init",
            up: r#"
                DEFINE TABLE user SCHEMAFULL;
                DEFINE FIELD email     ON user TYPE string ASSERT $value != "";
                DEFINE FIELD name      ON user TYPE string ASSERT $value != "";
                DEFINE FIELD bio       ON user TYPE string;
                DEFINE FIELD avatar_url ON user TYPE string;
                DEFINE INDEX user_email_unique ON user FIELDS email UNIQUE;
                "#,
        }]
    }

    async fn start(&self, _ctx: &InitCtx<'_>) -> anyhow::Result<()> {
        tracing::info!(module = self.name(), "users module started");
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        tracing::info!(module = self.name(), "users module stopped");
        Ok(())
    }
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "users module is healthy"
}

/// List users endpoint (stub implementation)
async fn list_users() -> axum::Json<Vec<serde_json::Value>> {
    let users = vec![
        json!({
            "id": "user-1",
            "email": "john@example.com",
            "name": "John Doe",
            "created_at": "2024-01-01T00:00:00Z"
        }),
        json!({
            "id": "user-2",
            "email": "jane@example.com",
            "name": "Jane Smith",
            "created_at": "2024-01-02T00:00:00Z"
        }),
    ];

    axum::Json(users)
}

/// Get user profile endpoint (stub implementation)
async fn get_profile() -> axum::Json<serde_json::Value> {
    axum::Json(json!({
        "id": "user-1",
        "email": "john@example.com",
        "name": "John Doe",
        "bio": "Software developer passionate about Rust",
        "avatar_url": "https://example.com/avatars/john.jpg",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-15T10:30:00Z"
    }))
}

/// Create a new instance of the users module
pub fn create_module() -> std::sync::Arc<dyn Module> {
    std::sync::Arc::new(UsersModule::new())
}
