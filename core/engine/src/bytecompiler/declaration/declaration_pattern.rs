use crate::vm::opcode::*;
use crate::{
    bytecompiler::{Access, ByteCompiler, Literal, Register, ToJsString},
    vm::opcode::BindingOpcode,
};
use boa_ast::{
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    property::PropertyName,
};
use thin_vec::ThinVec;

impl ByteCompiler<'_> {
    pub(crate) fn compile_declaration_pattern_impl(
        &mut self,
        pattern: &Pattern,
        def: BindingOpcode,
        object: &Register,
    ) {
        match pattern {
            Pattern::Object(pattern) => {
                ValueNotNullOrUndefined::emit(self, object.variable());

                let mut excluded_keys_registers = Vec::new();
                let rest_exits = pattern.has_rest();

                for binding in pattern.bindings() {
                    use ObjectPatternElement::{
                        AssignmentPropertyAccess, AssignmentRestPropertyAccess, Pattern,
                        RestProperty, SingleName,
                    };

                    match binding {
                        //  SingleNameBinding : BindingIdentifier Initializer[opt]
                        SingleName {
                            ident,
                            name,
                            default_init,
                        } => {
                            let dst = self.register_allocator.alloc();

                            match name {
                                PropertyName::Literal(ident) => {
                                    self.emit_get_property_by_name(&dst, None, object, ident.sym());
                                    let key = self.register_allocator.alloc();
                                    self.emit_push_literal(
                                        Literal::String(
                                            self.interner()
                                                .resolve_expect(ident.sym())
                                                .into_common(false),
                                        ),
                                        &key,
                                    );
                                    excluded_keys_registers.push(key);
                                }
                                PropertyName::Computed(node) => {
                                    let key = self.register_allocator.alloc();
                                    self.compile_expr(node, &key);
                                    if rest_exits {
                                        GetPropertyByValuePush::emit(self, dst.variable(),
                                            key.variable(),
                                            object.variable(),
                                            object.variable(),);
                                        excluded_keys_registers.push(key);
                                    } else {
                                        GetPropertyByValue::emit(self, dst.variable(),
                                            key.variable(),
                                            object.variable(),
                                            object.variable(),);
                                        self.register_allocator.dealloc(key);
                                    }
                                }
                            }

                            if let Some(init) = default_init {
                                let skip = self.emit_jump_if_not_undefined(&dst);
                                self.compile_expr(init, &dst);
                                self.patch_jump(skip);
                            }

                            self.emit_binding(def, ident.to_js_string(self.interner()), &dst);
                            self.register_allocator.dealloc(dst);
                        }
                        //  BindingRestProperty : ... BindingIdentifier
                        RestProperty { ident } => {
                            let value = self.register_allocator.alloc();
                            PushEmptyObject::emit(self, value.variable());
                            let mut excluded_keys =
                                ThinVec::with_capacity(excluded_keys_registers.len());
                            for r in &excluded_keys_registers {
                                excluded_keys.push(r.variable());
                            }
                            CopyDataProperties::emit(self, value.variable(),
                                object.variable(),
                                excluded_keys,);
                            while let Some(r) = excluded_keys_registers.pop() {
                                self.register_allocator.dealloc(r);
                            }
                            self.emit_binding(def, ident.to_js_string(self.interner()), &value);
                            self.register_allocator.dealloc(value);
                        }
                        AssignmentRestPropertyAccess { access } => {
                            let value = self.register_allocator.alloc();
                            PushEmptyObject::emit(self, value.variable());
                            let mut excluded_keys =
                                ThinVec::with_capacity(excluded_keys_registers.len());
                            for r in &excluded_keys_registers {
                                excluded_keys.push(r.variable());
                            }
                            CopyDataProperties::emit(self, value.variable(),
                                object.variable(),
                                excluded_keys,);
                            while let Some(r) = excluded_keys_registers.pop() {
                                self.register_allocator.dealloc(r);
                            }
                            self.access_set(Access::Property { access }, |_| &value);
                            self.register_allocator.dealloc(value);
                        }
                        AssignmentPropertyAccess {
                            name,
                            access,
                            default_init,
                        } => {
                            let key = self.register_allocator.alloc();
                            match &name {
                                PropertyName::Literal(ident) => {
                                    let key = self.register_allocator.alloc();
                                    self.emit_push_literal(
                                        Literal::String(
                                            self.interner()
                                                .resolve_expect(ident.sym())
                                                .into_common(false),
                                        ),
                                        &key,
                                    );
                                    excluded_keys_registers.push(key);
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, &key);
                                    ToPropertyKey::emit(self, key.variable(), key.variable());
                                }
                            }

                            let dst = self.register_allocator.alloc();
                            self.access_set(
                                Access::Property { access },
                                |compiler: &mut ByteCompiler<'_>| {
                                    match name {
                                        PropertyName::Literal(ident) => {
                                            compiler.emit_get_property_by_name(
                                                &dst,
                                                None,
                                                object,
                                                ident.sym(),
                                            );
                                            compiler.register_allocator.dealloc(key);
                                        }
                                        PropertyName::Computed(_) => {
                                            if rest_exits {
                                                GetPropertyByValuePush::emit(&mut *compiler, dst.variable(),
                                                    key.variable(),
                                                    object.variable(),
                                                    object.variable(),);
                                                excluded_keys_registers.push(key);
                                            } else {
                                                GetPropertyByValue::emit(&mut *compiler, dst.variable(),
                                                    key.variable(),
                                                    object.variable(),
                                                    object.variable(),);
                                                compiler.register_allocator.dealloc(key);
                                            }
                                        }
                                    }

                                    if let Some(init) = default_init {
                                        let skip = compiler.emit_jump_if_not_undefined(&dst);
                                        compiler.compile_expr(init, &dst);
                                        compiler.patch_jump(skip);
                                    }

                                    &dst
                                },
                            );
                            self.register_allocator.dealloc(dst);
                        }
                        Pattern {
                            name,
                            pattern,
                            default_init,
                        } => {
                            let dst = self.register_allocator.alloc();

                            match name {
                                PropertyName::Literal(ident) => {
                                    self.emit_get_property_by_name(&dst, None, object, ident.sym());
                                }
                                PropertyName::Computed(node) => {
                                    let key = self.register_allocator.alloc();
                                    self.compile_expr(node, &key);
                                    GetPropertyByValue::emit(self, dst.variable(),
                                        key.variable(),
                                        object.variable(),
                                        object.variable(),);
                                    self.register_allocator.dealloc(key);
                                }
                            }

                            if let Some(init) = default_init {
                                let skip = self.emit_jump_if_not_undefined(&dst);
                                self.compile_expr(init, &dst);
                                self.patch_jump(skip);
                            }
                            self.compile_declaration_pattern(pattern, def, &dst);
                            self.register_allocator.dealloc(dst);
                        }
                    }
                }

                while let Some(r) = excluded_keys_registers.pop() {
                    self.register_allocator.dealloc(r);
                }
            }
            Pattern::Array(pattern) => {
                ValueNotNullOrUndefined::emit(self, object.variable());
                GetIterator::emit(self, object.variable());

                let handler_index = self.push_handler();
                for element in pattern.bindings() {
                    self.compile_array_pattern_element(element, def);
                }

                let no_exception_thrown = self.jump();
                self.patch_handler(handler_index);

                let has_exception = self.register_allocator.alloc();
                let exception = self.register_allocator.alloc();
                MaybeException::emit(self, has_exception.variable(), exception.variable());

                let iterator_close_handler = self.push_handler();
                self.iterator_close(false);
                self.patch_handler(iterator_close_handler);

                let jump = self.jump_if_false(&has_exception);
                self.register_allocator.dealloc(has_exception);

                Throw::emit(self, exception.variable());
                self.register_allocator.dealloc(exception);

                self.patch_jump(jump);
                ReThrow::emit(self);

                self.patch_jump(no_exception_thrown);

                self.iterator_close(false);
            }
        }
    }

    fn compile_array_pattern_element(&mut self, element: &ArrayPatternElement, def: BindingOpcode) {
        use ArrayPatternElement::{
            Elision, Pattern, PatternRest, PropertyAccess, PropertyAccessRest, SingleName,
            SingleNameRest,
        };

        match element {
            // ArrayBindingPattern : [ Elision ]
            Elision => {
                IteratorNext::emit(self);
            }
            // SingleNameBinding : BindingIdentifier Initializer[opt]
            SingleName {
                ident,
                default_init,
            } => {
                IteratorNext::emit(self);
                let value = self.register_allocator.alloc();
                IteratorDone::emit(self, value.variable());
                let done = self.jump_if_true(&value);
                IteratorValue::emit(self, value.variable());
                let skip_push = self.jump();
                self.patch_jump(done);
                PushUndefined::emit(self, value.variable());
                self.patch_jump(skip_push);

                if let Some(init) = default_init {
                    let skip = self.emit_jump_if_not_undefined(&value);
                    self.compile_expr(init, &value);
                    self.patch_jump(skip);
                }

                self.emit_binding(def, ident.to_js_string(self.interner()), &value);
                self.register_allocator.dealloc(value);
            }
            PropertyAccess {
                access,
                default_init,
            } => {
                let value = self.register_allocator.alloc();
                self.access_set(Access::Property { access }, |compiler| {
                    IteratorNext::emit(&mut *compiler);
                    IteratorDone::emit(&mut *compiler, value.variable());
                    let done = compiler.jump_if_true(&value);
                    IteratorValue::emit(&mut *compiler, value.variable());
                    let skip_push = compiler.jump();
                    compiler.patch_jump(done);
                    PushUndefined::emit(&mut *compiler, value.variable());
                    compiler.patch_jump(skip_push);

                    if let Some(init) = default_init {
                        let skip = compiler.emit_jump_if_not_undefined(&value);
                        compiler.compile_expr(init, &value);
                        compiler.patch_jump(skip);
                    }

                    &value
                });
                self.register_allocator.dealloc(value);
            }
            // BindingElement : BindingPattern Initializer[opt]
            Pattern {
                pattern,
                default_init,
            } => {
                IteratorNext::emit(self);
                let value = self.register_allocator.alloc();
                IteratorDone::emit(self, value.variable());
                let done = self.jump_if_true(&value);
                IteratorValue::emit(self, value.variable());
                let skip_push = self.jump();
                self.patch_jump(done);
                PushUndefined::emit(self, value.variable());
                self.patch_jump(skip_push);

                if let Some(init) = default_init {
                    let skip = self.emit_jump_if_not_undefined(&value);
                    self.compile_expr(init, &value);
                    self.patch_jump(skip);
                }
                self.compile_declaration_pattern(pattern, def, &value);
                self.register_allocator.dealloc(value);
            }
            // BindingRestElement : ... BindingIdentifier
            SingleNameRest { ident } => {
                let value = self.register_allocator.alloc();
                IteratorToArray::emit(self, value.variable());
                self.emit_binding(def, ident.to_js_string(self.interner()), &value);
                self.register_allocator.dealloc(value);
            }
            PropertyAccessRest { access } => {
                let value = self.register_allocator.alloc();
                self.access_set(Access::Property { access }, |compiler| {
                    IteratorToArray::emit(&mut *compiler, value.variable());
                    &value
                });
                self.register_allocator.dealloc(value);
            }
            // BindingRestElement : ... BindingPattern
            PatternRest { pattern } => {
                let value = self.register_allocator.alloc();
                IteratorToArray::emit(self, value.variable());
                self.compile_declaration_pattern(pattern, def, &value);
                self.register_allocator.dealloc(value);
            }
        }
    }
}
