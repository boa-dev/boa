use crate::{
    bytecompiler::{Access, ByteCompiler},
    environments::BindingLocatorError,
    vm::{BindingOpcode, Opcode},
};
use boa_ast::expression::{
    access::{PropertyAccess, PropertyAccessField},
    operator::{assign::AssignOp, Assign},
};

impl ByteCompiler<'_, '_> {
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

            match access {
                Access::Variable { name } => {
                    let binding = self.get_binding_value(name);
                    let index = self.get_or_insert_binding(binding);
                    let lex = self.current_environment.borrow().is_lex_binding(name);

                    if lex {
                        self.emit(Opcode::GetName, &[index]);
                    } else {
                        self.emit(Opcode::GetNameAndLocator, &[index]);
                    }

                    if short_circuit {
                        early_exit = Some(self.emit_opcode_with_operand(opcode));
                        self.compile_expr(assign.rhs(), true);
                    } else {
                        self.compile_expr(assign.rhs(), true);
                        self.emit_opcode(opcode);
                    }
                    if use_expr {
                        self.emit_opcode(Opcode::Dup);
                    }
                    if lex {
                        match self.set_mutable_binding(name) {
                            Ok(binding) => {
                                let index = self.get_or_insert_binding(binding);
                                self.emit(Opcode::SetName, &[index]);
                            }
                            Err(BindingLocatorError::MutateImmutable) => {
                                let index = self.get_or_insert_name(name);
                                self.emit(Opcode::ThrowMutateImmutable, &[index]);
                            }
                            Err(BindingLocatorError::Silent) => {
                                self.emit(Opcode::Pop, &[]);
                            }
                        }
                    } else {
                        self.emit_opcode(Opcode::SetNameByLocator);
                    }
                }
                Access::Property { access } => match access {
                    PropertyAccess::Simple(access) => match access.field() {
                        PropertyAccessField::Const(name) => {
                            let index = self.get_or_insert_name((*name).into());
                            self.compile_expr(access.target(), true);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::Dup);

                            self.emit(Opcode::GetPropertyByName, &[index]);
                            if short_circuit {
                                pop_count = 2;
                                early_exit = Some(self.emit_opcode_with_operand(opcode));
                                self.compile_expr(assign.rhs(), true);
                            } else {
                                self.compile_expr(assign.rhs(), true);
                                self.emit_opcode(opcode);
                            }

                            self.emit(Opcode::SetPropertyByName, &[index]);
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
                                early_exit = Some(self.emit_opcode_with_operand(opcode));
                                self.compile_expr(assign.rhs(), true);
                            } else {
                                self.compile_expr(assign.rhs(), true);
                                self.emit_opcode(opcode);
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

                        self.emit(Opcode::GetPrivateField, &[index]);
                        if short_circuit {
                            pop_count = 1;
                            early_exit = Some(self.emit_opcode_with_operand(opcode));
                            self.compile_expr(assign.rhs(), true);
                        } else {
                            self.compile_expr(assign.rhs(), true);
                            self.emit_opcode(opcode);
                        }

                        self.emit(Opcode::SetPrivateField, &[index]);
                        if !use_expr {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                    PropertyAccess::Super(access) => match access.field() {
                        PropertyAccessField::Const(name) => {
                            let index = self.get_or_insert_name((*name).into());
                            self.emit_opcode(Opcode::Super);
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::This);
                            self.emit_opcode(Opcode::Swap);
                            self.emit_opcode(Opcode::This);

                            self.emit(Opcode::GetPropertyByName, &[index]);
                            if short_circuit {
                                pop_count = 2;
                                early_exit = Some(self.emit_opcode_with_operand(opcode));
                                self.compile_expr(assign.rhs(), true);
                            } else {
                                self.compile_expr(assign.rhs(), true);
                                self.emit_opcode(opcode);
                            }

                            self.emit(Opcode::SetPropertyByName, &[index]);
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
                                early_exit = Some(self.emit_opcode_with_operand(opcode));
                                self.compile_expr(assign.rhs(), true);
                            } else {
                                self.compile_expr(assign.rhs(), true);
                                self.emit_opcode(opcode);
                            }

                            self.emit_opcode(Opcode::This);
                            self.emit_opcode(Opcode::RotateRight);
                            self.emit_u8(2);

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
                if pop_count == 0 {
                    self.patch_jump(early_exit);
                } else {
                    let exit = self.emit_opcode_with_operand(Opcode::Jump);
                    self.patch_jump(early_exit);
                    for _ in 0..pop_count {
                        self.emit_opcode(Opcode::Swap);
                        self.emit_opcode(Opcode::Pop);
                    }
                    self.patch_jump(exit);
                }
            }
        }
    }
}
