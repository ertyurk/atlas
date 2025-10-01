use serde::{Deserialize, Serialize};

/// Example domain model for the Books module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    /// Unique identifier for the book
    pub id: String,
    /// Title of the book
    pub title: String,
    /// Author of the book
    pub author: String,
    /// URL-friendly slug for the book
    pub slug: String,
}

/// Request model for creating a new book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBook {
    /// Title of the book
    pub title: String,
    /// Author of the book
    pub author: String,
    /// URL-friendly slug for the book
    pub slug: String,
}
