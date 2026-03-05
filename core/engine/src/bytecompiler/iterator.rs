use crate::{
    bytecompiler::{ByteCompiler, Register},
    js_string,
};

impl ByteCompiler<'_> {
    /// Compiles the `IteratorNext` instruction.
    ///
    /// If `pop_on_error` is true, it will additionally pop the
    /// iterator from the iterator stack if any error occurs.
    pub(crate) fn iterator_next(&mut self, pop_on_error: bool) {
        let ex = self.register_allocator.alloc();
        let needs_throw = self.register_allocator.alloc();
        self.bytecode.emit_iterator_next();
        self.bytecode
            .emit_maybe_exception(needs_throw.variable(), ex.variable());
        let skip = self.jump_if_false(&needs_throw);
        if pop_on_error {
            self.bytecode
                .emit_iterator_pop(needs_throw.variable(), needs_throw.variable());
        }
        self.bytecode.emit_throw(ex.variable());
        self.patch_jump(skip);
        self.register_allocator.dealloc(ex);
        self.register_allocator.dealloc(needs_throw);
    }

    /// Compiles the `IteratorValue` instruction.
    ///
    /// If `pop_on_error` is true, it will additionally pop the
    /// iterator from the iterator stack if any error occurs.
    pub(crate) fn iterator_value(&mut self, dst: &Register, pop_on_error: bool) {
        let ex = self.register_allocator.alloc();
        let needs_throw = self.register_allocator.alloc();
        self.bytecode.emit_iterator_value(dst.variable());
        self.bytecode
            .emit_maybe_exception(needs_throw.variable(), ex.variable());
        let skip = self.jump_if_false(&needs_throw);
        if pop_on_error {
            self.bytecode
                .emit_iterator_pop(needs_throw.variable(), needs_throw.variable());
        }
        self.bytecode.emit_throw(ex.variable());
        self.patch_jump(skip);
        self.register_allocator.dealloc(ex);
        self.register_allocator.dealloc(needs_throw);
    }

    /// Compiles the `IteratorToArray` instruction.
    ///
    /// The active iterator will be used as the input iterator, so the caller
    /// needs to make sure that the iterator will be pushed before calling this.
    pub(crate) fn iterator_to_array(&mut self, array: &Register) {
        let temp = self.register_allocator.alloc();
        self.bytecode.emit_push_new_array(array.variable());

        let loop_ = self.next_opcode_location();
        self.push_loop_control_info(None, loop_, false);

        self.bytecode.emit_increment_loop_iteration();

        self.iterator_next(false);

        self.bytecode.emit_iterator_done(temp.variable());
        let end = self.jump_if_true(&temp);

        self.iterator_value(&temp, false);
        self.bytecode
            .emit_push_value_to_array(temp.variable(), array.variable());

        self.bytecode.emit_jump(loop_);
        self.pop_loop_control_info();

        self.patch_jump(end);
        self.register_allocator.dealloc(temp);
    }

    /// Compiles the `IteratorReturn` instruction.
    pub(crate) fn iterator_return(&mut self, value: &Register, called: &Register) {
        let temp = self.register_allocator.alloc();
        let temp2 = self.register_allocator.alloc();

        // preemtively push false and only push true when we are sure `return`
        // was called.
        self.bytecode.emit_push_false(called.variable());

        self.bytecode.emit_iterator_stack_empty(temp.variable());
        let no_iterators = self.jump_if_true(&temp);

        self.bytecode.emit_iterator_done(temp.variable());
        self.bytecode
            .emit_iterator_pop(value.variable(), temp2.variable());
        self.register_allocator.dealloc(temp2);
        let done = self.jump_if_true(&temp);

        let name_index = self.get_or_insert_string(js_string!("return"));
        self.bytecode.emit_move(temp.variable(), value.variable());
        self.bytecode
            .emit_get_method(temp.variable(), name_index.into());
        let no_return = self.jump_if_null_or_undefined(&temp);

        self.bytecode.emit_push_from_register(value.variable());
        self.bytecode.emit_push_from_register(temp.variable());
        self.register_allocator.dealloc(temp);

        // Need to recover the original return value. Otherwise, the call
        // could override the return register and we won't be able to return
        // using the correct value.
        self.bytecode
            .emit_set_register_from_accumulator(value.variable());
        self.bytecode.emit_call(0u8.into());
        self.bytecode.emit_set_accumulator(value.variable());
        self.bytecode.emit_pop_into_register(value.variable());
        self.bytecode.emit_push_true(called.variable());

        self.patch_jump(no_iterators);
        self.patch_jump(done);
        self.patch_jump(no_return);
    }
}
