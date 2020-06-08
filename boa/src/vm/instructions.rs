use super::Reg;
use crate::builtins::value::Value;
use std::fmt::{Debug, Error, Formatter};

#[derive(Clone)]
pub enum Instruction {
    /// Loads a value into a register
    Ld(Reg, Value),

    /// Loads a value into the accumulator
    Lda(Value),

    /// Binds a value from a register to an ident
    Bind(Reg, String),

    /// Adds the values from destination and source and stores the result in destination
    Add { dest: Reg, src: Reg },
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Ld(r, v) => write!(f, "Ld\t{}\t\t{}", r, v),
            Self::Bind(r, v) => write!(f, "Bind\t{}\t\t{}", r, v),
            Self::Add { dest, src } => write!(f, "Add\t{}\t\t{}", dest, src),
            _ => write!(f, "unimplemented"),
        }
    }
}
