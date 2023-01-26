use crate::{
    bytecompiler::{Access, ByteCompiler},
    vm::Opcode,
    JsResult,
};
use boa_ast::expression::operator::{update::UpdateOp, Update};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_update(&mut self, update: &Update, use_expr: bool) -> JsResult<()> {
        let access = Access::from_update_target(update.target());

        match update.op() {
            UpdateOp::IncrementPre => {
                self.access_set(access, true, |compiler, _| {
                    compiler.access_get(access, true)?;
                    compiler.emit_opcode(Opcode::Inc);
                    Ok(())
                })?;
            }
            UpdateOp::DecrementPre => {
                self.access_set(access, true, |compiler, _| {
                    compiler.access_get(access, true)?;
                    compiler.emit_opcode(Opcode::Dec);
                    Ok(())
                })?;
            }
            UpdateOp::IncrementPost => {
                self.access_set(access, false, |compiler, level| {
                    compiler.access_get(access, true)?;
                    compiler.emit_opcode(Opcode::IncPost);
                    compiler.emit_opcode(Opcode::RotateRight);
                    compiler.emit_u8(level + 2);
                    Ok(())
                })?;
            }
            UpdateOp::DecrementPost => {
                self.access_set(access, false, |compiler, level| {
                    compiler.access_get(access, true)?;
                    compiler.emit_opcode(Opcode::DecPost);
                    compiler.emit_opcode(Opcode::RotateRight);
                    compiler.emit_u8(level + 2);
                    Ok(())
                })?;
            }
        }

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }

        Ok(())
    }
}
