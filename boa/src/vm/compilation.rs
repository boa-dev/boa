use super::*;
use crate::{syntax::ast::Const, syntax::ast::Node, value::RcBigInt, value::RcString};

#[derive(Debug, Default)]
/// The compiler struct holds all the instructions.
pub struct Compiler {
    /// Vector of instructions
    pub(super) instructions: Vec<Instruction>,
    /// The pool stores constant data that can be indexed with the opcodes and pushed on the stack
    pub(super) pool: Vec<Value>,
}

impl Compiler {
    /// Add a new instruction.
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    /// This specilaized method puts the string value in the pool then adds an instructions which points to the correct index
    pub fn add_string_instruction<S>(&mut self, string: S)
    where
        S: Into<RcString>,
    {
        let index = self.pool.len();
        self.add_instruction(Instruction::String(index));
        self.pool.push(string.into().into());
    }

    /// This specilaized method puts the BigInt value in the pool then adds an instructions which points to the correct index
    pub fn add_bigint_instruction<B>(&mut self, bigint: B)
    where
        B: Into<RcBigInt>,
    {
        let index = self.pool.len();
        self.add_instruction(Instruction::BigInt(index));
        self.pool.push(bigint.into().into());
    }
}

pub(crate) trait CodeGen {
    fn compile(&self, compiler: &mut Compiler);
}

impl CodeGen for Node {
    fn compile(&self, compiler: &mut Compiler) {
        let _timer = BoaProfiler::global().start_event(&format!("Node ({})", &self), "codeGen");
        match *self {
            Node::Const(Const::Undefined) => compiler.add_instruction(Instruction::Undefined),
            Node::Const(Const::Null) => compiler.add_instruction(Instruction::Null),
            Node::Const(Const::Bool(true)) => compiler.add_instruction(Instruction::True),
            Node::Const(Const::Bool(false)) => compiler.add_instruction(Instruction::False),
            Node::Const(Const::Num(num)) => compiler.add_instruction(Instruction::Rational(num)),
            Node::Const(Const::Int(num)) => match num {
                0 => compiler.add_instruction(Instruction::Zero),
                1 => compiler.add_instruction(Instruction::One),
                _ => compiler.add_instruction(Instruction::Int32(num)),
            },
            Node::Const(Const::String(ref string)) => {
                compiler.add_string_instruction(string.clone())
            }
            Node::Const(Const::BigInt(ref bigint)) => {
                compiler.add_bigint_instruction(bigint.clone())
            }
            Node::BinOp(ref op) => op.compile(compiler),
            Node::UnaryOp(ref op) => op.compile(compiler),
            Node::VarDeclList(ref list) => {
                for var_decl in list.as_ref() {
                    let name = var_decl.name();
                    let index = compiler.pool.len();
                    compiler.add_instruction(Instruction::DefVar(index));
                    compiler.pool.push(name.into());

                    if let Some(v) = var_decl.init() {
                        v.compile(compiler);
                        compiler.add_instruction(Instruction::InitLexical(index))
                    };
                }
            }
            Node::LetDeclList(ref list) => {
                for let_decl in list.as_ref() {
                    let name = let_decl.name();
                    let index = compiler.pool.len();
                    compiler.add_instruction(Instruction::DefLet(index));
                    compiler.pool.push(name.into());

                    // If name has a value we can init here too
                    if let Some(v) = let_decl.init() {
                        v.compile(compiler);
                        compiler.add_instruction(Instruction::InitLexical(index))
                    };
                }
            }
            Node::ConstDeclList(ref list) => {
                for const_decl in list.as_ref() {
                    let name = const_decl.name();
                    let index = compiler.pool.len();
                    compiler.add_instruction(Instruction::DefConst(index));
                    compiler.pool.push(name.into());

                    if let Some(v) = const_decl.init() {
                        v.compile(compiler);
                        compiler.add_instruction(Instruction::InitLexical(index))
                    };
                }
            }
            Node::Identifier(ref name) => name.compile(compiler),
            _ => unimplemented!(),
        }
    }
}
