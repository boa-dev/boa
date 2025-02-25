mod assign;
mod binary;
mod object_literal;
mod unary;
mod update;

use std::ops::Deref;

use super::{Access, Callable, NodeKind, Operand, Register, ToJsString};
use crate::{
    bytecompiler::{ByteCompiler, Literal},
    vm::{GeneratorResumeKind, Opcode},
};
use boa_ast::{
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        literal::{Literal as AstLiteral, TemplateElement, TemplateLiteral},
        operator::Conditional,
        Identifier,
    },
    Expression,
};

impl ByteCompiler<'_> {
    fn compile_literal(&mut self, lit: &AstLiteral, dst: &Register) {
        match lit {
            AstLiteral::String(v) => {
                self.emit_push_literal(Literal::String(v.to_js_string(self.interner())), dst);
            }
            AstLiteral::Int(v) => self.emit_push_integer(*v, dst),
            AstLiteral::Num(v) => self.emit_push_rational(*v, dst),
            AstLiteral::BigInt(v) => {
                self.emit_push_literal(Literal::BigInt(v.clone().into()), dst);
            }
            AstLiteral::Bool(true) => self.push_true(dst),
            AstLiteral::Bool(false) => self.push_false(dst),
            AstLiteral::Null => self.push_null(dst),
            AstLiteral::Undefined => self.push_undefined(dst),
        }
    }

    fn compile_conditional(&mut self, op: &Conditional, dst: &Register) {
        self.compile_expr(op.condition(), dst);
        let jelse = self.jump_if_false(dst);
        self.compile_expr(op.if_true(), dst);
        let exit = self.jump();
        self.patch_jump(jelse);
        self.compile_expr(op.if_false(), dst);
        self.patch_jump(exit);
    }

    fn compile_template_literal(&mut self, template_literal: &TemplateLiteral, dst: &Register) {
        let mut registers = Vec::with_capacity(template_literal.elements().len());
        for element in template_literal.elements() {
            let value = self.register_allocator.alloc();
            match element {
                TemplateElement::String(s) => {
                    self.emit_push_literal(
                        Literal::String(s.to_js_string(self.interner())),
                        &value,
                    );
                }
                TemplateElement::Expr(expr) => {
                    self.compile_expr(expr, &value);
                }
            }
            registers.push(value);
        }

        let mut args = Vec::with_capacity(registers.len() + 2);
        args.push(Operand::Register(dst));
        args.push(Operand::Varying(registers.len() as u32));
        for reg in &registers {
            args.push(Operand::Register(reg));
        }
        self.emit(Opcode::ConcatToString, &args);
        for reg in registers {
            self.register_allocator.dealloc(reg);
        }
    }

    pub(crate) fn compile_expr_impl(&mut self, expr: &Expression, dst: &Register) {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit, dst),
            Expression::RegExpLiteral(regexp) => {
                let pattern_index = self.get_or_insert_name(Identifier::new(regexp.pattern()));
                let flags_index = self.get_or_insert_name(Identifier::new(regexp.flags()));
                self.emit(
                    Opcode::PushRegExp,
                    &[
                        Operand::Register(dst),
                        Operand::Varying(pattern_index),
                        Operand::Varying(flags_index),
                    ],
                );
            }
            Expression::Unary(unary) => self.compile_unary(unary, dst),
            Expression::Update(update) => self.compile_update(update, dst),
            Expression::Binary(binary) => self.compile_binary(binary, dst),
            Expression::BinaryInPrivate(binary) => self.compile_binary_in_private(binary, dst),
            Expression::Assign(assign) => self.compile_assign(assign, dst),
            Expression::ObjectLiteral(object) => self.compile_object_literal(object, dst),
            Expression::Identifier(name) => self.access_get(Access::Variable { name: *name }, dst),
            Expression::PropertyAccess(access) => self.access_get(Access::Property { access }, dst),
            Expression::Conditional(op) => self.compile_conditional(op, dst),
            Expression::ArrayLiteral(literal) => {
                let value = self.register_allocator.alloc();

                self.emit(Opcode::PushNewArray, &[Operand::Register(dst)]);

                for element in literal.as_ref() {
                    if let Some(element) = element {
                        self.compile_expr(element, &value);
                        if let Expression::Spread(_) = element {
                            self.emit(Opcode::GetIterator, &[Operand::Register(&value)]);
                            self.emit(Opcode::PushIteratorToArray, &[Operand::Register(dst)]);
                        } else {
                            self.emit(
                                Opcode::PushValueToArray,
                                &[Operand::Register(&value), Operand::Register(dst)],
                            );
                        }
                    } else {
                        self.emit(Opcode::PushElisionToArray, &[Operand::Register(dst)]);
                    }
                }
                self.register_allocator.dealloc(value);
            }
            Expression::This => self.access_get(Access::This, dst),
            Expression::Spread(spread) => self.compile_expr(spread.target(), dst),
            Expression::FunctionExpression(function) => {
                self.function_with_binding(function.as_ref().into(), NodeKind::Expression, dst);
            }
            Expression::ArrowFunction(function) => {
                self.function_with_binding(function.as_ref().into(), NodeKind::Expression, dst);
            }
            Expression::AsyncArrowFunction(function) => {
                self.function_with_binding(function.as_ref().into(), NodeKind::Expression, dst);
            }
            Expression::GeneratorExpression(function) => {
                self.function_with_binding(function.as_ref().into(), NodeKind::Expression, dst);
            }
            Expression::AsyncFunctionExpression(function) => {
                self.function_with_binding(function.as_ref().into(), NodeKind::Expression, dst);
            }
            Expression::AsyncGeneratorExpression(function) => {
                self.function_with_binding(function.as_ref().into(), NodeKind::Expression, dst);
            }
            Expression::Call(call) => self.call(Callable::Call(call), dst),
            Expression::New(new) => self.call(Callable::New(new), dst),
            Expression::TemplateLiteral(template_literal) => {
                self.compile_template_literal(template_literal, dst);
            }
            Expression::Await(expr) => {
                self.compile_expr(expr.target(), dst);
                self.emit(Opcode::Await, &[Operand::Register(dst)]);
                let resume_kind = self.register_allocator.alloc();
                self.pop_into_register(&resume_kind);
                self.pop_into_register(dst);
                self.emit(
                    Opcode::GeneratorNext,
                    &[Operand::Register(&resume_kind), Operand::Register(dst)],
                );
                self.register_allocator.dealloc(resume_kind);
            }
            Expression::Yield(r#yield) => {
                if let Some(expr) = r#yield.target() {
                    self.compile_expr(expr, dst);
                } else {
                    self.push_undefined(dst);
                }

                if r#yield.delegate() {
                    if self.is_async() {
                        self.emit(Opcode::GetAsyncIterator, &[Operand::Register(dst)]);
                    } else {
                        self.emit(Opcode::GetIterator, &[Operand::Register(dst)]);
                    }

                    let resume_kind = self.register_allocator.alloc();
                    let is_return = self.register_allocator.alloc();
                    self.push_undefined(dst);
                    self.emit_resume_kind(GeneratorResumeKind::Normal, &resume_kind);

                    let start_address = self.next_opcode_location();

                    let (throw_method_undefined, return_method_undefined) =
                        self.generator_delegate_next(dst, &resume_kind, &is_return);

                    if self.is_async() {
                        self.emit(Opcode::Await, &[Operand::Register(dst)]);
                        self.pop_into_register(&resume_kind);
                        self.pop_into_register(dst);
                    } else {
                        self.emit_resume_kind(GeneratorResumeKind::Normal, &resume_kind);
                    }

                    let (return_gen, exit) =
                        self.generator_delegate_resume(dst, &resume_kind, &is_return);

                    if self.is_async() {
                        self.emit(Opcode::IteratorValue, &[Operand::Register(dst)]);
                        self.async_generator_yield(dst, &resume_kind);
                    } else {
                        self.emit(Opcode::IteratorResult, &[Operand::Register(dst)]);
                        self.emit(Opcode::GeneratorYield, &[Operand::Register(dst)]);
                        self.pop_into_register(&resume_kind);
                        self.pop_into_register(dst);
                    }
                    self.emit(Opcode::Jump, &[Operand::U32(start_address)]);

                    self.register_allocator.dealloc(resume_kind);
                    self.register_allocator.dealloc(is_return);

                    self.patch_jump(return_gen);
                    self.patch_jump(return_method_undefined);
                    if self.is_async() {
                        self.emit(Opcode::Await, &[Operand::Register(dst)]);
                        self.emit_opcode(Opcode::Pop);
                    } else {
                        self.push_from_register(dst);
                    }
                    self.close_active_iterators();

                    self.r#return(true);

                    self.patch_jump(throw_method_undefined);
                    self.iterator_close(self.is_async());
                    self.emit_type_error("iterator does not have a throw method");

                    self.patch_jump(exit);
                } else {
                    self.r#yield(dst);
                }
            }
            Expression::TaggedTemplate(template) => {
                let this = self.register_allocator.alloc();
                let function = self.register_allocator.alloc();

                match template.tag() {
                    Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                        self.compile_expr(access.target(), &this);
                        match access.field() {
                            PropertyAccessField::Const(ident) => {
                                self.emit_get_property_by_name(&function, &this, &this, *ident);
                            }
                            PropertyAccessField::Expr(field) => {
                                let key = self.register_allocator.alloc();
                                self.compile_expr(field, &key);
                                self.emit(
                                    Opcode::GetPropertyByValue,
                                    &[
                                        Operand::Register(&function),
                                        Operand::Register(&key),
                                        Operand::Register(&this),
                                        Operand::Register(&this),
                                    ],
                                );
                                self.register_allocator.dealloc(key);
                            }
                        }
                    }
                    Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                        let index = self.get_or_insert_private_name(access.field());
                        self.compile_expr(access.target(), &this);
                        self.emit(
                            Opcode::GetPrivateField,
                            &[
                                Operand::Register(&function),
                                Operand::Register(&this),
                                Operand::Varying(index),
                            ],
                        );
                    }
                    expr => {
                        self.push_undefined(&this);
                        self.compile_expr(expr, &function);
                    }
                }

                self.push_from_register(&this);
                self.push_from_register(&function);

                self.register_allocator.dealloc(this);
                self.register_allocator.dealloc(function);

                let site = template.identifier();
                let count = template.cookeds().len() as u32;
                let jump_label = self.template_lookup(dst, site);

                let mut part_registers = Vec::with_capacity(count as usize * 2);

                for (cooked, raw) in template.cookeds().iter().zip(template.raws()) {
                    let value = self.register_allocator.alloc();
                    if let Some(cooked) = cooked {
                        self.emit_push_literal(
                            Literal::String(cooked.to_js_string(self.interner())),
                            &value,
                        );
                    } else {
                        self.push_undefined(&value);
                    }
                    part_registers.push(value);
                    let value = self.register_allocator.alloc();
                    self.emit_push_literal(
                        Literal::String(raw.to_js_string(self.interner())),
                        &value,
                    );
                    part_registers.push(value);
                }

                let mut args = Vec::with_capacity(count as usize * 2 + 2);
                args.push(Operand::U64(site));
                args.push(Operand::Register(dst));
                args.push(Operand::Varying(count));
                for r in &part_registers {
                    args.push(Operand::Register(r));
                }
                self.emit(Opcode::TemplateCreate, &args);
                for r in part_registers {
                    self.register_allocator.dealloc(r);
                }

                self.patch_jump(jump_label);
                self.push_from_register(dst);

                for expr in template.exprs() {
                    let value = self.register_allocator.alloc();
                    self.compile_expr(expr, &value);
                    self.push_from_register(&value);
                    self.register_allocator.dealloc(value);
                }

                self.emit_with_varying_operand(Opcode::Call, template.exprs().len() as u32 + 1);
                self.pop_into_register(dst);
            }
            Expression::ClassExpression(class) => {
                self.compile_class(class.deref().into(), Some(dst));
            }
            Expression::SuperCall(super_call) => {
                let value = self.register_allocator.alloc();
                self.emit(Opcode::SuperCallPrepare, &[Operand::Register(&value)]);
                self.push_from_register(&value);
                self.register_allocator.dealloc(value);

                let contains_spread = super_call
                    .arguments()
                    .iter()
                    .any(|arg| matches!(arg, Expression::Spread(_)));

                if contains_spread {
                    let array = self.register_allocator.alloc();
                    let value = self.register_allocator.alloc();

                    self.emit(Opcode::PushNewArray, &[Operand::Register(&array)]);

                    for arg in super_call.arguments() {
                        self.compile_expr(arg, &value);
                        if let Expression::Spread(_) = arg {
                            self.emit(Opcode::GetIterator, &[Operand::Register(&value)]);
                            self.emit(Opcode::PushIteratorToArray, &[Operand::Register(&array)]);
                        } else {
                            self.emit(
                                Opcode::PushValueToArray,
                                &[Operand::Register(&value), Operand::Register(&array)],
                            );
                        }
                    }

                    self.push_from_register(&array);

                    self.register_allocator.dealloc(value);
                    self.register_allocator.dealloc(array);
                } else {
                    for arg in super_call.arguments() {
                        let value = self.register_allocator.alloc();
                        self.compile_expr(arg, &value);
                        self.push_from_register(&value);
                        self.register_allocator.dealloc(value);
                    }
                }

                if contains_spread {
                    self.emit_opcode(Opcode::SuperCallSpread);
                } else {
                    self.emit_with_varying_operand(
                        Opcode::SuperCall,
                        super_call.arguments().len() as u32,
                    );
                }
                self.pop_into_register(dst);
                self.emit(Opcode::BindThisValue, &[Operand::Register(dst)]);
            }
            Expression::ImportCall(import) => {
                self.compile_expr(import.argument(), dst);
                self.emit(Opcode::ImportCall, &[Operand::Register(dst)]);
            }
            Expression::NewTarget => {
                self.emit(Opcode::NewTarget, &[Operand::Register(dst)]);
            }
            Expression::ImportMeta => {
                self.emit(Opcode::ImportMeta, &[Operand::Register(dst)]);
            }
            Expression::Optional(opt) => {
                let this = self.register_allocator.alloc();
                self.compile_optional_preserve_this(opt, &this, dst);
                self.register_allocator.dealloc(this);
            }
            Expression::Parenthesized(parenthesized) => {
                self.compile_expr(parenthesized.expression(), dst);
            }
            // TODO: try to remove this variant somehow
            Expression::FormalParameterList(_) => unreachable!(),
            Expression::Debugger => (),
        }
    }
}
