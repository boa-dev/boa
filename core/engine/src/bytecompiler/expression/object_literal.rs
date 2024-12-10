use crate::{
    bytecompiler::{Access, ByteCompiler, FunctionSpec, MethodKind, Operand, Register},
    vm::Opcode,
};
use boa_ast::{
    expression::literal::{ObjectLiteral, PropertyDefinition},
    property::{MethodDefinitionKind, PropertyName},
    Expression,
};
use boa_interner::Sym;

impl ByteCompiler<'_> {
    pub(crate) fn compile_object_literal(&mut self, literal: &ObjectLiteral, dst: &Register) {
        self.emit(Opcode::PushEmptyObject, &[Operand::Register(dst)]);

        for property in literal.properties() {
            match property {
                PropertyDefinition::IdentifierReference(ident) => {
                    let value = self.register_allocator.alloc();
                    self.access_get(Access::Variable { name: *ident }, &value);
                    let index = self.get_or_insert_name(*ident);
                    self.emit(
                        Opcode::DefineOwnPropertyByName,
                        &[
                            Operand::Register(dst),
                            Operand::Register(&value),
                            Operand::Varying(index),
                        ],
                    );
                    self.register_allocator.dealloc(value);
                }
                PropertyDefinition::Property(name, expr) => match name {
                    PropertyName::Literal(name) => {
                        let value = self.register_allocator.alloc();
                        self.compile_expr(expr, &value);
                        if *name == Sym::__PROTO__ && !self.json_parse {
                            self.emit(
                                Opcode::SetPrototype,
                                &[Operand::Register(dst), Operand::Register(&value)],
                            );
                        } else {
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(
                                Opcode::DefineOwnPropertyByName,
                                &[
                                    Operand::Register(dst),
                                    Operand::Register(&value),
                                    Operand::Varying(index),
                                ],
                            );
                        }
                        self.register_allocator.dealloc(value);
                    }
                    PropertyName::Computed(name_node) => {
                        let key = self.register_allocator.alloc();
                        self.compile_expr(name_node, &key);
                        self.emit(
                            Opcode::ToPropertyKey,
                            &[Operand::Register(&key), Operand::Register(&key)],
                        );
                        let function = self.register_allocator.alloc();
                        self.compile_expr(expr, &function);
                        if expr.is_anonymous_function_definition() {
                            self.emit(
                                Opcode::SetFunctionName,
                                &[
                                    Operand::Register(&function),
                                    Operand::Register(&key),
                                    Operand::U8(0),
                                ],
                            );
                        }
                        self.emit(
                            Opcode::DefineOwnPropertyByValue,
                            &[
                                Operand::Register(&function),
                                Operand::Register(&key),
                                Operand::Register(dst),
                            ],
                        );
                        self.register_allocator.dealloc(key);
                        self.register_allocator.dealloc(function);
                    }
                },
                PropertyDefinition::MethodDefinition(m) => {
                    let kind = match m.kind() {
                        MethodDefinitionKind::Get => MethodKind::Get,
                        MethodDefinitionKind::Set => MethodKind::Set,
                        _ => MethodKind::Ordinary,
                    };
                    match m.name() {
                        PropertyName::Literal(name) => {
                            let method = self.object_method(m.into(), kind);
                            self.emit(
                                Opcode::SetHomeObject,
                                &[Operand::Register(&method), Operand::Register(dst)],
                            );
                            let index = self.get_or_insert_name((*name).into());
                            let opcode = match kind {
                                MethodKind::Get => Opcode::SetPropertyGetterByName,
                                MethodKind::Set => Opcode::SetPropertySetterByName,
                                MethodKind::Ordinary => Opcode::DefineOwnPropertyByName,
                            };
                            self.emit(
                                opcode,
                                &[
                                    Operand::Register(dst),
                                    Operand::Register(&method),
                                    Operand::Varying(index),
                                ],
                            );

                            self.register_allocator.dealloc(method);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_object_literal_computed_method(
                                name_node,
                                m.into(),
                                kind,
                                dst,
                            );
                        }
                    }
                }
                PropertyDefinition::SpreadObject(expr) => {
                    let source = self.register_allocator.alloc();
                    self.compile_expr(expr, &source);
                    self.emit(
                        Opcode::CopyDataProperties,
                        &[
                            Operand::Register(dst),
                            Operand::Register(&source),
                            Operand::Varying(0),
                        ],
                    );
                    self.register_allocator.dealloc(source);
                }
                PropertyDefinition::CoverInitializedName(_, _) => {
                    unreachable!("invalid assignment pattern in object literal")
                }
            }
        }
    }

    fn compile_object_literal_computed_method(
        &mut self,
        expr: &Expression,
        function: FunctionSpec<'_>,
        kind: MethodKind,
        object: &Register,
    ) {
        let key = self.register_allocator.alloc();
        self.compile_expr(expr, &key);

        self.emit(
            Opcode::ToPropertyKey,
            &[Operand::Register(&key), Operand::Register(&key)],
        );

        let method = self.object_method(function, kind);
        let value = match kind {
            MethodKind::Get => 1,
            MethodKind::Set => 2,
            MethodKind::Ordinary => 0,
        };

        self.emit(
            Opcode::SetFunctionName,
            &[
                Operand::Register(&method),
                Operand::Register(&key),
                Operand::U8(value),
            ],
        );

        self.emit(
            Opcode::SetHomeObject,
            &[Operand::Register(&method), Operand::Register(object)],
        );

        let operands = &[
            Operand::Register(&method),
            Operand::Register(&key),
            Operand::Register(object),
        ];
        match kind {
            MethodKind::Get => self.emit(Opcode::SetPropertyGetterByValue, operands),
            MethodKind::Set => self.emit(Opcode::SetPropertySetterByValue, operands),
            MethodKind::Ordinary => self.emit(Opcode::DefineOwnPropertyByValue, operands),
        }

        self.register_allocator.dealloc(key);
        self.register_allocator.dealloc(method);
    }
}
