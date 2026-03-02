use super::{ByteCompiler, Literal, Register};
use crate::vm::opcode::{
    AsyncGeneratorYield, Await, CreateIteratorResult, GeneratorNext, GeneratorYield, IsObject,
    IteratorReturn, IteratorStackEmpty, Jump, ThrowNewTypeError,
};
use crate::{js_string, vm::GeneratorResumeKind};

impl ByteCompiler<'_> {
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
        let value = self.register_allocator.alloc();
        let called = self.register_allocator.alloc();
        IteratorReturn::emit(self, value.variable(), called.variable());

        // `iterator` didn't have a `return` method, is already done or is not on the iterator stack.
        let early_exit = self.jump_if_false(&called);
        self.register_allocator.dealloc(called);

        if async_ {
            Await::emit(self, value.variable());
            let resume_kind = self.register_allocator.alloc();
            self.pop_into_register(&resume_kind);
            self.pop_into_register(&value);
            GeneratorNext::emit(self, resume_kind.variable(), value.variable());
            self.register_allocator.dealloc(resume_kind);
        }

        IsObject::emit(self, value.variable());
        let skip_throw = self.jump_if_true(&value);

        self.register_allocator.dealloc(value);

        let error_msg = self.get_or_insert_literal(Literal::String(js_string!(
            "inner result was not an object"
        )));
        ThrowNewTypeError::emit(self, error_msg.into());

        self.patch_jump(skip_throw);
        self.patch_jump(early_exit);
    }

    /// Closes all active iterators in the current [`CallFrame`][crate::vm::CallFrame].
    pub(super) fn close_active_iterators(&mut self) {
        let start = self.next_opcode_location();

        let empty = self.register_allocator.alloc();
        IteratorStackEmpty::emit(self, empty.variable());
        let exit = self.jump_if_true(&empty);
        self.register_allocator.dealloc(empty);

        self.iterator_close(self.is_async_generator());
        Jump::emit(self, start);
        self.patch_jump(exit);
    }

    /// Yields from the current generator.
    ///
    /// This is equivalent to the [`Yield ( value )`][yield] operation from the spec.
    ///
    /// stack:
    /// - value **=>** received
    ///
    /// [yield]: https://tc39.es/ecma262/#sec-yield
    pub(super) fn r#yield(&mut self, value: &Register) {
        let resume_kind = self.register_allocator.alloc();

        // 1. Let generatorKind be GetGeneratorKind().
        if self.is_async() {
            // 2. If generatorKind is async, return ? AsyncGeneratorYield(? Await(value)).
            Await::emit(self, value.variable());
            self.pop_into_register(&resume_kind);
            self.pop_into_register(value);
            GeneratorNext::emit(self, resume_kind.variable(), value.variable());
            self.async_generator_yield(value, &resume_kind);
        } else {
            // 3. Otherwise, return ? GeneratorYield(CreateIterResultObject(value, false)).
            CreateIteratorResult::emit(self, value.variable(), false.into());
            GeneratorYield::emit(self, value.variable());
            self.pop_into_register(&resume_kind);
            self.pop_into_register(value);
        }

        GeneratorNext::emit(self, resume_kind.variable(), value.variable());
        self.register_allocator.dealloc(resume_kind);
    }

    /// Yields from the current async generator.
    ///
    /// This is equivalent to the [`AsyncGeneratorYield ( value )`][async_yield] operation from the spec.
    ///
    /// stack:
    /// - value **=>** `resume_kind`, received
    ///
    /// [async_yield]: https://tc39.es/ecma262/#sec-asyncgeneratoryield
    pub(super) fn async_generator_yield(&mut self, value: &Register, resume_kind: &Register) {
        AsyncGeneratorYield::emit(self, value.variable());
        self.pop_into_register(resume_kind);
        self.pop_into_register(value);

        let non_return_resume =
            self.jump_if_not_resume_kind(GeneratorResumeKind::Return, resume_kind);

        Await::emit(self, value.variable());
        self.pop_into_register(resume_kind);
        self.pop_into_register(value);

        let non_normal_resume =
            self.jump_if_not_resume_kind(GeneratorResumeKind::Normal, resume_kind);

        self.emit_resume_kind(GeneratorResumeKind::Return, resume_kind);

        self.patch_jump(non_normal_resume);
        self.patch_jump(non_return_resume);
    }
}
