use crate::{
    builtins::value::{ResultValue, Value},
    environment::lexical_environment::VariableScope,
    realm::Realm,
    syntax::ast::{self, Node},
};
use gc::{Finalize, Gc, GcCell, GcCellRef, Trace};

pub(crate) mod compilation;

#[cfg(test)]
mod tests;

// === Misc
#[derive(Copy, Clone, Debug, Default)]
pub struct Reg(u8);

#[derive(Debug, Clone)]
pub enum In {
    /// Loads a value into a register
    Ld(Reg, Value),

    /// Binds a value from a register to an ident
    Bind(Reg, String),

    /// Adds the values from destination and source and stores the result in destination
    Add { dest: Reg, src: Reg },
}

// === Execution
#[derive(Debug)]
pub struct VM {
    realm: Realm,
    regs: Vec<Value>, // TODO: find a possible way of this being an array
}

impl VM {
    /// Sets a register's value to `undefined` and returns its previous one
    fn clear(&mut self, reg: Reg) -> Value {
        let v = self.regs[reg.0 as usize].clone();
        self.regs[reg.0 as usize] = Value::undefined();
        v
    }

    pub fn new(realm: Realm) -> Self {
        VM {
            realm,
            regs: vec![Value::undefined(); 8],
        }
    }

    fn set(&mut self, reg: Reg, val: Value) {
        self.regs[reg.0 as usize] = val;
    }

    pub fn run(&mut self, instrs: &[In]) -> ResultValue {
        let mut idx = 0;

        while idx < instrs.len() {
            match &instrs[idx] {
                In::Ld(r, v) => self.set(*r, v.clone()),

                In::Bind(r, ident) => {
                    let val = self.clear(*r);

                    if self.realm.environment.has_binding(ident) {
                        self.realm.environment.set_mutable_binding(ident, val, true);
                    } else {
                        self.realm.environment.create_mutable_binding(
                            ident.clone(), // fix
                            true,
                            VariableScope::Function,
                        );
                        self.realm.environment.initialize_binding(ident, val);
                    }
                }

                In::Add { dest, src } => {
                    let l = self.clear(*dest);
                    let r = self.clear(*src);

                    self.set(*dest, l + r);
                }

                _ => {
                    dbg!(&instrs[idx]);
                    panic!();
                }
            }

            idx += 1;
        }

        Ok(self.clear(Reg(0)))
    }
}
