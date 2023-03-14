use boa_interner::Sym;

use crate::{js_string, vm::Opcode};

use super::{ByteCompiler, Literal};

impl ByteCompiler<'_, '_> {
    /// Closes an async iterator
    ///
    /// This is equal to the [`AsyncIteratorClose`][spec] operation.
    ///
    /// Stack:
    ///  - iterator, next_method **=>** \<empty\>
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asynciteratorclose
    pub(super) fn async_iterator_close(&mut self) {
        // Need to remove `next_method` to manipulate the iterator
        self.emit_opcode(Opcode::Pop);

        let index = self.get_or_insert_name(Sym::RETURN.into());
        self.emit(Opcode::GetMethod, &[index]);
        let skip_jump = self.jump_if_not_undefined();

        // The iterator is still in the stack, so pop it to cleanup.
        self.emit_opcode(Opcode::Pop);
        let early_exit = self.jump();

        self.patch_jump(skip_jump);
        self.emit(Opcode::Call, &[0]);
        self.emit_opcode(Opcode::Await);
        self.emit_opcode(Opcode::GeneratorNext);
        self.emit_opcode(Opcode::IsObject);
        let skip_throw = self.jump_if_true();

        let error_msg = self.get_or_insert_literal(Literal::String(js_string!(
            "inner result was not an object"
        )));
        self.emit(Opcode::ThrowNewTypeError, &[error_msg]);

        self.patch_jump(skip_throw);
        self.patch_jump(early_exit);
    }
}
