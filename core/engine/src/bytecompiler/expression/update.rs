use crate::bytecompiler::{Access, BindingAccessOpcode, ByteCompiler, Register, ToJsString};
use boa_ast::{
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        operator::{update::UpdateOp, Update},
    },
    scope::BindingLocatorError,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_update(&mut self, update: &Update, dst: &Register) {
        let increment = matches!(
            update.op(),
            UpdateOp::IncrementPost | UpdateOp::IncrementPre
        );
        let post = matches!(
            update.op(),
            UpdateOp::IncrementPost | UpdateOp::DecrementPost
        );

        match Access::from_update_target(update.target()) {
            Access::Variable { name } => {
                let name = name.to_js_string(self.interner());
                let binding = self.lexical_scope.get_identifier_reference(name.clone());
                let is_lexical = binding.is_lexical();
                let index = self.get_binding(&binding);

                if is_lexical {
                    self.emit_binding_access(BindingAccessOpcode::GetName, &index, dst);
                } else {
                    self.emit_binding_access(BindingAccessOpcode::GetNameAndLocator, &index, dst);
                }

                let value = self.register_allocator.alloc();
                if increment {
                    self.bytecode.emit_inc(value.variable(), dst.variable());
                } else {
                    self.bytecode.emit_dec(value.variable(), dst.variable());
                }

                if is_lexical {
                    match self.lexical_scope.set_mutable_binding(name.clone()) {
                        Ok(binding) => {
                            let index = self.insert_binding(binding);
                            self.emit_binding_access(BindingAccessOpcode::SetName, &index, &value);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_string(name);
                            self.bytecode.emit_throw_mutate_immutable(index.into());
                        }
                        Err(BindingLocatorError::Silent) => {}
                    }
                } else {
                    self.emit_binding_access(BindingAccessOpcode::SetNameByLocator, &index, &value);
                }
                if !post {
                    self.bytecode.emit_move(dst.variable(), value.variable());
                }

                self.register_allocator.dealloc(value);
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => {
                    let object = self.register_allocator.alloc();
                    self.compile_expr(access.target(), &object);

                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            self.emit_get_property_by_name(dst, &object, &object, *ident);
                            let value = self.register_allocator.alloc();
                            if increment {
                                self.bytecode.emit_inc(value.variable(), dst.variable());
                            } else {
                                self.bytecode.emit_dec(value.variable(), dst.variable());
                            }

                            self.emit_set_property_by_name(&value, &object, &object, *ident);

                            if !post {
                                self.bytecode.emit_move(dst.variable(), value.variable());
                            }

                            self.register_allocator.dealloc(object);
                            self.register_allocator.dealloc(value);
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = self.register_allocator.alloc();
                            self.compile_expr(expr, &key);

                            self.bytecode.emit_get_property_by_value_push(
                                dst.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                            );

                            let value = self.register_allocator.alloc();
                            if increment {
                                self.bytecode.emit_inc(value.variable(), dst.variable());
                            } else {
                                self.bytecode.emit_dec(value.variable(), dst.variable());
                            }

                            self.bytecode.emit_set_property_by_value(
                                value.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                            );

                            if !post {
                                self.bytecode.emit_move(dst.variable(), value.variable());
                            }

                            self.register_allocator.dealloc(key);
                            self.register_allocator.dealloc(object);
                            self.register_allocator.dealloc(value);
                        }
                    }
                }
                PropertyAccess::Private(access) => {
                    let index = self.get_or_insert_private_name(access.field());

                    let object = self.register_allocator.alloc();
                    self.compile_expr(access.target(), &object);

                    self.bytecode.emit_get_private_field(
                        dst.variable(),
                        object.variable(),
                        index.into(),
                    );

                    let value = self.register_allocator.alloc();
                    if increment {
                        self.bytecode.emit_inc(value.variable(), dst.variable());
                    } else {
                        self.bytecode.emit_dec(value.variable(), dst.variable());
                    }
                    self.bytecode.emit_set_private_field(
                        value.variable(),
                        object.variable(),
                        index.into(),
                    );

                    if !post {
                        self.bytecode.emit_move(dst.variable(), value.variable());
                    }

                    self.register_allocator.dealloc(value);
                    self.register_allocator.dealloc(object);
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(ident) => {
                        let object = self.register_allocator.alloc();
                        let receiver = self.register_allocator.alloc();
                        self.bytecode.emit_super(object.variable());
                        self.bytecode.emit_this(receiver.variable());

                        self.emit_get_property_by_name(dst, &receiver, &object, *ident);

                        let value = self.register_allocator.alloc();
                        if increment {
                            self.bytecode.emit_inc(value.variable(), dst.variable());
                        } else {
                            self.bytecode.emit_dec(value.variable(), dst.variable());
                        }

                        self.emit_set_property_by_name(&value, &receiver, &object, *ident);
                        if !post {
                            self.bytecode.emit_move(dst.variable(), value.variable());
                        }

                        self.register_allocator.dealloc(receiver);
                        self.register_allocator.dealloc(object);
                        self.register_allocator.dealloc(value);
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = self.register_allocator.alloc();
                        let receiver = self.register_allocator.alloc();
                        self.bytecode.emit_super(object.variable());
                        self.bytecode.emit_this(receiver.variable());

                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);

                        self.bytecode.emit_get_property_by_value(
                            dst.variable(),
                            key.variable(),
                            receiver.variable(),
                            object.variable(),
                        );
                        if increment {
                            self.bytecode.emit_inc(dst.variable(), dst.variable());
                        } else {
                            self.bytecode.emit_dec(dst.variable(), dst.variable());
                        }
                        self.bytecode.emit_set_property_by_value(
                            dst.variable(),
                            key.variable(),
                            receiver.variable(),
                            object.variable(),
                        );

                        self.register_allocator.dealloc(receiver);
                        self.register_allocator.dealloc(object);
                        self.register_allocator.dealloc(key);
                    }
                },
            },
            Access::This => unreachable!(),
        }
    }
}
