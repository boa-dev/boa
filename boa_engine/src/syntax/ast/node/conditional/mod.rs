//! Conditional nodes

pub mod conditional_op;
pub mod if_node;

pub use self::{conditional_op::ConditionalOp, if_node::If};

#[cfg(test)]
mod tests;
