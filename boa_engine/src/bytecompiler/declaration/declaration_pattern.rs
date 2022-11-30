use boa_ast::{
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    property::PropertyName,
};

use crate::{
    bytecompiler::{Access, ByteCompiler, Literal},
    vm::{BindingOpcode, Opcode},
    JsResult,
};

pub(crate) fn compile_declaration_pattern_impl(
    byte_compiler: &mut ByteCompiler<'_>,
    pattern: &Pattern,
    def: BindingOpcode,
) -> JsResult<()> {
    match pattern {
        Pattern::Object(pattern) => {
            byte_compiler.emit_opcode(Opcode::ValueNotNullOrUndefined);

            byte_compiler.emit_opcode(Opcode::RequireObjectCoercible);

            let mut additional_excluded_keys_count = 0;
            let rest_exits = pattern.has_rest();

            for binding in pattern.bindings() {
                use ObjectPatternElement::{
                    AssignmentPropertyAccess, AssignmentRestPropertyAccess, Pattern, RestProperty,
                    SingleName,
                };

                match binding {
                    //  SingleNameBinding : BindingIdentifier Initializer[opt]
                    SingleName {
                        ident,
                        name,
                        default_init,
                    } => {
                        byte_compiler.emit_opcode(Opcode::Dup);
                        match name {
                            PropertyName::Literal(name) => {
                                let index = byte_compiler.get_or_insert_name((*name).into());
                                byte_compiler.emit(Opcode::GetPropertyByName, &[index]);
                            }
                            PropertyName::Computed(node) => {
                                byte_compiler.compile_expr(node, true)?;
                                if rest_exits {
                                    byte_compiler.emit_opcode(Opcode::GetPropertyByValuePush);
                                } else {
                                    byte_compiler.emit_opcode(Opcode::GetPropertyByValue);
                                }
                            }
                        }

                        if let Some(init) = default_init {
                            let skip =
                                byte_compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                            byte_compiler.compile_expr(init, true)?;
                            byte_compiler.patch_jump(skip);
                        }
                        byte_compiler.emit_binding(def, *ident);

                        if rest_exits && name.computed().is_some() {
                            byte_compiler.emit_opcode(Opcode::Swap);
                            additional_excluded_keys_count += 1;
                        }
                    }
                    //  BindingRestProperty : ... BindingIdentifier
                    RestProperty {
                        ident,
                        excluded_keys,
                    } => {
                        byte_compiler.emit_opcode(Opcode::PushEmptyObject);

                        for key in excluded_keys {
                            byte_compiler.emit_push_literal(Literal::String(
                                byte_compiler
                                    .interner()
                                    .resolve_expect(key.sym())
                                    .into_common(false),
                            ));
                        }

                        byte_compiler.emit(
                            Opcode::CopyDataProperties,
                            &[excluded_keys.len() as u32, additional_excluded_keys_count],
                        );
                        byte_compiler.emit_binding(def, *ident);
                    }
                    AssignmentRestPropertyAccess {
                        access,
                        excluded_keys,
                    } => {
                        byte_compiler.emit_opcode(Opcode::Dup);
                        byte_compiler.emit_opcode(Opcode::PushEmptyObject);
                        for key in excluded_keys {
                            byte_compiler.emit_push_literal(Literal::String(
                                byte_compiler
                                    .interner()
                                    .resolve_expect(key.sym())
                                    .into_common(false),
                            ));
                        }
                        byte_compiler
                            .emit(Opcode::CopyDataProperties, &[excluded_keys.len() as u32, 0]);
                        byte_compiler.access_set(
                            Access::Property { access },
                            false,
                            ByteCompiler::access_set_top_of_stack_expr_fn,
                        )?;
                    }
                    AssignmentPropertyAccess {
                        name,
                        access,
                        default_init,
                    } => {
                        byte_compiler.emit_opcode(Opcode::Dup);
                        match name {
                            PropertyName::Literal(name) => {
                                let index = byte_compiler.get_or_insert_name((*name).into());
                                byte_compiler.emit(Opcode::GetPropertyByName, &[index]);
                            }
                            PropertyName::Computed(node) => {
                                byte_compiler.compile_expr(node, true)?;
                                if rest_exits {
                                    byte_compiler.emit_opcode(Opcode::GetPropertyByValuePush);
                                } else {
                                    byte_compiler.emit_opcode(Opcode::GetPropertyByValue);
                                }
                            }
                        }

                        if let Some(init) = default_init {
                            let skip =
                                byte_compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                            byte_compiler.compile_expr(init, true)?;
                            byte_compiler.patch_jump(skip);
                        }

                        byte_compiler.access_set(
                            Access::Property { access },
                            false,
                            ByteCompiler::access_set_top_of_stack_expr_fn,
                        )?;

                        if rest_exits && name.computed().is_some() {
                            byte_compiler.emit_opcode(Opcode::Swap);
                            additional_excluded_keys_count += 1;
                        }
                    }
                    Pattern {
                        name,
                        pattern,
                        default_init,
                    } => {
                        byte_compiler.emit_opcode(Opcode::Dup);
                        match name {
                            PropertyName::Literal(name) => {
                                let index = byte_compiler.get_or_insert_name((*name).into());
                                byte_compiler.emit(Opcode::GetPropertyByName, &[index]);
                            }
                            PropertyName::Computed(node) => {
                                byte_compiler.compile_expr(node, true)?;
                                byte_compiler.emit_opcode(Opcode::GetPropertyByValue);
                            }
                        }

                        if let Some(init) = default_init {
                            let skip =
                                byte_compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                            byte_compiler.compile_expr(init, true)?;
                            byte_compiler.patch_jump(skip);
                        }

                        byte_compiler.compile_declaration_pattern(pattern, def)?;
                    }
                }
            }

            if !rest_exits {
                byte_compiler.emit_opcode(Opcode::Pop);
            }
        }
        Pattern::Array(pattern) => {
            byte_compiler.emit_opcode(Opcode::ValueNotNullOrUndefined);
            byte_compiler.emit_opcode(Opcode::InitIterator);

            for binding in pattern.bindings().iter() {
                use ArrayPatternElement::{
                    Elision, Pattern, PatternRest, PropertyAccess, PropertyAccessRest, SingleName,
                    SingleNameRest,
                };

                match binding {
                    // ArrayBindingPattern : [ Elision ]
                    Elision => {
                        byte_compiler.emit_opcode(Opcode::IteratorNext);
                        byte_compiler.emit_opcode(Opcode::Pop);
                    }
                    // SingleNameBinding : BindingIdentifier Initializer[opt]
                    SingleName {
                        ident,
                        default_init,
                    } => {
                        byte_compiler.emit_opcode(Opcode::IteratorNext);
                        if let Some(init) = default_init {
                            let skip =
                                byte_compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                            byte_compiler.compile_expr(init, true)?;
                            byte_compiler.patch_jump(skip);
                        }
                        byte_compiler.emit_binding(def, *ident);
                    }
                    PropertyAccess { access } => {
                        byte_compiler.emit_opcode(Opcode::IteratorNext);
                        byte_compiler.access_set(
                            Access::Property { access },
                            false,
                            ByteCompiler::access_set_top_of_stack_expr_fn,
                        )?;
                    }
                    // BindingElement : BindingPattern Initializer[opt]
                    Pattern {
                        pattern,
                        default_init,
                    } => {
                        byte_compiler.emit_opcode(Opcode::IteratorNext);

                        if let Some(init) = default_init {
                            let skip =
                                byte_compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                            byte_compiler.compile_expr(init, true)?;
                            byte_compiler.patch_jump(skip);
                        }

                        byte_compiler.compile_declaration_pattern(pattern, def)?;
                    }
                    // BindingRestElement : ... BindingIdentifier
                    SingleNameRest { ident } => {
                        byte_compiler.emit_opcode(Opcode::IteratorToArray);
                        byte_compiler.emit_binding(def, *ident);
                    }
                    PropertyAccessRest { access } => {
                        byte_compiler.emit_opcode(Opcode::IteratorToArray);
                        byte_compiler.access_set(
                            Access::Property { access },
                            false,
                            ByteCompiler::access_set_top_of_stack_expr_fn,
                        )?;
                    }
                    // BindingRestElement : ... BindingPattern
                    PatternRest { pattern } => {
                        byte_compiler.emit_opcode(Opcode::IteratorToArray);
                        byte_compiler.compile_declaration_pattern(pattern, def)?;
                    }
                }
            }

            byte_compiler.emit_opcode(Opcode::IteratorClose);
        }
    }
    Ok(())
}
