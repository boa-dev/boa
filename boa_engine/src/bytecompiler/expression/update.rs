use crate::{
    bytecompiler::{Access, ByteCompiler},
    vm::Opcode,
};
use boa_ast::expression::operator::{update::UpdateOp, Update};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_update(&mut self, update: &Update, use_expr: bool) {
        let access = Access::from_update_target(update.target());

        match update.op() {
            UpdateOp::IncrementPre => {
                self.access_set(access, true, |compiler, _| {
                    compiler.access_get(access, true);
                    compiler.emit_opcode(Opcode::Inc);
                });
            }
            UpdateOp::DecrementPre => {
                self.access_set(access, true, |compiler, _| {
                    compiler.access_get(access, true);
                    compiler.emit_opcode(Opcode::Dec);
                });
            }
            UpdateOp::IncrementPost => {
                self.access_set(access, false, |compiler, level| {
                    compiler.access_get(access, true);
                    compiler.emit_opcode(Opcode::IncPost);
                    compiler.emit_opcode(Opcode::RotateRight);
                    compiler.emit_u8(level + 2);
                });
            }
            UpdateOp::DecrementPost => {
                self.access_set(access, false, |compiler, level| {
                    compiler.access_get(access, true);
                    compiler.emit_opcode(Opcode::DecPost);
                    compiler.emit_opcode(Opcode::RotateRight);
                    compiler.emit_u8(level + 2);
                });
            }
        }

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }
}
