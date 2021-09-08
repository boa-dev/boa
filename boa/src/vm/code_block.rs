use crate::{vm::Opcode, JsString, JsValue};

use std::{convert::TryInto, fmt::Write, mem::size_of};

/// This represents wether an object can be read from [`CodeBlock`] code.
pub unsafe trait Readable {}

unsafe impl Readable for u8 {}
unsafe impl Readable for i8 {}
unsafe impl Readable for u16 {}
unsafe impl Readable for i16 {}
unsafe impl Readable for u32 {}
unsafe impl Readable for i32 {}
unsafe impl Readable for u64 {}
unsafe impl Readable for i64 {}
unsafe impl Readable for f32 {}
unsafe impl Readable for f64 {}

#[derive(Debug)]
pub struct CodeBlock {
    /// Bytecode
    pub(crate) code: Vec<u8>,

    /// Literals
    pub(crate) literals: Vec<JsValue>,

    /// Variables names
    pub(crate) names: Vec<JsString>,
}

impl Default for CodeBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeBlock {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            literals: Vec::new(),
            names: Vec::new(),
        }
    }

    /// Read type T from code.
    ///
    /// # Safety
    ///
    /// Does not check if read happens out-of-bounds.
    pub unsafe fn read_unchecked<T: Readable>(&self, offset: usize) -> T {
        // This has to be an unaligned read because we can't gurantee that
        // the types are aligned.
        self.code.as_ptr().add(offset).cast::<T>().read_unaligned()
    }

    /// Read type T from code.
    pub fn read<T: Readable>(&self, offset: usize) -> T {
        assert!(offset + size_of::<T>() - 1 < self.code.len());

        // Safety: We checked that it is not an out-of-bounds read,
        // so this is safe.
        unsafe { self.read_unchecked(offset) }
    }

    pub(crate) fn instruction_operands(&self, pc: &mut usize) -> String {
        let opcode: Opcode = self.code[*pc].try_into().unwrap();
        *pc += size_of::<Opcode>();
        match opcode {
            Opcode::PushInt8 => {
                let result = self.read::<i8>(*pc).to_string();
                *pc += size_of::<i8>();
                result
            }
            Opcode::PushInt16 => {
                let result = self.read::<i16>(*pc).to_string();
                *pc += size_of::<i16>();
                result
            }
            Opcode::PushInt32 => {
                let result = self.read::<i32>(*pc).to_string();
                *pc += size_of::<i32>();
                result
            }
            Opcode::PushRational => {
                let operand = self.read::<f64>(*pc);
                *pc += size_of::<f64>();
                ryu_js::Buffer::new().format(operand).to_string()
            }
            Opcode::PushLiteral
            | Opcode::PushNewArray
            | Opcode::Jump
            | Opcode::JumpIfFalse
            | Opcode::JumpIfTrue
            | Opcode::Case
            | Opcode::Default
            | Opcode::LogicalAnd
            | Opcode::LogicalOr
            | Opcode::Coalesce => {
                let result = self.read::<u32>(*pc).to_string();
                *pc += size_of::<u32>();
                result
            }
            Opcode::DefVar
            | Opcode::DefLet
            | Opcode::DefConst
            | Opcode::InitLexical
            | Opcode::GetName
            | Opcode::SetName
            | Opcode::GetPropertyByName
            | Opcode::SetPropertyByName => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!("{:04}: '{}'", operand, self.names[operand as usize])
            }
            Opcode::Pop
            | Opcode::Dup
            | Opcode::Swap
            | Opcode::PushZero
            | Opcode::PushOne
            | Opcode::PushNaN
            | Opcode::PushPositiveInfinity
            | Opcode::PushNegativeInfinity
            | Opcode::PushNull
            | Opcode::PushTrue
            | Opcode::PushFalse
            | Opcode::PushUndefined
            | Opcode::PushEmptyObject
            | Opcode::Add
            | Opcode::Sub
            | Opcode::Div
            | Opcode::Mul
            | Opcode::Mod
            | Opcode::Pow
            | Opcode::ShiftRight
            | Opcode::ShiftLeft
            | Opcode::UnsignedShiftRight
            | Opcode::BitOr
            | Opcode::BitAnd
            | Opcode::BitXor
            | Opcode::BitNot
            | Opcode::In
            | Opcode::Eq
            | Opcode::StrictEq
            | Opcode::NotEq
            | Opcode::StrictNotEq
            | Opcode::GreaterThan
            | Opcode::GreaterThanOrEq
            | Opcode::LessThan
            | Opcode::LessThanOrEq
            | Opcode::InstanceOf
            | Opcode::TypeOf
            | Opcode::Void
            | Opcode::LogicalNot
            | Opcode::Pos
            | Opcode::Neg
            | Opcode::GetPropertyByValue
            | Opcode::SetPropertyByValue
            | Opcode::ToBoolean
            | Opcode::Throw
            | Opcode::This
            | Opcode::Nop => String::new(),
        }
    }
}

impl std::fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Code: \n")?;

        writeln!(f, "    Location  Count   Opcode              Operands")?;
        let mut pc = 0;
        let mut count = 0;
        while pc < self.code.len() {
            let opcode: Opcode = self.code[pc].try_into().unwrap();
            write!(
                f,
                "    {:06}    {:04}    {:<20}",
                pc,
                count,
                opcode.as_str()
            )?;
            writeln!(f, "{}", self.instruction_operands(&mut pc))?;
            count += 1;
        }

        f.write_char('\n')?;

        f.write_str("Literals:\n")?;
        if !self.literals.is_empty() {
            for (i, value) in self.literals.iter().enumerate() {
                writeln!(f, "    {:04}: <{}> {}", i, value.type_of(), value.display())?;
            }
        } else {
            writeln!(f, "    <empty>")?;
        }

        f.write_char('\n')?;

        f.write_str("Names:\n")?;
        if !self.names.is_empty() {
            for (i, value) in self.names.iter().enumerate() {
                writeln!(f, "    {:04}: {}", i, value)?;
            }
        } else {
            writeln!(f, "    <empty>")?;
        }

        Ok(())
    }
}
