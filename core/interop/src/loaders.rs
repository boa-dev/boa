//! A collection of JS [`boa_engine::module::ModuleLoader`]s utilities to help in
//! creating custom module loaders.
pub mod cached;
pub mod embedded;
pub mod fallback;
pub mod filesystem;
pub mod functions;
pub mod hashmap;

pub use cached::CachedModuleLoader;
pub use fallback::FallbackModuleLoader;
pub use filesystem::FsModuleLoader;
pub use functions::FnModuleLoader;
pub use hashmap::HashMapModuleLoader;

/// Create a [`FallbackModuleLoader`] from two loaders. This is a simple utility
/// function.
#[must_use]
pub fn fallback<L, R>(loader: L, fallback: R) -> FallbackModuleLoader<L, R>
where
    L: boa_engine::module::ModuleLoader,
    R: boa_engine::module::ModuleLoader + Clone + 'static,
{
    FallbackModuleLoader::new(loader, fallback)
}

/// Create a [`FsModuleLoader`] from a root path. This is a simple utility function.
#[must_use]
pub fn filesystem(root: std::path::PathBuf) -> FsModuleLoader {
    FsModuleLoader::new(root)
}
