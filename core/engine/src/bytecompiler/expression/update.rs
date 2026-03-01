use crate::bytecompiler::{Access, BindingAccessOpcode, ByteCompiler, Register, ToJsString};
use crate::vm::opcode::{
    Dec, GetPrivateField, GetPropertyByValue, GetPropertyByValuePush, Inc, Move, SetPrivateField,
    SetPropertyByValue, Super, This, ThrowMutateImmutable,
};
use boa_ast::{
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        operator::{Update, update::UpdateOp},
    },
    scope::BindingLocatorError,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_update(&mut self, update: &Update, dst: &Register) {
        let mut compiler = self.position_guard(update);
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
                let name = name.to_js_string(compiler.interner());
                let binding = compiler
                    .lexical_scope
                    .get_identifier_reference(name.clone());
                let is_lexical = binding.is_lexical();
                let index = compiler.get_binding(&binding);

                if is_lexical {
                    compiler.emit_binding_access(BindingAccessOpcode::GetName, &index, dst);
                } else {
                    compiler.emit_binding_access(
                        BindingAccessOpcode::GetNameAndLocator,
                        &index,
                        dst,
                    );
                }

                let value = compiler.register_allocator.alloc();
                if increment {
                    Inc::emit(&mut compiler, value.variable(), dst.variable());
                } else {
                    Dec::emit(&mut compiler, value.variable(), dst.variable());
                }

                if is_lexical {
                    match compiler.lexical_scope.set_mutable_binding(name.clone()) {
                        Ok(binding) => {
                            let index = compiler.insert_binding(binding);
                            compiler.emit_binding_access(
                                BindingAccessOpcode::SetName,
                                &index,
                                &value,
                            );
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = compiler.get_or_insert_string(name);
                            ThrowMutateImmutable::emit(&mut compiler, index.into());
                        }
                        Err(BindingLocatorError::Silent) => {}
                    }
                } else {
                    compiler.emit_binding_access(
                        BindingAccessOpcode::SetNameByLocator,
                        &index,
                        &value,
                    );
                }
                if !post {
                    Move::emit(&mut compiler, dst.variable(), value.variable());
                }

                compiler.register_allocator.dealloc(value);
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => {
                    let object = compiler.register_allocator.alloc();
                    compiler.compile_expr(access.target(), &object);

                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            compiler.emit_get_property_by_name(dst, None, &object, ident.sym());
                            let value = compiler.register_allocator.alloc();
                            if increment {
                                Inc::emit(&mut compiler, value.variable(), dst.variable());
                            } else {
                                Dec::emit(&mut compiler, value.variable(), dst.variable());
                            }

                            compiler.emit_set_property_by_name(&value, None, &object, ident.sym());

                            if !post {
                                Move::emit(&mut compiler, dst.variable(), value.variable());
                            }

                            compiler.register_allocator.dealloc(object);
                            compiler.register_allocator.dealloc(value);
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = compiler.register_allocator.alloc();
                            compiler.compile_expr(expr, &key);

                            GetPropertyByValuePush::emit(
                                &mut compiler,
                                dst.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                            );

                            let value = compiler.register_allocator.alloc();
                            if increment {
                                Inc::emit(&mut compiler, value.variable(), dst.variable());
                            } else {
                                Dec::emit(&mut compiler, value.variable(), dst.variable());
                            }

                            SetPropertyByValue::emit(
                                &mut compiler,
                                value.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                            );

                            if !post {
                                Move::emit(&mut compiler, dst.variable(), value.variable());
                            }

                            compiler.register_allocator.dealloc(key);
                            compiler.register_allocator.dealloc(object);
                            compiler.register_allocator.dealloc(value);
                        }
                    }
                }
                PropertyAccess::Private(access) => {
                    let index = compiler.get_or_insert_private_name(access.field());

                    let object = compiler.register_allocator.alloc();
                    compiler.compile_expr(access.target(), &object);

                    GetPrivateField::emit(
                        &mut compiler,
                        dst.variable(),
                        object.variable(),
                        index.into(),
                    );

                    let value = compiler.register_allocator.alloc();
                    if increment {
                        Inc::emit(&mut compiler, value.variable(), dst.variable());
                    } else {
                        Dec::emit(&mut compiler, value.variable(), dst.variable());
                    }
                    SetPrivateField::emit(
                        &mut compiler,
                        value.variable(),
                        object.variable(),
                        index.into(),
                    );

                    if !post {
                        Move::emit(&mut compiler, dst.variable(), value.variable());
                    }

                    compiler.register_allocator.dealloc(value);
                    compiler.register_allocator.dealloc(object);
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(ident) => {
                        let object = compiler.register_allocator.alloc();
                        let receiver = compiler.register_allocator.alloc();
                        Super::emit(&mut compiler, object.variable());
                        This::emit(&mut compiler, receiver.variable());

                        compiler.emit_get_property_by_name(
                            dst,
                            Some(&receiver),
                            &object,
                            ident.sym(),
                        );

                        let value = compiler.register_allocator.alloc();
                        if increment {
                            Inc::emit(&mut compiler, value.variable(), dst.variable());
                        } else {
                            Dec::emit(&mut compiler, value.variable(), dst.variable());
                        }

                        compiler.emit_set_property_by_name(
                            &value,
                            Some(&receiver),
                            &object,
                            ident.sym(),
                        );
                        if !post {
                            Move::emit(&mut compiler, dst.variable(), value.variable());
                        }

                        compiler.register_allocator.dealloc(receiver);
                        compiler.register_allocator.dealloc(object);
                        compiler.register_allocator.dealloc(value);
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = compiler.register_allocator.alloc();
                        let receiver = compiler.register_allocator.alloc();
                        Super::emit(&mut compiler, object.variable());
                        This::emit(&mut compiler, receiver.variable());

                        let key = compiler.register_allocator.alloc();
                        compiler.compile_expr(expr, &key);

                        GetPropertyByValue::emit(
                            &mut compiler,
                            dst.variable(),
                            key.variable(),
                            receiver.variable(),
                            object.variable(),
                        );
                        if increment {
                            Inc::emit(&mut compiler, dst.variable(), dst.variable());
                        } else {
                            Dec::emit(&mut compiler, dst.variable(), dst.variable());
                        }
                        SetPropertyByValue::emit(
                            &mut compiler,
                            dst.variable(),
                            key.variable(),
                            receiver.variable(),
                            object.variable(),
                        );

                        compiler.register_allocator.dealloc(receiver);
                        compiler.register_allocator.dealloc(object);
                        compiler.register_allocator.dealloc(key);
                    }
                },
            },
            Access::This => unreachable!(),
        }
    }
}
