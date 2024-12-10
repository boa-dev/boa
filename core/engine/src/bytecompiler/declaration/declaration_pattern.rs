use crate::{
    bytecompiler::{Access, ByteCompiler, Literal, Operand, Register, ToJsString},
    vm::{BindingOpcode, Opcode},
};
use boa_ast::{
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    property::PropertyName,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_declaration_pattern_impl(
        &mut self,
        pattern: &Pattern,
        def: BindingOpcode,
        object: &Register,
    ) {
        match pattern {
            Pattern::Object(pattern) => {
                self.emit(
                    Opcode::ValueNotNullOrUndefined,
                    &[Operand::Register(object)],
                );

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
                                    self.emit_get_property_by_name(&dst, object, object, *ident);
                                    let key = self.register_allocator.alloc();
                                    self.emit_push_literal(
                                        Literal::String(
                                            self.interner()
                                                .resolve_expect(*ident)
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
                                        self.emit(
                                            Opcode::GetPropertyByValuePush,
                                            &[
                                                Operand::Register(&dst),
                                                Operand::Register(&key),
                                                Operand::Register(object),
                                                Operand::Register(object),
                                            ],
                                        );
                                        excluded_keys_registers.push(key);
                                    } else {
                                        self.emit(
                                            Opcode::GetPropertyByValue,
                                            &[
                                                Operand::Register(&dst),
                                                Operand::Register(&key),
                                                Operand::Register(object),
                                                Operand::Register(object),
                                            ],
                                        );
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
                            self.emit(Opcode::PushEmptyObject, &[Operand::Register(&value)]);
                            let mut args = Vec::from([
                                Operand::Register(&value),
                                Operand::Register(object),
                                Operand::Varying(excluded_keys_registers.len() as u32),
                            ]);
                            for r in &excluded_keys_registers {
                                args.push(Operand::Register(r));
                            }
                            self.emit(Opcode::CopyDataProperties, &args);
                            while let Some(r) = excluded_keys_registers.pop() {
                                self.register_allocator.dealloc(r);
                            }
                            self.emit_binding(def, ident.to_js_string(self.interner()), &value);
                            self.register_allocator.dealloc(value);
                        }
                        AssignmentRestPropertyAccess { access } => {
                            let value = self.register_allocator.alloc();
                            self.emit(Opcode::PushEmptyObject, &[Operand::Register(&value)]);
                            let mut args = Vec::from([
                                Operand::Register(&value),
                                Operand::Register(object),
                                Operand::Varying(excluded_keys_registers.len() as u32),
                            ]);
                            for r in &excluded_keys_registers {
                                args.push(Operand::Register(r));
                            }
                            self.emit(Opcode::CopyDataProperties, &args);
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
                                                .resolve_expect(*ident)
                                                .into_common(false),
                                        ),
                                        &key,
                                    );
                                    excluded_keys_registers.push(key);
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, &key);
                                    self.emit(
                                        Opcode::ToPropertyKey,
                                        &[Operand::Register(&key), Operand::Register(&key)],
                                    );
                                }
                            }

                            let dst = self.register_allocator.alloc();
                            self.access_set(
                                Access::Property { access },
                                |compiler: &mut ByteCompiler<'_>| {
                                    match name {
                                        PropertyName::Literal(ident) => {
                                            compiler.emit_get_property_by_name(
                                                &dst, object, object, *ident,
                                            );
                                            compiler.register_allocator.dealloc(key);
                                        }
                                        PropertyName::Computed(_) => {
                                            if rest_exits {
                                                compiler.emit(
                                                    Opcode::GetPropertyByValuePush,
                                                    &[
                                                        Operand::Register(&dst),
                                                        Operand::Register(&key),
                                                        Operand::Register(object),
                                                        Operand::Register(object),
                                                    ],
                                                );
                                                excluded_keys_registers.push(key);
                                            } else {
                                                compiler.emit(
                                                    Opcode::GetPropertyByValue,
                                                    &[
                                                        Operand::Register(&dst),
                                                        Operand::Register(&key),
                                                        Operand::Register(object),
                                                        Operand::Register(object),
                                                    ],
                                                );
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
                                    self.emit_get_property_by_name(&dst, object, object, *ident);
                                }
                                PropertyName::Computed(node) => {
                                    let key = self.register_allocator.alloc();
                                    self.compile_expr(node, &key);
                                    self.emit(
                                        Opcode::GetPropertyByValue,
                                        &[
                                            Operand::Register(&dst),
                                            Operand::Register(&key),
                                            Operand::Register(object),
                                            Operand::Register(object),
                                        ],
                                    );
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
                self.emit(
                    Opcode::ValueNotNullOrUndefined,
                    &[Operand::Register(object)],
                );
                self.emit(Opcode::GetIterator, &[Operand::Register(object)]);

                let handler_index = self.push_handler();
                for element in pattern.bindings() {
                    self.compile_array_pattern_element(element, def);
                }

                let no_exception_thrown = self.jump();
                self.patch_handler(handler_index);

                let has_exception = self.register_allocator.alloc();
                let exception = self.register_allocator.alloc();
                self.emit(
                    Opcode::MaybeException,
                    &[
                        Operand::Register(&has_exception),
                        Operand::Register(&exception),
                    ],
                );

                let iterator_close_handler = self.push_handler();
                self.iterator_close(false);
                self.patch_handler(iterator_close_handler);

                let jump = self.jump_if_false(&has_exception);
                self.register_allocator.dealloc(has_exception);

                self.emit(Opcode::Throw, &[Operand::Register(&exception)]);
                self.register_allocator.dealloc(exception);

                self.patch_jump(jump);
                self.emit_opcode(Opcode::ReThrow);

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
                self.emit_opcode(Opcode::IteratorNext);
            }
            // SingleNameBinding : BindingIdentifier Initializer[opt]
            SingleName {
                ident,
                default_init,
            } => {
                self.emit_opcode(Opcode::IteratorNext);
                let value = self.register_allocator.alloc();
                self.emit(Opcode::IteratorDone, &[Operand::Register(&value)]);
                let done = self.jump_if_true(&value);
                self.emit(Opcode::IteratorValue, &[Operand::Register(&value)]);
                let skip_push = self.jump();
                self.patch_jump(done);
                self.push_undefined(&value);
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
                    compiler.emit_opcode(Opcode::IteratorNext);
                    compiler.emit(Opcode::IteratorDone, &[Operand::Register(&value)]);
                    let done = compiler.jump_if_true(&value);
                    compiler.emit(Opcode::IteratorValue, &[Operand::Register(&value)]);
                    let skip_push = compiler.jump();
                    compiler.patch_jump(done);
                    compiler.push_undefined(&value);
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
                self.emit_opcode(Opcode::IteratorNext);
                let value = self.register_allocator.alloc();
                self.emit(Opcode::IteratorDone, &[Operand::Register(&value)]);
                let done = self.jump_if_true(&value);
                self.emit(Opcode::IteratorValue, &[Operand::Register(&value)]);
                let skip_push = self.jump();
                self.patch_jump(done);
                self.push_undefined(&value);
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
                self.emit(Opcode::IteratorToArray, &[Operand::Register(&value)]);
                self.emit_binding(def, ident.to_js_string(self.interner()), &value);
                self.register_allocator.dealloc(value);
            }
            PropertyAccessRest { access } => {
                let value = self.register_allocator.alloc();
                self.access_set(Access::Property { access }, |compiler| {
                    compiler.emit(Opcode::IteratorToArray, &[Operand::Register(&value)]);
                    &value
                });
                self.register_allocator.dealloc(value);
            }
            // BindingRestElement : ... BindingPattern
            PatternRest { pattern } => {
                let value = self.register_allocator.alloc();
                self.emit(Opcode::IteratorToArray, &[Operand::Register(&value)]);
                self.compile_declaration_pattern(pattern, def, &value);
                self.register_allocator.dealloc(value);
            }
        }
    }
}
