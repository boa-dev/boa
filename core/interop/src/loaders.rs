//! A collection of [`boa_engine::module::ModuleLoader`]s utilities to help in
//! creating custom module loaders in a combinatorial way.

pub use hashmap::*;
pub use merge::*;
pub use predicate::*;

pub mod hashmap;
pub mod merge;
pub mod predicate;
