use super::Reg;
use crate::builtins::value::Value;

#[derive(Debug, Clone)]
pub enum Instruction {
    /// Loads a value into a register
    Ld(Reg, Value),

    /// Binds a value from a register to an ident
    Bind(Reg, String),

    /// Adds the values from destination and source and stores the result in destination
    Add { dest: Reg, src: Reg },
}
