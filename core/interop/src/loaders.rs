//! A collection of JS [`boa_engine::module::ModuleLoader`]s utilities to help in
//! creating custom module loaders.

pub use hashmap::HashMapModuleLoader;

pub mod json;

pub mod hashmap;
