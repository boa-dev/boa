use crate::{
    bytecompiler::{Access, ByteCompiler, Literal},
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
                                    self.emit(Opcode::GetPropertyByName, &[index]);
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
                                &[excluded_keys.len() as u32, additional_excluded_keys_count],
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
                            self.emit(Opcode::CopyDataProperties, &[excluded_keys.len() as u32, 0]);
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
                                    self.emit(Opcode::GetPropertyByName, &[index]);
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
                                    self.emit(Opcode::GetPropertyByName, &[index]);
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

                for element in pattern.bindings() {
                    self.compile_array_pattern_element(element, def);
                }

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
                self.emit_opcode(Opcode::IteratorValue);
                if let Some(init) = default_init {
                    let skip = self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                    self.compile_expr(init, true);
                    self.patch_jump(skip);
                }
                self.emit_binding(def, *ident);
            }
            PropertyAccess { access } => {
                self.access_set(Access::Property { access }, false, |compiler, _level| {
                    compiler.emit_opcode(Opcode::IteratorNext);
                    compiler.emit_opcode(Opcode::IteratorValue);
                });
            }
            // BindingElement : BindingPattern Initializer[opt]
            Pattern {
                pattern,
                default_init,
            } => {
                self.emit_opcode(Opcode::IteratorNext);
                self.emit_opcode(Opcode::IteratorValue);

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
