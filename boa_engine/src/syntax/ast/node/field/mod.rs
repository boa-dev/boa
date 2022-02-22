//! Field nodes

pub mod get_const_field;
pub mod get_field;

pub use self::{get_const_field::GetConstField, get_field::GetField};

#[cfg(test)]
mod tests;
