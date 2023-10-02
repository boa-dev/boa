mod assign;
mod binary;
mod object_literal;
mod unary;
mod update;

use super::{Access, Callable, NodeKind, Operand};
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

impl ByteCompiler<'_, '_> {
    fn compile_literal(&mut self, lit: &AstLiteral, use_expr: bool) {
        match lit {
            AstLiteral::String(v) => self.emit_push_literal(Literal::String(
                self.interner().resolve_expect(*v).into_common(false),
            )),
            AstLiteral::Int(v) => self.emit_push_integer(*v),
            AstLiteral::Num(v) => self.emit_push_rational(*v),
            AstLiteral::BigInt(v) => {
                self.emit_push_literal(Literal::BigInt(v.clone().into()));
            }
            AstLiteral::Bool(true) => self.emit(Opcode::PushTrue, &[]),
            AstLiteral::Bool(false) => self.emit(Opcode::PushFalse, &[]),
            AstLiteral::Null => self.emit(Opcode::PushNull, &[]),
            AstLiteral::Undefined => self.emit(Opcode::PushUndefined, &[]),
        }

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }

    fn compile_conditional(&mut self, op: &Conditional, use_expr: bool) {
        self.compile_expr(op.condition(), true);
        let jelse = self.jump_if_false();
        self.compile_expr(op.if_true(), true);
        let exit = self.jump();
        self.patch_jump(jelse);
        self.compile_expr(op.if_false(), true);
        self.patch_jump(exit);

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        };
    }

    fn compile_template_literal(&mut self, template_literal: &TemplateLiteral, use_expr: bool) {
        for element in template_literal.elements() {
            match element {
                TemplateElement::String(s) => self.emit_push_literal(Literal::String(
                    self.interner().resolve_expect(*s).into_common(false),
                )),
                TemplateElement::Expr(expr) => {
                    self.compile_expr(expr, true);
                }
            }
        }

        self.emit_with_varying_operand(
            Opcode::ConcatToString,
            template_literal.elements().len() as u32,
        );

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }

    pub(crate) fn compile_expr_impl(&mut self, expr: &Expression, use_expr: bool) {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit, use_expr),
            Expression::RegExpLiteral(regexp) => {
                let pattern_index = self.get_or_insert_name(Identifier::new(regexp.pattern()));
                let flags_index = self.get_or_insert_name(Identifier::new(regexp.flags()));
                self.emit(
                    Opcode::PushRegExp,
                    &[
                        Operand::Varying(pattern_index),
                        Operand::Varying(flags_index),
                    ],
                );
            }
            Expression::Unary(unary) => self.compile_unary(unary, use_expr),
            Expression::Update(update) => self.compile_update(update, use_expr),
            Expression::Binary(binary) => self.compile_binary(binary, use_expr),
            Expression::BinaryInPrivate(binary) => {
                self.compile_binary_in_private(binary, use_expr);
            }
            Expression::Assign(assign) => self.compile_assign(assign, use_expr),
            Expression::ObjectLiteral(object) => {
                self.compile_object_literal(object, use_expr);
            }
            Expression::Identifier(name) => {
                self.access_get(Access::Variable { name: *name }, use_expr);
            }
            Expression::PropertyAccess(access) => {
                self.access_get(Access::Property { access }, use_expr);
            }
            Expression::Conditional(op) => self.compile_conditional(op, use_expr),
            Expression::ArrayLiteral(array) => {
                self.emit_opcode(Opcode::PushNewArray);

                for element in array.as_ref() {
                    if let Some(element) = element {
                        self.compile_expr(element, true);
                        if let Expression::Spread(_) = element {
                            self.emit_opcode(Opcode::GetIterator);
                            self.emit_opcode(Opcode::PushIteratorToArray);
                        } else {
                            self.emit_opcode(Opcode::PushValueToArray);
                        }
                    } else {
                        self.emit_opcode(Opcode::PushElisionToArray);
                    }
                }

                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::This => {
                self.access_get(Access::This, use_expr);
            }
            Expression::Spread(spread) => self.compile_expr(spread.target(), true),
            Expression::Function(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, use_expr);
            }
            Expression::ArrowFunction(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, use_expr);
            }
            Expression::AsyncArrowFunction(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, use_expr);
            }
            Expression::Generator(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, use_expr);
            }
            Expression::AsyncFunction(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, use_expr);
            }
            Expression::AsyncGenerator(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, use_expr);
            }
            Expression::Call(call) => self.call(Callable::Call(call), use_expr),
            Expression::New(new) => self.call(Callable::New(new), use_expr),
            Expression::TemplateLiteral(template_literal) => {
                self.compile_template_literal(template_literal, use_expr);
            }
            Expression::Await(expr) => {
                self.compile_expr(expr.target(), true);
                self.emit_opcode(Opcode::Await);
                self.emit_opcode(Opcode::GeneratorNext);
                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::Yield(r#yield) => {
                // stack:
                if let Some(expr) = r#yield.target() {
                    self.compile_expr(expr, true);
                } else {
                    self.emit_opcode(Opcode::PushUndefined);
                }

                // stack: value

                if r#yield.delegate() {
                    if self.in_async() {
                        self.emit_opcode(Opcode::GetAsyncIterator);
                    } else {
                        self.emit_opcode(Opcode::GetIterator);
                    }

                    // stack:
                    self.emit_opcode(Opcode::PushUndefined);

                    // stack: undefined
                    self.emit_resume_kind(GeneratorResumeKind::Normal);

                    // stack: resume_kind, undefined
                    let start_address = self.next_opcode_location();
                    let (throw_method_undefined, return_method_undefined) =
                        self.emit_opcode_with_two_operands(Opcode::GeneratorDelegateNext);

                    if self.in_async() {
                        self.emit_opcode(Opcode::Pop);
                        self.emit_opcode(Opcode::Await);
                    }

                    let (return_gen, exit) =
                        self.emit_opcode_with_two_operands(Opcode::GeneratorDelegateResume);
                    if self.in_async() {
                        self.emit_opcode(Opcode::IteratorValue);
                        self.async_generator_yield();
                    } else {
                        self.emit_opcode(Opcode::IteratorResult);
                        self.emit_opcode(Opcode::GeneratorYield);
                    }
                    self.emit(Opcode::Jump, &[Operand::U32(start_address)]);

                    self.patch_jump(return_gen);
                    self.patch_jump(return_method_undefined);
                    if self.in_async() {
                        self.emit_opcode(Opcode::Await);
                        self.emit_opcode(Opcode::Pop);
                    }
                    self.close_active_iterators();

                    self.r#return(true);

                    self.patch_jump(throw_method_undefined);
                    self.iterator_close(self.in_async());
                    self.emit_opcode(Opcode::Throw);

                    self.patch_jump(exit);
                } else {
                    self.r#yield();
                }

                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::TaggedTemplate(template) => {
                match template.tag() {
                    Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                        self.compile_expr(access.target(), true);
                        self.emit_opcode(Opcode::Dup);
                        self.emit_opcode(Opcode::Dup);
                        match access.field() {
                            PropertyAccessField::Const(field) => {
                                let index = self.get_or_insert_name((*field).into());
                                self.emit_with_varying_operand(Opcode::GetPropertyByName, index);
                            }
                            PropertyAccessField::Expr(field) => {
                                self.compile_expr(field, true);
                                self.emit_opcode(Opcode::GetPropertyByValue);
                            }
                        }
                    }
                    Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                        self.compile_expr(access.target(), true);
                        self.emit(Opcode::Dup, &[]);
                        let index = self.get_or_insert_private_name(access.field());
                        self.emit_with_varying_operand(Opcode::GetPrivateField, index);
                    }
                    expr => {
                        self.emit_opcode(Opcode::PushUndefined);
                        self.compile_expr(expr, true);
                    }
                }

                let site = template.identifier();
                let count = template.cookeds().len() as u32;

                let jump_label = self.emit_opcode_with_operand(Opcode::TemplateLookup);
                self.emit_u64(site);

                for (cooked, raw) in template.cookeds().iter().zip(template.raws()) {
                    if let Some(cooked) = cooked {
                        self.emit_push_literal(Literal::String(
                            self.interner().resolve_expect(*cooked).into_common(false),
                        ));
                    } else {
                        self.emit_opcode(Opcode::PushUndefined);
                    }
                    self.emit_push_literal(Literal::String(
                        self.interner().resolve_expect(*raw).into_common(false),
                    ));
                }

                self.emit(
                    Opcode::TemplateCreate,
                    &[Operand::Varying(count), Operand::U64(site)],
                );

                self.patch_jump(jump_label);

                for expr in template.exprs() {
                    self.compile_expr(expr, true);
                }

                self.emit_with_varying_operand(Opcode::Call, template.exprs().len() as u32 + 1);
            }
            Expression::Class(class) => self.class(class, true),
            Expression::SuperCall(super_call) => {
                self.emit_opcode(Opcode::SuperCallPrepare);

                let contains_spread = super_call
                    .arguments()
                    .iter()
                    .any(|arg| matches!(arg, Expression::Spread(_)));

                if contains_spread {
                    self.emit_opcode(Opcode::PushNewArray);
                    for arg in super_call.arguments() {
                        self.compile_expr(arg, true);
                        if let Expression::Spread(_) = arg {
                            self.emit_opcode(Opcode::GetIterator);
                            self.emit_opcode(Opcode::PushIteratorToArray);
                        } else {
                            self.emit_opcode(Opcode::PushValueToArray);
                        }
                    }
                } else {
                    for arg in super_call.arguments() {
                        self.compile_expr(arg, true);
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

                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::ImportCall(import) => {
                self.compile_expr(import.argument(), true);
                self.emit_opcode(Opcode::ImportCall);
                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::NewTarget => {
                if use_expr {
                    self.emit_opcode(Opcode::NewTarget);
                }
            }
            Expression::ImportMeta => {
                if use_expr {
                    self.emit_opcode(Opcode::ImportMeta);
                }
            }
            Expression::Optional(opt) => {
                self.compile_optional_preserve_this(opt);

                self.emit_opcode(Opcode::Swap);
                self.emit_opcode(Opcode::Pop);

                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::Parenthesized(parenthesized) => {
                self.compile_expr(parenthesized.expression(), use_expr);
            }
            // TODO: try to remove this variant somehow
            Expression::FormalParameterList(_) => unreachable!(),
        }
    }
}
