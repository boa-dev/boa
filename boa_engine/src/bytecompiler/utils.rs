use crate::{js_string, vm::Opcode};

use super::{ByteCompiler, Literal};

impl ByteCompiler<'_, '_> {
    /// Closes an iterator
    ///
    /// This is equivalent to the [`IteratorClose`][iter] and [`AsyncIteratorClose`][async]
    /// operations.
    ///
    /// Iterator Stack:
    ///  - iterator **=>** \<empty\>
    ///
    /// [iter]: https://tc39.es/ecma262/#sec-iteratorclose
    /// [async]: https://tc39.es/ecma262/#sec-asynciteratorclose
    pub(super) fn iterator_close(&mut self, async_: bool) {
        self.emit_opcode(Opcode::IteratorDone);

        let skip_return = self.jump_if_true();

        // iterator didn't finish iterating.
        self.emit_opcode(Opcode::IteratorReturn);

        // `iterator` didn't have a `return` method, so we can early exit.
        let early_exit = self.jump_if_false();
        if async_ {
            self.emit_opcode(Opcode::Await);
            self.emit_opcode(Opcode::GeneratorNext);
        }
        self.emit_opcode(Opcode::IsObject);
        let skip_throw = self.jump_if_true();

        let error_msg = self.get_or_insert_literal(Literal::String(js_string!(
            "inner result was not an object"
        )));
        self.emit(Opcode::ThrowNewTypeError, &[error_msg]);

        self.patch_jump(skip_return);
        self.emit_opcode(Opcode::IteratorPop);

        self.patch_jump(skip_throw);
        self.patch_jump(early_exit);
    }

    /// Closes all active iterators in the current [`CallFrame`][crate::vm::CallFrame].
    pub(super) fn close_active_iterators(&mut self, async_: bool) {
        let start = self.next_opcode_location();
        self.emit_opcode(Opcode::IteratorStackEmpty);
        let empty = self.jump_if_true();
        self.iterator_close(async_);
        self.emit(Opcode::Jump, &[start]);
        self.patch_jump(empty);
    }
}
