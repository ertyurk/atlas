pub mod models;
pub mod routes;

/// Entry point for wiring the Books module into the application.
pub fn register() {
    routes::register();
}
