use crate::{
    bytecompiler::{Access, ByteCompiler, NodeKind},
    vm::Opcode,
};
use boa_ast::{
    expression::literal::ObjectLiteral,
    property::{MethodDefinition, PropertyDefinition, PropertyName},
};
use boa_interner::Sym;

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_object_literal(&mut self, object: &ObjectLiteral, use_expr: bool) {
        self.emit_opcode(Opcode::PushEmptyObject);
        for property in object.properties() {
            self.emit_opcode(Opcode::Dup);
            match property {
                PropertyDefinition::IdentifierReference(ident) => {
                    let index = self.get_or_insert_name(*ident);
                    self.access_get(Access::Variable { name: *ident }, true);
                    self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                }
                PropertyDefinition::Property(name, expr) => match name {
                    PropertyName::Literal(name) => {
                        self.compile_expr(expr, true);
                        let index = self.get_or_insert_name((*name).into());
                        if *name == Sym::__PROTO__ && !self.json_parse {
                            self.emit_opcode(Opcode::SetPrototype);
                        } else {
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                    }
                    PropertyName::Computed(name_node) => {
                        self.compile_expr(name_node, true);
                        self.emit_opcode(Opcode::ToPropertyKey);
                        if expr.is_anonymous_function_definition() {
                            self.emit_opcode(Opcode::Dup);
                            self.compile_expr(expr, true);
                            self.emit_opcode(Opcode::SetFunctionName);
                            self.emit_u8(0);
                        } else {
                            self.compile_expr(expr, true);
                        }
                        self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                },
                PropertyDefinition::MethodDefinition(name, kind) => match kind {
                    MethodDefinition::Get(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::SetPropertyGetterByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_expr(name_node, true);
                            self.emit_opcode(Opcode::ToPropertyKey);
                            self.emit_opcode(Opcode::Dup);
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            self.emit_opcode(Opcode::SetFunctionName);
                            self.emit_u8(1);
                            self.emit_opcode(Opcode::SetPropertyGetterByValue);
                        }
                    },
                    MethodDefinition::Set(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::SetPropertySetterByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_expr(name_node, true);
                            self.emit_opcode(Opcode::ToPropertyKey);
                            self.emit_opcode(Opcode::Dup);
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            self.emit_opcode(Opcode::SetFunctionName);
                            self.emit_u8(2);
                            self.emit_opcode(Opcode::SetPropertySetterByValue);
                        }
                    },
                    MethodDefinition::Ordinary(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_expr(name_node, true);
                            self.emit_opcode(Opcode::ToPropertyKey);
                            self.emit_opcode(Opcode::Dup);
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            self.emit_opcode(Opcode::SetFunctionName);
                            self.emit_u8(0);
                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                        }
                    },
                    MethodDefinition::Async(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_expr(name_node, true);
                            self.emit_opcode(Opcode::ToPropertyKey);
                            self.emit_opcode(Opcode::Dup);
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            self.emit_opcode(Opcode::SetFunctionName);
                            self.emit_u8(0);
                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                        }
                    },
                    MethodDefinition::Generator(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_expr(name_node, true);
                            self.emit_opcode(Opcode::ToPropertyKey);
                            self.emit_opcode(Opcode::Dup);
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            self.emit_opcode(Opcode::SetFunctionName);
                            self.emit_u8(0);
                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                        }
                    },
                    MethodDefinition::AsyncGenerator(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_expr(name_node, true);
                            self.emit_opcode(Opcode::ToPropertyKey);
                            self.emit_opcode(Opcode::Dup);
                            self.object_method(expr.into(), NodeKind::Expression, true);
                            self.emit_opcode(Opcode::SetFunctionName);
                            self.emit_u8(0);
                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                        }
                    },
                },
                PropertyDefinition::SpreadObject(expr) => {
                    self.compile_expr(expr, true);
                    self.emit_opcode(Opcode::Swap);
                    self.emit(Opcode::CopyDataProperties, &[0, 0]);
                    self.emit_opcode(Opcode::Pop);
                }
                PropertyDefinition::CoverInitializedName(_, _) => {
                    unreachable!("invalid assignment pattern in object literal")
                }
            }
        }

        if !use_expr {
            self.emit(Opcode::Pop, &[]);
        }
    }
}
