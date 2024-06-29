use crate::{
    bytecompiler::{Access, ByteCompiler, InstructionOperand, Operand2, ToJsString},
    environments::BindingLocatorError,
    vm::Opcode,
};
use boa_ast::expression::{
    access::{PropertyAccess, PropertyAccessField},
    operator::{update::UpdateOp, Update},
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_update(&mut self, update: &Update, use_expr: bool) {
        let opcode = match update.op() {
            UpdateOp::IncrementPost | UpdateOp::IncrementPre => Opcode::Inc,
            UpdateOp::DecrementPre | UpdateOp::DecrementPost => Opcode::Dec,
        };
        let post = matches!(
            update.op(),
            UpdateOp::IncrementPost | UpdateOp::DecrementPost
        );

        match Access::from_update_target(update.target()) {
            Access::Variable { name } => {
                let name = name.to_js_string(self.interner());
                let binding = self
                    .lexical_environment
                    .get_identifier_reference(name.clone());
                let index = self.get_or_insert_binding(binding.locator());

                if binding.is_lexical() {
                    self.emit_with_varying_operand(Opcode::GetName, index);
                } else {
                    self.emit_with_varying_operand(Opcode::GetNameAndLocator, index);
                }

                let src = self.register_allocator.alloc();
                let dst = self.register_allocator.alloc();

                self.pop_into_register(&src);

                self.emit2(
                    Opcode::ToNumeric,
                    &[
                        Operand2::Register(&dst),
                        Operand2::Operand(InstructionOperand::Register(&src)),
                    ],
                );
                self.emit2(
                    opcode,
                    &[
                        Operand2::Register(&src),
                        Operand2::Operand(InstructionOperand::Register(&dst)),
                    ],
                );
                self.push_from_register(&src);

                if binding.is_lexical() {
                    match self.lexical_environment.set_mutable_binding(name.clone()) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit_with_varying_operand(Opcode::SetName, index);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_string(name);
                            self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                        }
                        Err(BindingLocatorError::Silent) => {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                } else {
                    self.emit_opcode(Opcode::SetNameByLocator);
                }
                if post {
                    self.push_from_register(&dst);
                } else {
                    self.push_from_register(&src);
                }

                self.register_allocator.dealloc(src);
                self.register_allocator.dealloc(dst);
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => {
                    self.compile_expr(access.target(), true);

                    self.emit_opcode(Opcode::Dup);
                    self.emit_opcode(Opcode::Dup);
                    self.emit_opcode(Opcode::Dup);

                    // Stack: value, value, value, value
                    match access.field() {
                        PropertyAccessField::Const(name) => {
                            self.emit_get_property_by_name(*name);

                            let src = self.register_allocator.alloc();
                            let dst = self.register_allocator.alloc();
                            self.pop_into_register(&src);

                            self.emit2(
                                Opcode::ToNumeric,
                                &[
                                    Operand2::Register(&dst),
                                    Operand2::Operand(InstructionOperand::Register(&src)),
                                ],
                            );
                            self.emit2(
                                opcode,
                                &[
                                    Operand2::Register(&src),
                                    Operand2::Operand(InstructionOperand::Register(&dst)),
                                ],
                            );

                            self.push_from_register(&src);

                            self.emit_set_property_by_name(*name);

                            if post {
                                self.emit_opcode(Opcode::Pop);
                                self.push_from_register(&dst);
                            }

                            self.register_allocator.dealloc(src);
                            self.register_allocator.dealloc(dst);
                        }
                        PropertyAccessField::Expr(expr) => {
                            self.compile_expr(expr, true);

                            self.emit_opcode(Opcode::GetPropertyByValuePush);

                            let src = self.register_allocator.alloc();
                            let dst = self.register_allocator.alloc();
                            self.pop_into_register(&src);

                            self.emit2(
                                Opcode::ToNumeric,
                                &[
                                    Operand2::Register(&dst),
                                    Operand2::Operand(InstructionOperand::Register(&src)),
                                ],
                            );
                            self.emit2(
                                opcode,
                                &[
                                    Operand2::Register(&src),
                                    Operand2::Operand(InstructionOperand::Register(&dst)),
                                ],
                            );

                            self.push_from_register(&src);

                            self.emit_opcode(Opcode::SetPropertyByValue);

                            if post {
                                self.emit_opcode(Opcode::Pop);
                                self.push_from_register(&dst);
                            }

                            self.register_allocator.dealloc(src);
                            self.register_allocator.dealloc(dst);
                        }
                    }
                }
                PropertyAccess::Private(access) => {
                    let index = self.get_or_insert_private_name(access.field());
                    self.compile_expr(access.target(), true);

                    self.emit_opcode(Opcode::Dup);

                    self.emit_with_varying_operand(Opcode::GetPrivateField, index);

                    let src = self.register_allocator.alloc();
                    let dst = self.register_allocator.alloc();
                    self.pop_into_register(&src);

                    self.emit2(
                        Opcode::ToNumeric,
                        &[
                            Operand2::Register(&dst),
                            Operand2::Operand(InstructionOperand::Register(&src)),
                        ],
                    );
                    self.emit2(
                        opcode,
                        &[
                            Operand2::Register(&src),
                            Operand2::Operand(InstructionOperand::Register(&dst)),
                        ],
                    );

                    self.push_from_register(&src);

                    self.emit_with_varying_operand(Opcode::SetPrivateField, index);
                    if post {
                        self.emit_opcode(Opcode::Pop);
                        self.push_from_register(&dst);
                    }

                    self.register_allocator.dealloc(src);
                    self.register_allocator.dealloc(dst);
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::Dup);
                        self.emit_opcode(Opcode::This);
                        self.emit_opcode(Opcode::Swap);
                        self.emit_opcode(Opcode::This);

                        self.emit_get_property_by_name(*name);

                        let src = self.register_allocator.alloc();
                        let dst = self.register_allocator.alloc();
                        self.pop_into_register(&src);

                        self.emit2(
                            Opcode::ToNumeric,
                            &[
                                Operand2::Register(&dst),
                                Operand2::Operand(InstructionOperand::Register(&src)),
                            ],
                        );
                        self.emit2(
                            opcode,
                            &[
                                Operand2::Register(&src),
                                Operand2::Operand(InstructionOperand::Register(&dst)),
                            ],
                        );

                        self.push_from_register(&src);

                        self.emit_set_property_by_name(*name);
                        if post {
                            self.emit_opcode(Opcode::Pop);
                            self.push_from_register(&dst);
                        }

                        self.register_allocator.dealloc(src);
                        self.register_allocator.dealloc(dst);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::Dup);
                        self.emit_opcode(Opcode::This);
                        self.compile_expr(expr, true);

                        self.emit_opcode(Opcode::GetPropertyByValuePush);

                        let src = self.register_allocator.alloc();
                        let dst = self.register_allocator.alloc();
                        self.pop_into_register(&src);

                        self.emit2(
                            Opcode::ToNumeric,
                            &[
                                Operand2::Register(&dst),
                                Operand2::Operand(InstructionOperand::Register(&src)),
                            ],
                        );
                        self.emit2(
                            opcode,
                            &[
                                Operand2::Register(&src),
                                Operand2::Operand(InstructionOperand::Register(&dst)),
                            ],
                        );

                        self.emit_opcode(Opcode::This);
                        self.push_from_register(&src);

                        self.emit_opcode(Opcode::SetPropertyByValue);
                        if post {
                            self.emit_opcode(Opcode::Pop);
                            self.push_from_register(&dst);
                        }

                        self.register_allocator.dealloc(src);
                        self.register_allocator.dealloc(dst);
                    }
                },
            },
            Access::This => unreachable!(),
        }

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }
}
