//! Field nodes

pub mod get_const_field;
pub mod get_field;
pub mod get_private_field;
pub mod get_super_field;

pub use self::{
    get_const_field::GetConstField, get_field::GetField, get_private_field::GetPrivateField,
    get_super_field::GetSuperField,
};

#[cfg(test)]
mod tests;
