use crate::{
    js_string,
    vm::{GeneratorResumeKind, Opcode},
};

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
    pub(super) fn close_active_iterators(&mut self) {
        let start = self.next_opcode_location();
        self.emit_opcode(Opcode::IteratorStackEmpty);
        let empty = self.jump_if_true();
        self.iterator_close(self.in_async_generator);
        self.emit(Opcode::Jump, &[start]);
        self.patch_jump(empty);
    }

    /// Yields from the current generator.
    ///
    /// This is equivalent to the [`Yield ( value )`][yield] operation from the spec.
    ///
    /// stack:
    /// - value **=>** received
    ///
    /// [yield]: https://tc39.es/ecma262/#sec-yield
    pub(super) fn r#yield(&mut self) {
        // 1. Let generatorKind be GetGeneratorKind().
        if self.in_async_generator {
            // 2. If generatorKind is async, return ? AsyncGeneratorYield(? Await(value)).
            self.emit_opcode(Opcode::Await);
            self.emit_opcode(Opcode::GeneratorNext);
            self.async_generator_yield();
        } else {
            // 3. Otherwise, return ? GeneratorYield(CreateIterResultObject(value, false)).
            self.emit_opcode(Opcode::CreateIteratorResult);
            self.emit_u8(u8::from(false));
            self.emit_opcode(Opcode::GeneratorYield);
        }

        self.emit_opcode(Opcode::GeneratorNext);
    }

    /// Yields from the current async generator.
    ///
    /// This is equivalent to the [`AsyncGeneratorYield ( value )`][async_yield] operation from the spec.
    ///
    /// stack:
    /// - value **=>** received
    ///
    /// [async_yield]: https://tc39.es/ecma262/#sec-asyncgeneratoryield
    pub(super) fn async_generator_yield(&mut self) {
        // Stack: value
        self.emit_opcode(Opcode::AsyncGeneratorYield);

        // Stack: resume_kind, received
        let non_return_resume = self.emit_opcode_with_operand(Opcode::JumpIfNotResumeKind);
        self.emit_u8(GeneratorResumeKind::Return as u8);

        // Stack: resume_kind(Return), received
        self.emit_opcode(Opcode::Pop);

        // Stack: received
        self.emit_opcode(Opcode::Await);

        // Stack: resume_kind, received
        let non_normal_resume = self.emit_opcode_with_operand(Opcode::JumpIfNotResumeKind);
        self.emit_u8(GeneratorResumeKind::Normal as u8);

        // Stack: resume_kind(Normal), received
        self.emit_opcode(Opcode::Pop);

        // Stack: received
        self.emit_push_integer(GeneratorResumeKind::Return as i32);

        // Stack: resume_kind(Return) received
        self.patch_jump(non_normal_resume);
        self.patch_jump(non_return_resume);

        // Stack: resume_kind, received
    }
}
