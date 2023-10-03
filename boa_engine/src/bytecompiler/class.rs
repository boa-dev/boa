use super::{ByteCompiler, Literal, Operand};
use crate::vm::{BindingOpcode, CodeBlock, CodeBlockFlags, Opcode};
use boa_ast::{
    expression::Identifier,
    function::{Class, ClassElement, FormalParameterList},
    property::{MethodDefinition, PropertyName},
};
use boa_gc::Gc;
use boa_interner::Sym;

// Static class elements that are initialized at a later time in the class creation.
enum StaticElement {
    // A static class block with it's function code.
    StaticBlock(Gc<CodeBlock>),

    // A static class field with it's function code and optional name index.
    StaticField((Gc<CodeBlock>, Option<u32>)),
}

impl ByteCompiler<'_, '_> {
    /// This function compiles a class declaration or expression.
    ///
    /// The compilation of a class declaration and expression is mostly equal.
    /// A class declaration binds the resulting class object to it's identifier.
    /// A class expression leaves the resulting class object on the stack for following operations.
    pub(crate) fn compile_class(&mut self, class: &Class, expression: bool) {
        let class_name = class.name().map_or(Sym::EMPTY_STRING, Identifier::sym);

        let mut compiler = ByteCompiler::new(
            class_name,
            true,
            self.json_parse,
            self.current_environment.clone(),
            self.context,
        );

        if let Some(class_name) = class.name() {
            if class.has_binding_identifier() {
                compiler.code_block_flags |= CodeBlockFlags::HAS_BINDING_IDENTIFIER;
                compiler.push_compile_environment(false);
                compiler.create_immutable_binding(class_name, true);
            }
        }

        compiler.push_compile_environment(true);

        if let Some(expr) = class.constructor() {
            compiler.length = expr.parameters().length();
            compiler.params = expr.parameters().clone();

            let (env_label, _) = compiler.function_declaration_instantiation(
                expr.body(),
                expr.parameters(),
                false,
                true,
                false,
            );

            compiler.compile_statement_list(expr.body().statements(), false, false);

            let env_index = compiler.pop_compile_environment();

            if let Some(env_label) = env_label {
                compiler.patch_jump_with_target(env_label, env_index);
                compiler.pop_compile_environment();
            } else {
                compiler.code_block_flags |= CodeBlockFlags::IS_CLASS_CONSTRUCTOR;
            }

            compiler.emit_opcode(Opcode::PushUndefined);
        } else {
            if class.super_ref().is_some() {
                compiler.emit_opcode(Opcode::SuperCallDerived);
            } else {
                compiler.emit_opcode(Opcode::RestParameterPop);
                compiler.emit_opcode(Opcode::PushUndefined);
            }
            compiler.pop_compile_environment();
            compiler.code_block_flags |= CodeBlockFlags::IS_CLASS_CONSTRUCTOR;
        }
        compiler.emit_opcode(Opcode::SetReturnValue);

        if class.name().is_some() && class.has_binding_identifier() {
            compiler.pop_compile_environment();
        }

        let code = Gc::new(compiler.finish());
        let index = self.functions.len() as u32;
        self.functions.push(code);
        self.emit(
            Opcode::GetFunction,
            &[Operand::Varying(index), Operand::Bool(false)],
        );

        let class_env: Option<super::Label> = match class.name() {
            Some(name) if class.has_binding_identifier() => {
                self.push_compile_environment(false);
                self.create_immutable_binding(name, true);
                Some(self.emit_opcode_with_operand(Opcode::PushDeclarativeEnvironment))
            }
            _ => None,
        };

        self.emit_opcode(Opcode::Dup);
        if let Some(node) = class.super_ref() {
            self.compile_expr(node, true);
            self.emit_opcode(Opcode::PushClassPrototype);
        } else {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.emit_opcode(Opcode::SetClassPrototype);
        self.emit_opcode(Opcode::Swap);

        let count_label = self.emit_opcode_with_operand(Opcode::PushPrivateEnvironment);
        let mut count = 0;
        for element in class.elements() {
            match element {
                ClassElement::PrivateMethodDefinition(name, _)
                | ClassElement::PrivateStaticMethodDefinition(name, _)
                | ClassElement::PrivateFieldDefinition(name, _)
                | ClassElement::PrivateStaticFieldDefinition(name, _) => {
                    count += 1;
                    let index = self.get_or_insert_private_name(*name);
                    self.emit_u32(index);
                }
                _ => {}
            }
        }
        self.patch_jump_with_target(count_label, count);

        let mut static_elements = Vec::new();
        let mut static_field_name_count = 0;

        for element in class.elements() {
            match element {
                ClassElement::StaticMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassStaticGetterByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassStaticGetterByValue);
                            }
                        },
                        MethodDefinition::Set(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassStaticSetterByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassStaticSetterByValue);
                            }
                        },
                        MethodDefinition::Ordinary(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassStaticMethodByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassStaticMethodByValue);
                            }
                        },
                        MethodDefinition::Async(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassStaticMethodByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassStaticMethodByValue);
                            }
                        },
                        MethodDefinition::Generator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassStaticMethodByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassStaticMethodByValue);
                            }
                        },
                        MethodDefinition::AsyncGenerator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassStaticMethodByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassStaticMethodByValue);
                            }
                        },
                    }
                }
                ClassElement::PrivateStaticMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::SetPrivateGetter, index);
                        }
                        MethodDefinition::Set(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::SetPrivateSetter, index);
                        }
                        MethodDefinition::Ordinary(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::SetPrivateMethod, index);
                        }
                        MethodDefinition::Async(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::SetPrivateMethod, index);
                        }
                        MethodDefinition::Generator(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::SetPrivateMethod, index);
                        }
                        MethodDefinition::AsyncGenerator(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::SetPrivateMethod, index);
                        }
                    }
                }
                ClassElement::FieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    match name {
                        PropertyName::Literal(name) => {
                            self.emit_push_literal(Literal::String(
                                self.interner().resolve_expect(*name).into_common(false),
                            ));
                        }
                        PropertyName::Computed(name) => {
                            self.compile_expr(name, true);
                        }
                    }
                    let mut field_compiler = ByteCompiler::new(
                        Sym::EMPTY_STRING,
                        true,
                        self.json_parse,
                        self.current_environment.clone(),
                        self.context,
                    );
                    field_compiler.push_compile_environment(false);
                    field_compiler.create_immutable_binding(class_name.into(), true);
                    field_compiler.push_compile_environment(true);
                    if let Some(node) = field {
                        field_compiler.compile_expr(node, true);
                    } else {
                        field_compiler.emit_opcode(Opcode::PushUndefined);
                    }
                    field_compiler.emit_opcode(Opcode::SetReturnValue);

                    field_compiler.pop_compile_environment();
                    field_compiler.pop_compile_environment();

                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    let code = field_compiler.finish();
                    let code = Gc::new(code);
                    let index = self.functions.len() as u32;
                    self.functions.push(code);
                    self.emit(
                        Opcode::GetFunction,
                        &[Operand::Varying(index), Operand::Bool(false)],
                    );
                    self.emit_opcode(Opcode::PushClassField);
                }
                ClassElement::PrivateFieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    let name_index = self.get_or_insert_private_name(*name);
                    let mut field_compiler = ByteCompiler::new(
                        class_name,
                        true,
                        self.json_parse,
                        self.current_environment.clone(),
                        self.context,
                    );
                    field_compiler.push_compile_environment(false);
                    field_compiler.create_immutable_binding(class_name.into(), true);
                    field_compiler.push_compile_environment(true);
                    if let Some(node) = field {
                        field_compiler.compile_expr(node, true);
                    } else {
                        field_compiler.emit_opcode(Opcode::PushUndefined);
                    }
                    field_compiler.pop_compile_environment();
                    field_compiler.pop_compile_environment();

                    field_compiler.emit_opcode(Opcode::SetReturnValue);

                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    let code = field_compiler.finish();
                    let code = Gc::new(code);
                    let index = self.functions.len() as u32;
                    self.functions.push(code);
                    self.emit(
                        Opcode::GetFunction,
                        &[Operand::Varying(index), Operand::Bool(false)],
                    );
                    self.emit_with_varying_operand(Opcode::PushClassFieldPrivate, name_index);
                }
                ClassElement::StaticFieldDefinition(name, field) => {
                    let name_index = match name {
                        PropertyName::Literal(name) => {
                            Some(self.get_or_insert_name((*name).into()))
                        }
                        PropertyName::Computed(name) => {
                            self.compile_expr(name, true);
                            self.emit(
                                Opcode::RotateRight,
                                &[Operand::U8(3 + static_field_name_count)],
                            );
                            static_field_name_count += 1;
                            None
                        }
                    };
                    let mut field_compiler = ByteCompiler::new(
                        class_name,
                        true,
                        self.json_parse,
                        self.current_environment.clone(),
                        self.context,
                    );
                    field_compiler.push_compile_environment(false);
                    field_compiler.create_immutable_binding(class_name.into(), true);
                    field_compiler.push_compile_environment(true);
                    if let Some(node) = field {
                        field_compiler.compile_expr(node, true);
                    } else {
                        field_compiler.emit_opcode(Opcode::PushUndefined);
                    }
                    field_compiler.emit_opcode(Opcode::SetReturnValue);

                    field_compiler.pop_compile_environment();
                    field_compiler.pop_compile_environment();

                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    let code = field_compiler.finish();
                    let code = Gc::new(code);

                    static_elements.push(StaticElement::StaticField((code, name_index)));
                }
                ClassElement::PrivateStaticFieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    if let Some(node) = field {
                        self.compile_expr(node, true);
                    } else {
                        self.emit_opcode(Opcode::PushUndefined);
                    }
                    let index = self.get_or_insert_private_name(*name);
                    self.emit_with_varying_operand(Opcode::DefinePrivateField, index);
                }
                ClassElement::StaticBlock(body) => {
                    let mut compiler = ByteCompiler::new(
                        Sym::EMPTY_STRING,
                        true,
                        false,
                        self.current_environment.clone(),
                        self.context,
                    );
                    compiler.push_compile_environment(false);
                    compiler.create_immutable_binding(class_name.into(), true);
                    compiler.push_compile_environment(true);

                    compiler.function_declaration_instantiation(
                        body,
                        &FormalParameterList::default(),
                        false,
                        true,
                        false,
                    );

                    compiler.compile_statement_list(body.statements(), false, false);
                    compiler.pop_compile_environment();
                    compiler.pop_compile_environment();

                    let code = Gc::new(compiler.finish());
                    static_elements.push(StaticElement::StaticBlock(code));
                }
                ClassElement::PrivateMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::PushClassPrivateGetter, index);
                        }
                        MethodDefinition::Set(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::PushClassPrivateSetter, index);
                        }
                        MethodDefinition::Ordinary(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::PushClassPrivateMethod, index);
                        }
                        MethodDefinition::Async(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::PushClassPrivateMethod, index);
                        }
                        MethodDefinition::Generator(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::PushClassPrivateMethod, index);
                        }
                        MethodDefinition::AsyncGenerator(expr) => {
                            self.method(expr.into(), class_name);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit_with_varying_operand(Opcode::PushClassPrivateMethod, index);
                        }
                    }
                }
                ClassElement::MethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Swap);
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassGetterByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassGetterByValue);
                            }
                        },
                        MethodDefinition::Set(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassSetterByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassSetterByValue);
                            }
                        },
                        MethodDefinition::Ordinary(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassMethodByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::Async(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassMethodByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::Generator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassMethodByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::AsyncGenerator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), class_name);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit_with_varying_operand(
                                    Opcode::DefineClassMethodByName,
                                    index,
                                );
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), class_name);
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                    }
                    self.emit_opcode(Opcode::Swap);
                }
            }
        }

        for element in static_elements {
            match element {
                StaticElement::StaticBlock(code) => {
                    self.emit_opcode(Opcode::Dup);
                    let index = self.functions.len() as u32;
                    self.functions.push(code);
                    self.emit(
                        Opcode::GetFunction,
                        &[Operand::Varying(index), Operand::Bool(false)],
                    );
                    self.emit_opcode(Opcode::SetHomeObjectClass);
                    self.emit_with_varying_operand(Opcode::Call, 0);
                    self.emit_opcode(Opcode::Pop);
                }
                StaticElement::StaticField((code, name_index)) => {
                    self.emit_opcode(Opcode::Dup);
                    self.emit_opcode(Opcode::Dup);
                    let index = self.functions.len() as u32;
                    self.functions.push(code);
                    self.emit(
                        Opcode::GetFunction,
                        &[Operand::Varying(index), Operand::Bool(false)],
                    );
                    self.emit_opcode(Opcode::SetHomeObjectClass);
                    self.emit_with_varying_operand(Opcode::Call, 0);
                    if let Some(name_index) = name_index {
                        self.emit_with_varying_operand(Opcode::DefineOwnPropertyByName, name_index);
                    } else {
                        self.emit(Opcode::RotateLeft, &[Operand::U8(5)]);
                        self.emit_opcode(Opcode::Swap);
                        self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                }
            }
        }

        self.emit_opcode(Opcode::Swap);
        self.emit_opcode(Opcode::Pop);

        if let Some(class_env) = class_env {
            self.emit_opcode(Opcode::Dup);
            self.emit_binding(BindingOpcode::InitConst, class_name.into());

            let env_index = self.pop_compile_environment();
            self.patch_jump_with_target(class_env, env_index);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        self.emit_opcode(Opcode::PopPrivateEnvironment);

        if !expression {
            self.emit_binding(BindingOpcode::InitVar, class_name.into());
        }
    }
}
