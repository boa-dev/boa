use std::fmt::{Debug, Error, Formatter};

#[derive(Clone, Copy)]
pub enum Instruction {
    /// Adds the values from destination and source and stores the result in destination
    Add,

    // Loads an i32 onto the stack
    Int32(i32),
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Add => write!(f, "Add"),
            Self::Int32(i) => write!(f, "Int32\t{}", format!("{}", i)),
            _ => write!(f, "unimplemented"),
        }
    }
}
