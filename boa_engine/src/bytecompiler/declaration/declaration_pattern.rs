use crate::{
    bytecompiler::{Access, ByteCompiler, Literal, Operand},
    vm::{BindingOpcode, Opcode},
};
use boa_ast::{
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    property::PropertyName,
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_declaration_pattern_impl(
        &mut self,
        pattern: &Pattern,
        def: BindingOpcode,
    ) {
        match pattern {
            Pattern::Object(pattern) => {
                self.emit_opcode(Opcode::ValueNotNullOrUndefined);

                self.emit_opcode(Opcode::RequireObjectCoercible);

                let mut additional_excluded_keys_count = 0;
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
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);
                            match name {
                                PropertyName::Literal(name) => {
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit_with_varying_operand(
                                        Opcode::GetPropertyByName,
                                        index,
                                    );
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, true);
                                    if rest_exits {
                                        self.emit_opcode(Opcode::GetPropertyByValuePush);
                                    } else {
                                        self.emit_opcode(Opcode::GetPropertyByValue);
                                    }
                                }
                            }

                            if let Some(init) = default_init {
                                let skip =
                                    self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true);
                                self.patch_jump(skip);
                            }
                            self.emit_binding(def, *ident);

                            if rest_exits && name.computed().is_some() {
                                self.emit_opcode(Opcode::Swap);
                                additional_excluded_keys_count += 1;
                            }
                        }
                        //  BindingRestProperty : ... BindingIdentifier
                        RestProperty {
                            ident,
                            excluded_keys,
                        } => {
                            self.emit_opcode(Opcode::PushEmptyObject);

                            for key in excluded_keys {
                                self.emit_push_literal(Literal::String(
                                    self.interner().resolve_expect(key.sym()).into_common(false),
                                ));
                            }

                            self.emit(
                                Opcode::CopyDataProperties,
                                &[
                                    Operand::Varying(excluded_keys.len() as u32),
                                    Operand::Varying(additional_excluded_keys_count),
                                ],
                            );
                            self.emit_binding(def, *ident);
                        }
                        AssignmentRestPropertyAccess {
                            access,
                            excluded_keys,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::PushEmptyObject);
                            for key in excluded_keys {
                                self.emit_push_literal(Literal::String(
                                    self.interner().resolve_expect(key.sym()).into_common(false),
                                ));
                            }
                            self.emit(
                                Opcode::CopyDataProperties,
                                &[
                                    Operand::Varying(excluded_keys.len() as u32),
                                    Operand::Varying(0),
                                ],
                            );
                            self.access_set(
                                Access::Property { access },
                                false,
                                ByteCompiler::access_set_top_of_stack_expr_fn,
                            );
                        }
                        AssignmentPropertyAccess {
                            name,
                            access,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);
                            match name {
                                PropertyName::Literal(name) => {
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit_with_varying_operand(
                                        Opcode::GetPropertyByName,
                                        index,
                                    );
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, true);
                                    if rest_exits {
                                        self.emit_opcode(Opcode::GetPropertyByValuePush);
                                    } else {
                                        self.emit_opcode(Opcode::GetPropertyByValue);
                                    }
                                }
                            }

                            if let Some(init) = default_init {
                                let skip =
                                    self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true);
                                self.patch_jump(skip);
                            }

                            self.access_set(
                                Access::Property { access },
                                false,
                                ByteCompiler::access_set_top_of_stack_expr_fn,
                            );

                            if rest_exits && name.computed().is_some() {
                                self.emit_opcode(Opcode::Swap);
                                additional_excluded_keys_count += 1;
                            }
                        }
                        Pattern {
                            name,
                            pattern,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);
                            match name {
                                PropertyName::Literal(name) => {
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit_with_varying_operand(
                                        Opcode::GetPropertyByName,
                                        index,
                                    );
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, true);
                                    self.emit_opcode(Opcode::GetPropertyByValue);
                                }
                            }

                            if let Some(init) = default_init {
                                let skip =
                                    self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true);
                                self.patch_jump(skip);
                            }

                            self.compile_declaration_pattern(pattern, def);
                        }
                    }
                }

                if !rest_exits {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Pattern::Array(pattern) => {
                self.emit_opcode(Opcode::ValueNotNullOrUndefined);
                self.emit_opcode(Opcode::GetIterator);

                let handler_index = self.push_handler();
                for element in pattern.bindings() {
                    self.compile_array_pattern_element(element, def);
                }

                let no_exception_thrown = self.jump();
                self.patch_handler(handler_index);
                self.emit_opcode(Opcode::MaybeException);

                // stack: hasPending, exception?

                self.current_stack_value_count += 2;
                let iterator_close_handler = self.push_handler();
                self.iterator_close(false);
                self.patch_handler(iterator_close_handler);
                self.current_stack_value_count -= 2;

                let jump = self.jump_if_false();
                self.emit_opcode(Opcode::Throw);
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
                self.emit_opcode(Opcode::IteratorNextWithoutPop);
            }
            // SingleNameBinding : BindingIdentifier Initializer[opt]
            SingleName {
                ident,
                default_init,
            } => {
                self.emit_opcode(Opcode::IteratorNextWithoutPop);
                self.emit_opcode(Opcode::IteratorValueWithoutPop);
                if let Some(init) = default_init {
                    let skip = self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                    self.compile_expr(init, true);
                    self.patch_jump(skip);
                }
                self.emit_binding(def, *ident);
            }
            PropertyAccess { access } => {
                self.access_set(Access::Property { access }, false, |compiler, _level| {
                    compiler.emit_opcode(Opcode::IteratorNextWithoutPop);
                    compiler.emit_opcode(Opcode::IteratorValueWithoutPop);
                });
            }
            // BindingElement : BindingPattern Initializer[opt]
            Pattern {
                pattern,
                default_init,
            } => {
                self.emit_opcode(Opcode::IteratorNextWithoutPop);
                self.emit_opcode(Opcode::IteratorValueWithoutPop);

                if let Some(init) = default_init {
                    let skip = self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                    self.compile_expr(init, true);
                    self.patch_jump(skip);
                }

                self.compile_declaration_pattern(pattern, def);
            }
            // BindingRestElement : ... BindingIdentifier
            SingleNameRest { ident } => {
                self.emit_opcode(Opcode::IteratorToArray);
                self.emit_binding(def, *ident);
            }
            PropertyAccessRest { access } => {
                self.access_set(Access::Property { access }, false, |compiler, _level| {
                    compiler.emit_opcode(Opcode::IteratorToArray);
                });
            }
            // BindingRestElement : ... BindingPattern
            PatternRest { pattern } => {
                self.emit_opcode(Opcode::IteratorToArray);
                self.compile_declaration_pattern(pattern, def);
            }
        }
    }
}
