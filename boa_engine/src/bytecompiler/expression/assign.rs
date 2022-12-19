use boa_ast::expression::operator::{assign::AssignOp, Assign};

use crate::{
    bytecompiler::{Access, ByteCompiler},
    vm::{BindingOpcode, Opcode},
    JsResult,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_assign(&mut self, assign: &Assign, use_expr: bool) -> JsResult<()> {
        if assign.op() == AssignOp::Assign {
            match Access::from_assign_target(assign.lhs()) {
                Ok(access) => self.access_set(access, use_expr, |compiler, _| {
                    compiler.compile_expr(assign.rhs(), true)?;
                    Ok(())
                })?,
                Err(pattern) => {
                    self.compile_expr(assign.rhs(), true)?;
                    if use_expr {
                        self.emit_opcode(Opcode::Dup);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::SetName)?;
                }
            }
        } else {
            let access = Access::from_assign_target(assign.lhs())
                .expect("patterns should throw early errors on complex assignment operators");

            let shortcircuit_operator_compilation =
                |compiler: &mut ByteCompiler<'_>, opcode: Opcode| -> JsResult<()> {
                    let (early_exit, pop_count) =
                        compiler.access_set(access, use_expr, |compiler, level| {
                            compiler.access_get(access, true)?;
                            let early_exit = compiler.emit_opcode_with_operand(opcode);
                            compiler.compile_expr(assign.rhs(), true)?;
                            Ok((early_exit, level))
                        })?;
                    if pop_count == 0 {
                        compiler.patch_jump(early_exit);
                    } else {
                        let exit = compiler.emit_opcode_with_operand(Opcode::Jump);
                        compiler.patch_jump(early_exit);
                        for _ in 0..pop_count {
                            compiler.emit_opcode(Opcode::Swap);
                            compiler.emit_opcode(Opcode::Pop);
                        }
                        compiler.patch_jump(exit);
                    }
                    Ok(())
                };

            let opcode = match assign.op() {
                AssignOp::Assign => unreachable!(),
                AssignOp::Add => Opcode::Add,
                AssignOp::Sub => Opcode::Sub,
                AssignOp::Mul => Opcode::Mul,
                AssignOp::Div => Opcode::Div,
                AssignOp::Mod => Opcode::Mod,
                AssignOp::Exp => Opcode::Pow,
                AssignOp::And => Opcode::BitAnd,
                AssignOp::Or => Opcode::BitOr,
                AssignOp::Xor => Opcode::BitXor,
                AssignOp::Shl => Opcode::ShiftLeft,
                AssignOp::Shr => Opcode::ShiftRight,
                AssignOp::Ushr => Opcode::UnsignedShiftRight,
                AssignOp::BoolAnd => {
                    shortcircuit_operator_compilation(self, Opcode::LogicalAnd)?;
                    return Ok(());
                }
                AssignOp::BoolOr => {
                    shortcircuit_operator_compilation(self, Opcode::LogicalOr)?;
                    return Ok(());
                }
                AssignOp::Coalesce => {
                    shortcircuit_operator_compilation(self, Opcode::Coalesce)?;
                    return Ok(());
                }
            };

            self.access_set(access, use_expr, |compiler, _| {
                compiler.access_get(access, true)?;
                compiler.compile_expr(assign.rhs(), true)?;
                compiler.emit(opcode, &[]);
                Ok(())
            })?;
        }

        Ok(())
    }
}
