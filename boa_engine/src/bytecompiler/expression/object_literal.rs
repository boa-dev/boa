use crate::{
    bytecompiler::{Access, ByteCompiler, FunctionSpec, Operand},
    vm::Opcode,
};
use boa_ast::{
    expression::literal::ObjectLiteral,
    property::{MethodDefinition, PropertyDefinition, PropertyName},
    Expression,
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
                    self.emit_with_varying_operand(Opcode::DefineOwnPropertyByName, index);
                }
                PropertyDefinition::Property(name, expr) => match name {
                    PropertyName::Literal(name) => {
                        self.compile_expr(expr, true);
                        let index = self.get_or_insert_name((*name).into());
                        if *name == Sym::__PROTO__ && !self.json_parse {
                            self.emit_opcode(Opcode::SetPrototype);
                        } else {
                            self.emit_with_varying_operand(Opcode::DefineOwnPropertyByName, index);
                        }
                    }
                    PropertyName::Computed(name_node) => {
                        self.compile_expr(name_node, true);
                        self.emit_opcode(Opcode::ToPropertyKey);
                        if expr.is_anonymous_function_definition() {
                            self.emit_opcode(Opcode::Dup);
                            self.compile_expr(expr, true);
                            self.emit(Opcode::SetFunctionName, &[Operand::U8(0)]);
                        } else {
                            self.compile_expr(expr, true);
                        }
                        self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                },
                PropertyDefinition::MethodDefinition(name, kind) => match kind {
                    MethodDefinition::Get(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into());
                            self.emit_opcode(Opcode::SetHomeObject);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit_with_varying_operand(Opcode::SetPropertyGetterByName, index);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_object_literal_computed_method(
                                name_node,
                                expr.into(),
                                MethodKind::Get,
                            );
                        }
                    },
                    MethodDefinition::Set(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into());
                            self.emit_opcode(Opcode::SetHomeObject);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit_with_varying_operand(Opcode::SetPropertySetterByName, index);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_object_literal_computed_method(
                                name_node,
                                expr.into(),
                                MethodKind::Set,
                            );
                        }
                    },
                    MethodDefinition::Ordinary(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into());
                            self.emit_opcode(Opcode::SetHomeObject);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit_with_varying_operand(Opcode::DefineOwnPropertyByName, index);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_object_literal_computed_method(
                                name_node,
                                expr.into(),
                                MethodKind::Ordinary,
                            );
                        }
                    },
                    MethodDefinition::Async(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into());
                            self.emit_opcode(Opcode::SetHomeObject);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit_with_varying_operand(Opcode::DefineOwnPropertyByName, index);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_object_literal_computed_method(
                                name_node,
                                expr.into(),
                                MethodKind::Ordinary,
                            );
                        }
                    },
                    MethodDefinition::Generator(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into());
                            self.emit_opcode(Opcode::SetHomeObject);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit_with_varying_operand(Opcode::DefineOwnPropertyByName, index);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_object_literal_computed_method(
                                name_node,
                                expr.into(),
                                MethodKind::Ordinary,
                            );
                        }
                    },
                    MethodDefinition::AsyncGenerator(expr) => match name {
                        PropertyName::Literal(name) => {
                            self.object_method(expr.into());
                            self.emit_opcode(Opcode::SetHomeObject);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit_with_varying_operand(Opcode::DefineOwnPropertyByName, index);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_object_literal_computed_method(
                                name_node,
                                expr.into(),
                                MethodKind::Ordinary,
                            );
                        }
                    },
                },
                PropertyDefinition::SpreadObject(expr) => {
                    self.compile_expr(expr, true);
                    self.emit_opcode(Opcode::Swap);
                    self.emit(
                        Opcode::CopyDataProperties,
                        &[Operand::Varying(0), Operand::Varying(0)],
                    );
                    self.emit_opcode(Opcode::Pop);
                }
                PropertyDefinition::CoverInitializedName(_, _) => {
                    unreachable!("invalid assignment pattern in object literal")
                }
            }
        }

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }

    fn compile_object_literal_computed_method(
        &mut self,
        name: &Expression,
        function: FunctionSpec<'_>,
        kind: MethodKind,
    ) {
        // stack: object, object
        self.compile_expr(name, true);

        // stack: object, object, name
        self.emit_opcode(Opcode::ToPropertyKey);

        // stack: object, object, ToPropertyKey(name)
        self.emit_opcode(Opcode::Dup);

        // stack: object, object, ToPropertyKey(name), ToPropertyKey(name)
        self.object_method(function);

        // stack: object, object, ToPropertyKey(name), ToPropertyKey(name), method
        let value = match kind {
            MethodKind::Get => 1,
            MethodKind::Set => 2,
            MethodKind::Ordinary => 0,
        };
        self.emit(Opcode::SetFunctionName, &[Operand::U8(value)]);

        // stack: object, object, ToPropertyKey(name), method
        self.emit(Opcode::RotateLeft, &[Operand::U8(3)]);

        // stack: object, ToPropertyKey(name), method, object
        self.emit_opcode(Opcode::Swap);

        // stack: object, ToPropertyKey(name), object, method
        self.emit_opcode(Opcode::SetHomeObject);

        // stack: object, ToPropertyKey(name), object, method
        self.emit_opcode(Opcode::Swap);

        // stack: object, ToPropertyKey(name), method, object
        self.emit(Opcode::RotateRight, &[Operand::U8(3)]);

        // stack: object, object, ToPropertyKey(name), method
        match kind {
            MethodKind::Get => self.emit_opcode(Opcode::SetPropertyGetterByValue),
            MethodKind::Set => self.emit_opcode(Opcode::SetPropertySetterByValue),
            MethodKind::Ordinary => self.emit_opcode(Opcode::DefineOwnPropertyByValue),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MethodKind {
    Get,
    Set,
    Ordinary,
}
