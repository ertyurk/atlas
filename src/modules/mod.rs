pub mod books;
pub mod users;

use atlas_kernel::ModuleRegistry;

/// Register all project-specific modules with the registry
pub fn register_all(registry: &mut ModuleRegistry) {
    registry.register_custom(books::create_module());
    registry.register_custom(users::create_module());
}
