use serde::{Deserialize, Serialize};

/// Example domain model for the Books module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub slug: String,
}
