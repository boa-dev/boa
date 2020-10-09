use self::instructions::Instruction;
use crate::{realm::Realm, Value};
use std::fmt::{Display, Formatter, Result};

pub(crate) mod compilation;
pub(crate) mod instructions;

// === Misc
#[derive(Copy, Clone, Debug, Default)]
pub struct Reg(u8);

impl Display for Reg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

// === Execution
#[derive(Debug)]
pub struct VM {
    realm: Realm,
    accumulator: Value,
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
            accumulator: Value::undefined(),
            regs: vec![Value::undefined(); 8],
        }
    }

    fn set(&mut self, reg: Reg, val: Value) {
        self.regs[reg.0 as usize] = val;
    }

    fn set_accumulator(&mut self, val: Value) {
        self.accumulator = val;
    }

    // pub fn run(&mut self, instrs: &[Instruction]) -> super::Result<Value> {
    //     let mut idx = 0;

    //     while idx < instrs.len() {
    //         match &instrs[idx] {
    //             Instruction::Ld(r, v) => self.set(*r, v.clone()),

    //             Instruction::Lda(v) => self.set_accumulator(v.clone()),

    //             Instruction::Bind(r, ident) => {
    //                 let val = self.clear(*r);

    //                 if self.realm.environment.has_binding(ident) {
    //                     self.realm.environment.set_mutable_binding(ident, val, true);
    //                 } else {
    //                     self.realm.environment.create_mutable_binding(
    //                         ident.clone(), // fix
    //                         true,
    //                         VariableScope::Function,
    //                     );
    //                     self.realm.environment.initialize_binding(ident, val);
    //                 }
    //             }

    //             Instruction::Add { dest, src } => {
    //                 let l = self.clear(*dest);
    //                 let r = self.clear(*src);

    //                 self.set(*dest, l + r);
    //             }

    //             _ => {
    //                 dbg!(&instrs[idx]);
    //                 panic!();
    //             }
    //         }

    //         idx += 1;
    //     }

    //     Ok(Ok(self.clear(Reg(0))))
    // }
}
