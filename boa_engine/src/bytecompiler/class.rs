use boa_ast::{
    declaration::Binding,
    expression::Identifier,
    function::{Class, ClassElement},
    operations::bound_names,
    property::{MethodDefinition, PropertyName},
};
use boa_gc::Gc;
use boa_interner::Sym;
use rustc_hash::FxHashMap;

use crate::{
    vm::{BindingOpcode, CodeBlock, Opcode},
    JsResult,
};

use super::{ByteCompiler, Literal, NodeKind};

/// This function compiles a class declaration or expression.
///
/// The compilation of a class declaration and expression is mostly equal.
/// A class declaration binds the resulting class object to it's identifier.
/// A class expression leaves the resulting class object on the stack for following operations.
pub(crate) fn compile_class<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    class: &Class,
    expression: bool,
) -> JsResult<()> {
    let code = CodeBlock::new(
        class.name().map_or(Sym::EMPTY_STRING, Identifier::sym),
        0,
        true,
    );
    let mut compiler = ByteCompiler {
        code_block: code,
        literals_map: FxHashMap::default(),
        names_map: FxHashMap::default(),
        bindings_map: FxHashMap::default(),
        jump_info: Vec::new(),
        in_async_generator: false,
        json_parse: byte_compiler.json_parse,
        context: byte_compiler.context,
    };
    compiler.context.push_compile_time_environment(true);

    if let Some(expr) = class.constructor() {
        compiler.code_block.length = expr.parameters().length();
        compiler.code_block.params = expr.parameters().clone();
        compiler
            .context
            .create_mutable_binding(Sym::ARGUMENTS.into(), false, false);
        compiler.code_block.arguments_binding = Some(
            compiler
                .context
                .initialize_mutable_binding(Sym::ARGUMENTS.into(), false),
        );
        for parameter in expr.parameters().as_ref() {
            if parameter.is_rest_param() {
                compiler.emit_opcode(Opcode::RestParameterInit);
            }

            match parameter.variable().binding() {
                Binding::Identifier(ident) => {
                    compiler
                        .context
                        .create_mutable_binding(*ident, false, false);
                    if let Some(init) = parameter.variable().init() {
                        let skip = compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        compiler.compile_expr(init, true)?;
                        compiler.patch_jump(skip);
                    }
                    compiler.emit_binding(BindingOpcode::InitArg, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        compiler.context.create_mutable_binding(ident, false, false);
                    }
                    compiler.compile_declaration_pattern(pattern, BindingOpcode::InitArg)?;
                }
            }
        }
        if !expr.parameters().has_rest_parameter() {
            compiler.emit_opcode(Opcode::RestParameterPop);
        }
        let env_label = if expr.parameters().has_expressions() {
            compiler.code_block.num_bindings = compiler.context.get_binding_number();
            compiler.context.push_compile_time_environment(true);
            compiler.code_block.function_environment_push_location =
                compiler.next_opcode_location();
            Some(compiler.emit_opcode_with_two_operands(Opcode::PushFunctionEnvironment))
        } else {
            None
        };
        compiler.create_decls(expr.body(), false);
        compiler.compile_statement_list(expr.body(), false, false)?;
        if let Some(env_label) = env_label {
            let (num_bindings, compile_environment) =
                compiler.context.pop_compile_time_environment();
            let index_compile_environment = compiler.push_compile_environment(compile_environment);
            compiler.patch_jump_with_target(env_label.0, num_bindings as u32);
            compiler.patch_jump_with_target(env_label.1, index_compile_environment as u32);
            let (_, compile_environment) = compiler.context.pop_compile_time_environment();
            compiler.push_compile_environment(compile_environment);
        } else {
            let (num_bindings, compile_environment) =
                compiler.context.pop_compile_time_environment();
            compiler.push_compile_environment(compile_environment);
            compiler.code_block.num_bindings = num_bindings;
            compiler.code_block.is_class_constructor = true;
        }
    } else {
        if class.super_ref().is_some() {
            compiler.emit_opcode(Opcode::SuperCallDerived);
        }
        let (num_bindings, compile_environment) = compiler.context.pop_compile_time_environment();
        compiler.push_compile_environment(compile_environment);
        compiler.code_block.num_bindings = num_bindings;
        compiler.code_block.is_class_constructor = true;
    }

    compiler.emit_opcode(Opcode::PushUndefined);
    compiler.emit_opcode(Opcode::Return);

    let code = Gc::new(compiler.finish());
    let index = byte_compiler.code_block.functions.len() as u32;
    byte_compiler.code_block.functions.push(code);
    byte_compiler.emit(Opcode::GetFunction, &[index]);

    byte_compiler.emit_opcode(Opcode::Dup);
    if let Some(node) = class.super_ref() {
        byte_compiler.compile_expr(node, true)?;
        byte_compiler.emit_opcode(Opcode::PushClassPrototype);
    } else {
        byte_compiler.emit_opcode(Opcode::PushUndefined);
    }
    byte_compiler.emit_opcode(Opcode::SetClassPrototype);
    byte_compiler.emit_opcode(Opcode::Swap);

    // TODO: set function name for getter and setters
    for element in class.elements() {
        match element {
            ClassElement::StaticMethodDefinition(name, method_definition) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                match &method_definition {
                    MethodDefinition::Get(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassGetterByName, &[index]);
                        }
                        PropertyName::Computed(ref name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassGetterByValue);
                        }
                    },
                    MethodDefinition::Set(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassSetterByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassSetterByValue);
                        }
                    },
                    MethodDefinition::Ordinary(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassMethodByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassMethodByValue);
                        }
                    },
                    MethodDefinition::Async(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassMethodByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassMethodByValue);
                        }
                    },
                    MethodDefinition::Generator(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassMethodByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassMethodByValue);
                        }
                    },
                    MethodDefinition::AsyncGenerator(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassMethodByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassMethodByValue);
                        }
                    },
                }
            }
            // TODO: set names for private methods
            ClassElement::PrivateStaticMethodDefinition(name, method_definition) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                match &method_definition {
                    MethodDefinition::Get(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::SetPrivateGetter, &[index]);
                    }
                    MethodDefinition::Set(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::SetPrivateSetter, &[index]);
                    }
                    MethodDefinition::Ordinary(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::SetPrivateMethod, &[index]);
                    }
                    MethodDefinition::Async(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::SetPrivateMethod, &[index]);
                    }
                    MethodDefinition::Generator(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::SetPrivateMethod, &[index]);
                    }
                    MethodDefinition::AsyncGenerator(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::SetPrivateMethod, &[index]);
                    }
                }
            }
            ClassElement::FieldDefinition(name, field) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                match name {
                    PropertyName::Literal(name) => {
                        byte_compiler.emit_push_literal(Literal::String(
                            byte_compiler
                                .interner()
                                .resolve_expect(*name)
                                .into_common(false),
                        ));
                    }
                    PropertyName::Computed(name) => {
                        byte_compiler.compile_expr(name, true)?;
                    }
                }
                let field_code = CodeBlock::new(Sym::EMPTY_STRING, 0, true);
                let mut field_compiler = ByteCompiler {
                    code_block: field_code,
                    literals_map: FxHashMap::default(),
                    names_map: FxHashMap::default(),
                    bindings_map: FxHashMap::default(),
                    jump_info: Vec::new(),
                    in_async_generator: false,
                    json_parse: byte_compiler.json_parse,
                    context: byte_compiler.context,
                };
                field_compiler.context.push_compile_time_environment(true);
                if let Some(node) = field {
                    field_compiler.compile_expr(node, true)?;
                } else {
                    field_compiler.emit_opcode(Opcode::PushUndefined);
                }
                let (num_bindings, compile_environment) =
                    field_compiler.context.pop_compile_time_environment();
                field_compiler.push_compile_environment(compile_environment);
                field_compiler.code_block.num_bindings = num_bindings;
                field_compiler.emit_opcode(Opcode::Return);

                let code = Gc::new(field_compiler.finish());
                let index = byte_compiler.code_block.functions.len() as u32;
                byte_compiler.code_block.functions.push(code);
                byte_compiler.emit(Opcode::GetFunction, &[index]);
                byte_compiler.emit_opcode(Opcode::PushClassField);
            }
            ClassElement::PrivateFieldDefinition(name, field) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                let name_index = byte_compiler.get_or_insert_name((*name).into());
                let field_code = CodeBlock::new(Sym::EMPTY_STRING, 0, true);
                let mut field_compiler = ByteCompiler {
                    code_block: field_code,
                    literals_map: FxHashMap::default(),
                    names_map: FxHashMap::default(),
                    bindings_map: FxHashMap::default(),
                    jump_info: Vec::new(),
                    in_async_generator: false,
                    json_parse: byte_compiler.json_parse,
                    context: byte_compiler.context,
                };
                field_compiler.context.push_compile_time_environment(true);
                if let Some(node) = field {
                    field_compiler.compile_expr(node, true)?;
                } else {
                    field_compiler.emit_opcode(Opcode::PushUndefined);
                }
                let (num_bindings, compile_environment) =
                    field_compiler.context.pop_compile_time_environment();
                field_compiler.push_compile_environment(compile_environment);
                field_compiler.code_block.num_bindings = num_bindings;
                field_compiler.emit_opcode(Opcode::Return);

                let code = Gc::new(field_compiler.finish());
                let index = byte_compiler.code_block.functions.len() as u32;
                byte_compiler.code_block.functions.push(code);
                byte_compiler.emit(Opcode::GetFunction, &[index]);
                byte_compiler.emit(Opcode::PushClassFieldPrivate, &[name_index]);
            }
            ClassElement::StaticFieldDefinition(name, field) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                match name {
                    PropertyName::Literal(name) => {
                        if let Some(node) = field {
                            byte_compiler.compile_expr(node, true)?;
                        } else {
                            byte_compiler.emit_opcode(Opcode::PushUndefined);
                        }
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::DefineOwnPropertyByName, &[index]);
                    }
                    PropertyName::Computed(name_node) => {
                        byte_compiler.compile_expr(name_node, true)?;
                        byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                        if let Some(node) = field {
                            byte_compiler.compile_expr(node, true)?;
                        } else {
                            byte_compiler.emit_opcode(Opcode::PushUndefined);
                        }
                        byte_compiler.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                }
            }
            ClassElement::PrivateStaticFieldDefinition(name, field) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                if let Some(node) = field {
                    byte_compiler.compile_expr(node, true)?;
                } else {
                    byte_compiler.emit_opcode(Opcode::PushUndefined);
                }
                let index = byte_compiler.get_or_insert_name((*name).into());
                byte_compiler.emit(Opcode::SetPrivateField, &[index]);
            }
            ClassElement::StaticBlock(statement_list) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                let mut compiler =
                    ByteCompiler::new(Sym::EMPTY_STRING, true, false, byte_compiler.context);
                compiler.context.push_compile_time_environment(true);
                compiler.create_decls(statement_list, false);
                compiler.compile_statement_list(statement_list, false, false)?;
                let (num_bindings, compile_environment) =
                    compiler.context.pop_compile_time_environment();
                compiler.push_compile_environment(compile_environment);
                compiler.code_block.num_bindings = num_bindings;

                let code = Gc::new(compiler.finish());
                let index = byte_compiler.code_block.functions.len() as u32;
                byte_compiler.code_block.functions.push(code);
                byte_compiler.emit(Opcode::GetFunction, &[index]);
                byte_compiler.emit_opcode(Opcode::SetHomeObject);
                byte_compiler.emit(Opcode::Call, &[0]);
                byte_compiler.emit_opcode(Opcode::Pop);
            }
            // TODO: set names for private methods
            ClassElement::PrivateMethodDefinition(name, method_definition) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                match method_definition {
                    MethodDefinition::Get(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::PushClassPrivateGetter, &[index]);
                    }
                    MethodDefinition::Set(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::PushClassPrivateSetter, &[index]);
                    }
                    MethodDefinition::Ordinary(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::PushClassPrivateMethod, &[index]);
                    }
                    MethodDefinition::Async(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::PushClassPrivateMethod, &[index]);
                    }
                    MethodDefinition::Generator(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::PushClassPrivateMethod, &[index]);
                    }
                    MethodDefinition::AsyncGenerator(expr) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::PushClassPrivateMethod, &[index]);
                    }
                }
            }
            ClassElement::MethodDefinition(..) => {}
        }
    }

    byte_compiler.emit_opcode(Opcode::Swap);

    for element in class.elements() {
        match element {
            ClassElement::MethodDefinition(name, method_definition) => {
                byte_compiler.emit_opcode(Opcode::Dup);
                // TODO: set names for getters and setters
                match method_definition {
                    MethodDefinition::Get(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassGetterByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassGetterByValue);
                        }
                    },
                    MethodDefinition::Set(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassSetterByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassSetterByValue);
                        }
                    },
                    MethodDefinition::Ordinary(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassMethodByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassMethodByValue);
                        }
                    },
                    MethodDefinition::Async(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassMethodByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassMethodByValue);
                        }
                    },
                    MethodDefinition::Generator(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassMethodByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassMethodByValue);
                        }
                    },
                    MethodDefinition::AsyncGenerator(expr) => match name {
                        PropertyName::Literal(name) => {
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            let index = byte_compiler.get_or_insert_name((*name).into());
                            byte_compiler.emit(Opcode::DefineClassMethodByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            byte_compiler.compile_expr(name_node, true)?;
                            byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                            byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                            byte_compiler.emit_opcode(Opcode::DefineClassMethodByValue);
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

    byte_compiler.emit_opcode(Opcode::Pop);

    if !expression {
        byte_compiler.emit_binding(
            BindingOpcode::InitVar,
            class.name().expect("class statements must have a name"),
        );
    }
    Ok(())
}
