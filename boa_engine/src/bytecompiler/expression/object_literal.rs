use boa_ast::{
    expression::literal::ObjectLiteral,
    property::{MethodDefinition, PropertyDefinition, PropertyName},
};
use boa_interner::Sym;

use crate::{
    bytecompiler::{Access, ByteCompiler, NodeKind},
    vm::Opcode,
    JsNativeError, JsResult,
};

pub(crate) fn compile_object_literal<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    object: &ObjectLiteral,
    use_expr: bool,
) -> JsResult<()> {
    byte_compiler.emit_opcode(Opcode::PushEmptyObject);
    for property in object.properties() {
        byte_compiler.emit_opcode(Opcode::Dup);
        match property {
            PropertyDefinition::IdentifierReference(ident) => {
                let index = byte_compiler.get_or_insert_name(*ident);
                byte_compiler.access_get(Access::Variable { name: *ident }, true)?;
                byte_compiler.emit(Opcode::DefineOwnPropertyByName, &[index]);
            }
            PropertyDefinition::Property(name, expr) => match name {
                PropertyName::Literal(name) => {
                    byte_compiler.compile_expr(expr, true)?;
                    let index = byte_compiler.get_or_insert_name((*name).into());
                    if *name == Sym::__PROTO__ && !byte_compiler.json_parse {
                        byte_compiler.emit_opcode(Opcode::SetPrototype);
                    } else {
                        byte_compiler.emit(Opcode::DefineOwnPropertyByName, &[index]);
                    }
                }
                PropertyName::Computed(name_node) => {
                    byte_compiler.compile_expr(name_node, true)?;
                    byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                    byte_compiler.compile_expr(expr, true)?;
                    byte_compiler.emit_opcode(Opcode::DefineOwnPropertyByValue);
                }
            },
            PropertyDefinition::MethodDefinition(name, kind) => match kind {
                // TODO: set function name for getter and setters
                MethodDefinition::Get(expr) => match name {
                    PropertyName::Literal(name) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::SetPropertyGetterByName, &[index]);
                    }
                    PropertyName::Computed(name_node) => {
                        byte_compiler.compile_expr(name_node, true)?;
                        byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        byte_compiler.emit_opcode(Opcode::SetPropertyGetterByValue);
                    }
                },
                // TODO: set function name for getter and setters
                MethodDefinition::Set(expr) => match name {
                    PropertyName::Literal(name) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::SetPropertySetterByName, &[index]);
                    }
                    PropertyName::Computed(name_node) => {
                        byte_compiler.compile_expr(name_node, true)?;
                        byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        byte_compiler.emit_opcode(Opcode::SetPropertySetterByValue);
                    }
                },
                MethodDefinition::Ordinary(expr) => match name {
                    PropertyName::Literal(name) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::DefineOwnPropertyByName, &[index]);
                    }
                    PropertyName::Computed(name_node) => {
                        byte_compiler.compile_expr(name_node, true)?;
                        byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        byte_compiler.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                },
                MethodDefinition::Async(expr) => match name {
                    PropertyName::Literal(name) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::DefineOwnPropertyByName, &[index]);
                    }
                    PropertyName::Computed(name_node) => {
                        byte_compiler.compile_expr(name_node, true)?;
                        byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        byte_compiler.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                },
                MethodDefinition::Generator(expr) => match name {
                    PropertyName::Literal(name) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::DefineOwnPropertyByName, &[index]);
                    }
                    PropertyName::Computed(name_node) => {
                        byte_compiler.compile_expr(name_node, true)?;
                        byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        byte_compiler.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                },
                MethodDefinition::AsyncGenerator(expr) => match name {
                    PropertyName::Literal(name) => {
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        let index = byte_compiler.get_or_insert_name((*name).into());
                        byte_compiler.emit(Opcode::DefineOwnPropertyByName, &[index]);
                    }
                    PropertyName::Computed(name_node) => {
                        byte_compiler.compile_expr(name_node, true)?;
                        byte_compiler.emit_opcode(Opcode::ToPropertyKey);
                        byte_compiler.function(expr.into(), NodeKind::Expression, true)?;
                        byte_compiler.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                },
            },
            PropertyDefinition::SpreadObject(expr) => {
                byte_compiler.compile_expr(expr, true)?;
                byte_compiler.emit_opcode(Opcode::Swap);
                byte_compiler.emit(Opcode::CopyDataProperties, &[0, 0]);
                byte_compiler.emit_opcode(Opcode::Pop);
            }
            // TODO: Promote to early errors
            PropertyDefinition::CoverInitializedName(_, _) => {
                return Err(JsNativeError::syntax()
                    .with_message("invalid assignment pattern in object literal")
                    .into())
            }
        }
    }

    if !use_expr {
        byte_compiler.emit(Opcode::Pop, &[]);
    }

    Ok(())
}
