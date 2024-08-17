use crate::{
    bytecompiler::{Access, ByteCompiler, InstructionOperand, Operand, Operand2, ToJsString},
    environments::BindingLocatorError,
    vm::{BindingOpcode, Opcode},
};
use boa_ast::expression::{
    access::{PropertyAccess, PropertyAccessField},
    operator::{assign::AssignOp, Assign},
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_assign(&mut self, assign: &Assign, use_expr: bool) {
        if assign.op() == AssignOp::Assign {
            match Access::from_assign_target(assign.lhs()) {
                Ok(access) => self.access_set(access, use_expr, |compiler, _| {
                    compiler.compile_expr(assign.rhs(), true);
                }),
                Err(pattern) => {
                    self.compile_expr(assign.rhs(), true);
                    if use_expr {
                        self.emit_opcode(Opcode::Dup);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::SetName);
                }
            }
        } else {
            let access = Access::from_assign_target(assign.lhs())
                .expect("patterns should throw early errors on complex assignment operators");

            let opcode = match assign.op() {
                AssignOp::Assign => unreachable!(),
                AssignOp::Add => Opcode::Add,
                AssignOp::Sub => Opcode::Sub,
                AssignOp::Mul => Opcode::Mul,
                AssignOp::Div => Opcode::Div,
                AssignOp::Mod => Opcode::Mod,
                AssignOp::Exp => Opcode::Pow,
                AssignOp::And => Opcode::BitAnd,
                AssignOp::Or => Opcode::BitOr,
                AssignOp::Xor => Opcode::BitXor,
                AssignOp::Shl => Opcode::ShiftLeft,
                AssignOp::Shr => Opcode::ShiftRight,
                AssignOp::Ushr => Opcode::UnsignedShiftRight,
                AssignOp::BoolAnd => Opcode::LogicalAnd,
                AssignOp::BoolOr => Opcode::LogicalOr,
                AssignOp::Coalesce => Opcode::Coalesce,
            };

            let short_circuit = matches!(
                assign.op(),
                AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce
            );
            let mut pop_count = 0;
            let mut early_exit = None;

            let lhs = self.register_allocator.alloc();

            let emit_stack_opcode = |this: &mut ByteCompiler<'_>| {
                let rhs = this.register_allocator.alloc();
                this.pop_into_register(&rhs);
                this.pop_into_register(&lhs);
                this.emit2(
                    opcode,
                    &[
                        Operand2::Register(&lhs),
                        Operand2::Operand(InstructionOperand::Register(&lhs)),
                        Operand2::Operand(InstructionOperand::Register(&rhs)),
                    ],
                );
                this.register_allocator.dealloc(rhs);

                this.push_from_register(&lhs);
            };

            match access {
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

                    if short_circuit {
                        self.pop_into_register(&lhs);
                        early_exit =
                            Some(self.emit_opcode_with_operand2(
                                opcode,
                                InstructionOperand::Register(&lhs),
                            ));
                        self.compile_expr(assign.rhs(), true);

                        self.pop_into_register(&lhs);
                        self.push_from_register(&lhs);
                    } else {
                        self.compile_expr(assign.rhs(), true);
                        emit_stack_opcode(self);
                    }

                    if use_expr {
                        self.emit_opcode(Opcode::Dup);
                    }

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
                }
                Access::Property { access } => match access {
                    PropertyAccess::Simple(access) => match access.field() {
                        PropertyAccessField::Const(name) => {
                            self.compile_expr(access.target(), true);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);

                            self.emit_get_property_by_name(*name);

                            if short_circuit {
                                pop_count = 2;
                                self.pop_into_register(&lhs);
                                early_exit = Some(self.emit_opcode_with_operand2(
                                    opcode,
                                    InstructionOperand::Register(&lhs),
                                ));
                                self.compile_expr(assign.rhs(), true);
                                self.pop_into_register(&lhs);
                                self.push_from_register(&lhs);
                            } else {
                                self.compile_expr(assign.rhs(), true);
                                emit_stack_opcode(self);
                            }

                            self.emit_set_property_by_name(*name);

                            if !use_expr {
                                self.emit_opcode(Opcode::Pop);
                            }
                        }
                        PropertyAccessField::Expr(expr) => {
                            self.compile_expr(access.target(), true);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);
                            self.compile_expr(expr, true);

                            self.emit_opcode(Opcode::GetPropertyByValuePush);
                            if short_circuit {
                                pop_count = 3;

                                self.pop_into_register(&lhs);
                                early_exit = Some(self.emit_opcode_with_operand2(
                                    opcode,
                                    InstructionOperand::Register(&lhs),
                                ));
                                self.compile_expr(assign.rhs(), true);
                                self.pop_into_register(&lhs);
                                self.push_from_register(&lhs);
                            } else {
                                self.compile_expr(assign.rhs(), true);
                                emit_stack_opcode(self);
                            }

                            self.emit_opcode(Opcode::SetPropertyByValue);
                            if !use_expr {
                                self.emit_opcode(Opcode::Pop);
                            }
                        }
                    },
                    PropertyAccess::Private(access) => {
                        let index = self.get_or_insert_private_name(access.field());
                        self.compile_expr(access.target(), true);
                        self.emit_opcode(Opcode::Dup);

                        self.emit_with_varying_operand(Opcode::GetPrivateField, index);
                        if short_circuit {
                            pop_count = 1;

                            self.pop_into_register(&lhs);
                            early_exit = Some(self.emit_opcode_with_operand2(
                                opcode,
                                InstructionOperand::Register(&lhs),
                            ));
                            self.compile_expr(assign.rhs(), true);
                            self.pop_into_register(&lhs);
                            self.push_from_register(&lhs);
                        } else {
                            self.compile_expr(assign.rhs(), true);
                            emit_stack_opcode(self);
                        }

                        self.emit_with_varying_operand(Opcode::SetPrivateField, index);
                        if !use_expr {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                    PropertyAccess::Super(access) => match access.field() {
                        PropertyAccessField::Const(name) => {
                            self.emit_opcode(Opcode::Super);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::This);
                            self.emit_opcode(Opcode::Swap);
                            self.emit_opcode(Opcode::This);

                            self.emit_get_property_by_name(*name);
                            if short_circuit {
                                pop_count = 2;

                                self.pop_into_register(&lhs);
                                early_exit = Some(self.emit_opcode_with_operand2(
                                    opcode,
                                    InstructionOperand::Register(&lhs),
                                ));
                                self.compile_expr(assign.rhs(), true);
                                self.pop_into_register(&lhs);
                                self.push_from_register(&lhs);
                            } else {
                                self.compile_expr(assign.rhs(), true);
                                emit_stack_opcode(self);
                            }

                            self.emit_set_property_by_name(*name);
                            if !use_expr {
                                self.emit_opcode(Opcode::Pop);
                            }
                        }
                        PropertyAccessField::Expr(expr) => {
                            self.emit_opcode(Opcode::Super);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::This);
                            self.compile_expr(expr, true);

                            self.emit_opcode(Opcode::GetPropertyByValuePush);
                            if short_circuit {
                                pop_count = 2;

                                self.pop_into_register(&lhs);
                                early_exit = Some(self.emit_opcode_with_operand2(
                                    opcode,
                                    InstructionOperand::Register(&lhs),
                                ));
                                self.compile_expr(assign.rhs(), true);
                                self.pop_into_register(&lhs);
                                self.push_from_register(&lhs);
                            } else {
                                self.compile_expr(assign.rhs(), true);
                                emit_stack_opcode(self);
                            }

                            self.emit_opcode(Opcode::This);
                            self.emit(Opcode::RotateRight, &[Operand::U8(2)]);

                            self.emit_opcode(Opcode::SetPropertyByValue);
                            if !use_expr {
                                self.emit_opcode(Opcode::Pop);
                            }
                        }
                    },
                },
                Access::This => unreachable!(),
            }

            if let Some(early_exit) = early_exit {
                let exit = self.emit_opcode_with_operand(Opcode::Jump);

                self.patch_jump(early_exit);
                for _ in 0..pop_count {
                    self.emit_opcode(Opcode::Pop);
                }
                self.push_from_register(&lhs);
                self.patch_jump(exit);
            }

            self.register_allocator.dealloc(lhs);
        }
    }
}
