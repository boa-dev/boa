use crate::{
    bytecompiler::{Access, ByteCompiler, Operand, ToJsString},
    vm::Opcode,
};
use boa_ast::{
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        operator::{update::UpdateOp, Update},
    },
    scope::BindingLocatorError,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_update(&mut self, update: &Update, use_expr: bool) {
        let opcode = match update.op() {
            UpdateOp::IncrementPre => Opcode::Inc,
            UpdateOp::DecrementPre => Opcode::Dec,
            UpdateOp::IncrementPost => Opcode::IncPost,
            UpdateOp::DecrementPost => Opcode::DecPost,
        };
        let post = matches!(
            update.op(),
            UpdateOp::IncrementPost | UpdateOp::DecrementPost
        );

        match Access::from_update_target(update.target()) {
            Access::Variable { name } => {
                let name = name.to_js_string(self.interner());
                let binding = self.lexical_scope.get_identifier_reference(name.clone());
                let is_lexical = binding.is_lexical();
                let index = self.get_or_insert_binding(binding);

                if is_lexical {
                    self.emit_binding_access(Opcode::GetName, &index);
                } else {
                    self.emit_binding_access(Opcode::GetNameAndLocator, &index);
                }

                self.emit_opcode(opcode);
                if post {
                    self.emit_opcode(Opcode::Swap);
                } else {
                    self.emit_opcode(Opcode::Dup);
                }

                if is_lexical {
                    match self.lexical_scope.set_mutable_binding(name.clone()) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit_binding_access(Opcode::SetName, &index);
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
                    self.emit_binding_access(Opcode::SetNameByLocator, &index);
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
                        self.emit_opcode(opcode);
                        if post {
                            self.emit(Opcode::RotateRight, &[Operand::U8(4)]);
                        }

                        self.emit_set_property_by_name(*name);
                        if post {
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
                        self.emit_opcode(opcode);
                        if post {
                            self.emit(Opcode::RotateRight, &[Operand::U8(5)]);
                        }

                        self.emit_opcode(Opcode::SetPropertyByValue);
                        if post {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                },
                PropertyAccess::Private(access) => {
                    let index = self.get_or_insert_private_name(access.field());
                    self.compile_expr(access.target(), true);
                    self.emit_opcode(Opcode::Dup);

                    self.emit_with_varying_operand(Opcode::GetPrivateField, index);
                    self.emit_opcode(opcode);
                    if post {
                        self.emit(Opcode::RotateRight, &[Operand::U8(3)]);
                    }

                    self.emit_with_varying_operand(Opcode::SetPrivateField, index);
                    if post {
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
                        self.emit_opcode(opcode);
                        if post {
                            self.emit(Opcode::RotateRight, &[Operand::U8(3)]);
                        }

                        self.emit_set_property_by_name(*name);
                        if post {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::Dup);
                        self.emit_opcode(Opcode::This);
                        self.compile_expr(expr, true);

                        self.emit_opcode(Opcode::GetPropertyByValuePush);
                        self.emit_opcode(opcode);
                        if post {
                            self.emit(Opcode::RotateRight, &[Operand::U8(2)]);
                        }

                        self.emit_opcode(Opcode::This);
                        self.emit(Opcode::RotateRight, &[Operand::U8(2)]);

                        self.emit_opcode(Opcode::SetPropertyByValue);
                        if post {
                            self.emit_opcode(Opcode::Pop);
                        }
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
