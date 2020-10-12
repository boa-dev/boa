use crate::Value;

use super::Reg;
use std::fmt::{Debug, Error, Formatter};

#[derive(Clone)]
pub enum Instruction {
    // Loads an i32 onto the stack
    Int32(i32),

    /// Loads a value into a register
    Ld(Reg, Value),

    /// Loads a value into the accumulator
    Lda(Value),

    /// Binds a value from a register to an ident
    Bind(Reg, String),

    /// Adds the values from destination and source and stores the result in destination
    Add,
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Add => write!(f, "Add"),
            Self::Int32(i) => write!(f, "Int32\t{}", format!("{}", i)),
            Self::Bind(r, v) => write!(f, "Bind\t{}\t\t{}", r, format!("{:p}", v)),
            Self::Ld(r, v) => write!(f, "Ld\t{}\t\t{}", r, format!("{:p}", v)),
            _ => write!(f, "unimplemented"),
        }
    }
}
