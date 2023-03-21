use boa_interner::Sym;

use crate::{js_string, vm::Opcode};

use super::{ByteCompiler, Literal};

impl ByteCompiler<'_, '_> {
    /// Closes an iterator
    ///
    /// This is equivalent to the [`IteratorClose`][iter] and [`AsyncIteratorClose`][async]
    /// operations.
    ///
    /// Stack:
    ///  - iterator, `next_method`, done **=>** \<empty\>
    ///
    /// [iter]: https://tc39.es/ecma262/#sec-iteratorclose
    /// [async]: https://tc39.es/ecma262/#sec-asynciteratorclose
    pub(super) fn iterator_close(&mut self, async_: bool) {
        // Need to remove `next_method` to manipulate the iterator
        self.emit_opcode(Opcode::Swap);
        self.emit_opcode(Opcode::Pop);

        let skip_iter_pop = self.jump_if_false();

        // `iterator` is done, we can skip calling `return`.
        // `iterator` is still in the stack, so pop it to cleanup.
        self.emit_opcode(Opcode::Pop);
        let skip_return = self.jump();

        // iterator didn't finish iterating.
        self.patch_jump(skip_iter_pop);
        let index = self.get_or_insert_name(Sym::RETURN.into());
        self.emit(Opcode::GetMethod, &[index]);
        let skip_jump = self.jump_if_not_undefined();

        // `iterator` didn't have a `return` method, so we can early exit.
        // `iterator` is still in the stack, so pop it to cleanup.
        self.emit_opcode(Opcode::Pop);
        let early_exit = self.jump();

        self.patch_jump(skip_jump);
        self.emit(Opcode::Call, &[0]);
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
        self.patch_jump(skip_throw);
        self.patch_jump(early_exit);
    }
}
