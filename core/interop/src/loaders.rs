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
