pub mod books;

/// Register all project-specific modules.
pub fn register_all() {
    books::register();
}
