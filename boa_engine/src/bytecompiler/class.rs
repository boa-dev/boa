use super::{ByteCompiler, Literal, NodeKind};
use crate::vm::{BindingOpcode, Opcode};
use boa_ast::{
    declaration::Binding,
    expression::Identifier,
    function::{Class, ClassElement},
    operations::bound_names,
    property::{MethodDefinition, PropertyName},
};
use boa_gc::Gc;
use boa_interner::Sym;

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
                compiler.has_binding_identifier = true;
                compiler.push_compile_environment(false);
                compiler.create_immutable_binding(class_name, true);
            }
        }

        compiler.push_compile_environment(true);

        if let Some(expr) = class.constructor() {
            compiler.length = expr.parameters().length();
            compiler.params = expr.parameters().clone();
            compiler.create_mutable_binding(Sym::ARGUMENTS.into(), false, false);
            compiler.arguments_binding =
                Some(compiler.initialize_mutable_binding(Sym::ARGUMENTS.into(), false));
            for parameter in expr.parameters().as_ref() {
                if parameter.is_rest_param() {
                    compiler.emit_opcode(Opcode::RestParameterInit);
                }

                match parameter.variable().binding() {
                    Binding::Identifier(ident) => {
                        compiler.create_mutable_binding(*ident, false, false);
                        if let Some(init) = parameter.variable().init() {
                            let skip =
                                compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                            compiler.compile_expr(init, true);
                            compiler.patch_jump(skip);
                        }
                        compiler.emit_binding(BindingOpcode::InitArg, *ident);
                    }
                    Binding::Pattern(pattern) => {
                        for ident in bound_names(pattern) {
                            compiler.create_mutable_binding(ident, false, false);
                        }
                        compiler.compile_declaration_pattern(pattern, BindingOpcode::InitArg);
                    }
                }
            }
            if !expr.parameters().has_rest_parameter() {
                compiler.emit_opcode(Opcode::RestParameterPop);
            }
            let env_label = if expr.parameters().has_expressions() {
                compiler.num_bindings = compiler.current_environment.borrow().num_bindings();
                compiler.push_compile_environment(true);
                compiler.function_environment_push_location = compiler.next_opcode_location();
                Some(compiler.emit_opcode_with_two_operands(Opcode::PushFunctionEnvironment))
            } else {
                None
            };
            compiler.create_script_decls(expr.body(), false);
            compiler.compile_statement_list(expr.body(), false, false);

            let env_info = compiler.pop_compile_environment();

            if let Some(env_label) = env_label {
                compiler.patch_jump_with_target(env_label.0, env_info.num_bindings as u32);
                compiler.patch_jump_with_target(env_label.1, env_info.index as u32);
                compiler.pop_compile_environment();
            } else {
                compiler.num_bindings = env_info.num_bindings;
                compiler.is_class_constructor = true;
            }
        } else {
            if class.super_ref().is_some() {
                compiler.emit_opcode(Opcode::SuperCallDerived);
            }
            let env_info = compiler.pop_compile_environment();
            compiler.num_bindings = env_info.num_bindings;
            compiler.is_class_constructor = true;
        }

        if class.name().is_some() && class.has_binding_identifier() {
            compiler.pop_compile_environment();
        }

        compiler.emit_opcode(Opcode::PushUndefined);
        compiler.emit_opcode(Opcode::Return);

        let code = Gc::new(compiler.finish());
        let index = self.functions.len() as u32;
        self.functions.push(code);
        self.emit(Opcode::GetFunction, &[index]);
        self.emit_u8(0);

        self.emit_opcode(Opcode::Dup);
        if let Some(node) = class.super_ref() {
            self.compile_expr(node, true);
            self.emit_opcode(Opcode::PushClassPrototype);
        } else {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.emit_opcode(Opcode::SetClassPrototype);
        self.emit_opcode(Opcode::Swap);

        // TODO: set function name for getter and setters
        for element in class.elements() {
            match element {
                ClassElement::StaticMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassStaticGetterByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassStaticGetterByValue);
                            }
                        },
                        MethodDefinition::Set(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassStaticSetterByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassStaticSetterByValue);
                            }
                        },
                        MethodDefinition::Ordinary(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassStaticMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassStaticMethodByValue);
                            }
                        },
                        MethodDefinition::Async(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassStaticMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassStaticMethodByValue);
                            }
                        },
                        MethodDefinition::Generator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassStaticMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassStaticMethodByValue);
                            }
                        },
                        MethodDefinition::AsyncGenerator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassStaticMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassStaticMethodByValue);
                            }
                        },
                    }
                }
                // TODO: set names for private methods
                ClassElement::PrivateStaticMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::SetPrivateGetter, &[index]);
                        }
                        MethodDefinition::Set(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::SetPrivateSetter, &[index]);
                        }
                        MethodDefinition::Ordinary(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::SetPrivateMethod, &[index]);
                        }
                        MethodDefinition::Async(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::SetPrivateMethod, &[index]);
                        }
                        MethodDefinition::Generator(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::SetPrivateMethod, &[index]);
                        }
                        MethodDefinition::AsyncGenerator(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::SetPrivateMethod, &[index]);
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
                    let env_info = field_compiler.pop_compile_environment();
                    field_compiler.pop_compile_environment();
                    field_compiler.num_bindings = env_info.num_bindings;
                    field_compiler.emit_opcode(Opcode::Return);

                    let mut code = field_compiler.finish();
                    code.class_field_initializer_name = Some(Sym::EMPTY_STRING);
                    let code = Gc::new(code);
                    let index = self.functions.len() as u32;
                    self.functions.push(code);
                    self.emit(Opcode::GetFunction, &[index]);
                    self.emit_u8(0);
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
                    let env_info = field_compiler.pop_compile_environment();
                    field_compiler.pop_compile_environment();
                    field_compiler.num_bindings = env_info.num_bindings;
                    field_compiler.emit_opcode(Opcode::Return);

                    let mut code = field_compiler.finish();
                    code.class_field_initializer_name = Some(Sym::EMPTY_STRING);
                    let code = Gc::new(code);
                    let index = self.functions.len() as u32;
                    self.functions.push(code);
                    self.emit(Opcode::GetFunction, &[index]);
                    self.emit_u8(0);
                    self.emit(Opcode::PushClassFieldPrivate, &[name_index]);
                }
                ClassElement::StaticFieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    self.emit_opcode(Opcode::Dup);
                    let name_index = match name {
                        PropertyName::Literal(name) => {
                            Some(self.get_or_insert_name((*name).into()))
                        }
                        PropertyName::Computed(name) => {
                            self.compile_expr(name, true);
                            self.emit_opcode(Opcode::Swap);
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
                    let env_info = field_compiler.pop_compile_environment();
                    field_compiler.pop_compile_environment();
                    field_compiler.num_bindings = env_info.num_bindings;
                    field_compiler.emit_opcode(Opcode::Return);

                    let mut code = field_compiler.finish();
                    code.class_field_initializer_name = Some(Sym::EMPTY_STRING);
                    let code = Gc::new(code);
                    let index = self.functions.len() as u32;
                    self.functions.push(code);
                    self.emit(Opcode::GetFunction, &[index]);
                    self.emit_u8(0);
                    self.emit_opcode(Opcode::SetHomeObject);
                    self.emit(Opcode::Call, &[0]);
                    if let Some(name_index) = name_index {
                        self.emit(Opcode::DefineOwnPropertyByName, &[name_index]);
                    } else {
                        self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                }
                ClassElement::PrivateStaticFieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    if let Some(node) = field {
                        self.compile_expr(node, true);
                    } else {
                        self.emit_opcode(Opcode::PushUndefined);
                    }
                    let index = self.get_or_insert_private_name(*name);
                    self.emit(Opcode::DefinePrivateField, &[index]);
                }
                ClassElement::StaticBlock(statement_list) => {
                    self.emit_opcode(Opcode::Dup);
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
                    compiler.create_script_decls(statement_list, false);
                    compiler.compile_statement_list(statement_list, false, false);
                    let env_info = compiler.pop_compile_environment();
                    compiler.pop_compile_environment();
                    compiler.num_bindings = env_info.num_bindings;

                    let code = Gc::new(compiler.finish());
                    let index = self.functions.len() as u32;
                    self.functions.push(code);
                    self.emit(Opcode::GetFunction, &[index]);
                    self.emit_u8(0);
                    self.emit_opcode(Opcode::SetHomeObject);
                    self.emit(Opcode::Call, &[0]);
                    self.emit_opcode(Opcode::Pop);
                }
                // TODO: set names for private methods
                ClassElement::PrivateMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::PushClassPrivateGetter, &[index]);
                        }
                        MethodDefinition::Set(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::PushClassPrivateSetter, &[index]);
                        }
                        MethodDefinition::Ordinary(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::PushClassPrivateMethod, &[index]);
                        }
                        MethodDefinition::Async(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::PushClassPrivateMethod, &[index]);
                        }
                        MethodDefinition::Generator(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::PushClassPrivateMethod, &[index]);
                        }
                        MethodDefinition::AsyncGenerator(expr) => {
                            self.method(expr.into(), NodeKind::Expression, class_name, true);
                            let index = self.get_or_insert_private_name(*name);
                            self.emit(Opcode::PushClassPrivateMethod, &[index]);
                        }
                    }
                }
                ClassElement::MethodDefinition(..) => {}
            }
        }

        self.emit_opcode(Opcode::Swap);

        for element in class.elements() {
            match element {
                ClassElement::MethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    // TODO: set names for getters and setters
                    match method_definition {
                        MethodDefinition::Get(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassGetterByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassGetterByValue);
                            }
                        },
                        MethodDefinition::Set(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassSetterByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassSetterByValue);
                            }
                        },
                        MethodDefinition::Ordinary(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::Async(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::Generator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::AsyncGenerator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true);
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.method(expr.into(), NodeKind::Expression, class_name, true);
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                    }
                }
                ClassElement::PrivateMethodDefinition(..)
                | ClassElement::PrivateFieldDefinition(..)
                | ClassElement::StaticFieldDefinition(..)
                | ClassElement::PrivateStaticFieldDefinition(..)
                | ClassElement::StaticMethodDefinition(..)
                | ClassElement::PrivateStaticMethodDefinition(..)
                | ClassElement::StaticBlock(..)
                | ClassElement::FieldDefinition(..) => {}
            }
        }

        self.emit_opcode(Opcode::Pop);

        if !expression {
            self.emit_binding(
                BindingOpcode::InitVar,
                class.name().expect("class statements must have a name"),
            );
        }
    }
}
