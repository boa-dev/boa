mod assign;
mod binary;
mod object_literal;
mod unary;
mod update;

use std::ops::Deref;

use super::{Access, Callable, NodeKind, Register, ToJsString};
use crate::{
    bytecompiler::{ByteCompiler, Literal},
    vm::GeneratorResumeKind,
};
use boa_ast::{
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        literal::{
            Literal as AstLiteral, LiteralKind as AstLiteralKind, TemplateElement, TemplateLiteral,
        },
        operator::Conditional,
    },
    Expression,
};
use thin_vec::ThinVec;

impl ByteCompiler<'_> {
    fn compile_literal(&mut self, lit: &AstLiteral, dst: &Register) {
        match lit.kind() {
            AstLiteralKind::String(v) => {
                self.emit_push_literal(Literal::String(v.to_js_string(self.interner())), dst);
            }
            AstLiteralKind::Int(v) => self.emit_push_integer(*v, dst),
            AstLiteralKind::Num(v) => self.emit_push_rational(*v, dst),
            AstLiteralKind::BigInt(v) => {
                self.emit_push_literal(Literal::BigInt(v.clone().into()), dst);
            }
            AstLiteralKind::Bool(true) => self.bytecode.emit_push_true(dst.variable()),
            AstLiteralKind::Bool(false) => self.bytecode.emit_push_false(dst.variable()),
            AstLiteralKind::Null => self.bytecode.emit_push_null(dst.variable()),
            AstLiteralKind::Undefined => self.bytecode.emit_push_undefined(dst.variable()),
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

        let mut values = ThinVec::with_capacity(registers.len());
        for reg in &registers {
            values.push(reg.variable());
        }
        self.bytecode.emit_concat_to_string(dst.variable(), values);
        for reg in registers {
            self.register_allocator.dealloc(reg);
        }
    }

    pub(crate) fn compile_expr_impl(&mut self, expr: &Expression, dst: &Register) {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit, dst),
            Expression::RegExpLiteral(regexp) => {
                let pattern_index = self.get_or_insert_name(regexp.pattern());
                let flags_index = self.get_or_insert_name(regexp.flags());
                self.bytecode.emit_push_regexp(
                    dst.variable(),
                    pattern_index.into(),
                    flags_index.into(),
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

                self.bytecode.emit_push_new_array(dst.variable());

                for element in literal.as_ref() {
                    if let Some(element) = element {
                        self.compile_expr(element, &value);
                        if let Expression::Spread(_) = element {
                            self.bytecode.emit_get_iterator(value.variable());
                            self.bytecode.emit_push_iterator_to_array(dst.variable());
                        } else {
                            self.bytecode
                                .emit_push_value_to_array(value.variable(), dst.variable());
                        }
                    } else {
                        self.bytecode.emit_push_elision_to_array(dst.variable());
                    }
                }
                self.register_allocator.dealloc(value);
            }
            Expression::This(_this) => self.access_get(Access::This, dst),
            Expression::Spread(spread) => self.compile_expr(spread.target(), dst),
            Expression::FunctionExpression(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst);
            }
            Expression::ArrowFunction(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst);
            }
            Expression::AsyncArrowFunction(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst);
            }
            Expression::GeneratorExpression(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst);
            }
            Expression::AsyncFunctionExpression(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst);
            }
            Expression::AsyncGeneratorExpression(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst);
            }
            Expression::Call(call) => self.call(Callable::Call(call), dst),
            Expression::New(new) => self.call(Callable::New(new), dst),
            Expression::TemplateLiteral(template_literal) => {
                self.compile_template_literal(template_literal, dst);
            }
            Expression::Await(expr) => {
                self.compile_expr(expr.target(), dst);
                self.bytecode.emit_await(dst.variable());
                let resume_kind = self.register_allocator.alloc();
                self.pop_into_register(&resume_kind);
                self.pop_into_register(dst);
                self.bytecode
                    .emit_generator_next(resume_kind.variable(), dst.variable());
                self.register_allocator.dealloc(resume_kind);
            }
            Expression::Yield(r#yield) => {
                if let Some(expr) = r#yield.target() {
                    self.compile_expr(expr, dst);
                } else {
                    self.bytecode.emit_push_undefined(dst.variable());
                }

                if r#yield.delegate() {
                    if self.is_async() {
                        self.bytecode.emit_get_async_iterator(dst.variable());
                    } else {
                        self.bytecode.emit_get_iterator(dst.variable());
                    }

                    let resume_kind = self.register_allocator.alloc();
                    let is_return = self.register_allocator.alloc();
                    self.bytecode.emit_push_undefined(dst.variable());
                    self.emit_resume_kind(GeneratorResumeKind::Normal, &resume_kind);

                    let start_address = self.next_opcode_location();

                    let generator_delegate_next_label = self.next_opcode_location();
                    self.bytecode.emit_generator_delegate_next(
                        Self::DUMMY_ADDRESS,
                        Self::DUMMY_ADDRESS,
                        dst.variable(),
                        resume_kind.variable(),
                        is_return.variable(),
                    );

                    if self.is_async() {
                        self.bytecode.emit_await(dst.variable());
                        self.pop_into_register(&resume_kind);
                        self.pop_into_register(dst);
                    } else {
                        self.emit_resume_kind(GeneratorResumeKind::Normal, &resume_kind);
                    }

                    let generator_delegate_resume_label = self.next_opcode_location();
                    self.bytecode.emit_generator_delegate_resume(
                        Self::DUMMY_ADDRESS,
                        Self::DUMMY_ADDRESS,
                        dst.variable(),
                        resume_kind.variable(),
                        is_return.variable(),
                    );

                    if self.is_async() {
                        self.bytecode.emit_iterator_value(dst.variable());
                        self.async_generator_yield(dst, &resume_kind);
                    } else {
                        self.bytecode.emit_iterator_result(dst.variable());
                        self.bytecode.emit_generator_yield(dst.variable());
                        self.pop_into_register(&resume_kind);
                        self.pop_into_register(dst);
                    }
                    self.bytecode.emit_jump(start_address);

                    self.register_allocator.dealloc(resume_kind);
                    self.register_allocator.dealloc(is_return);

                    let generator_delegate_resume_return = self.next_opcode_location();
                    let generator_delegate_next_return = self.next_opcode_location();

                    if self.is_async() {
                        self.bytecode.emit_await(dst.variable());
                        self.bytecode.emit_pop();
                    } else {
                        self.push_from_register(dst);
                    }
                    self.close_active_iterators();

                    self.r#return(true);

                    let generator_delegate_next_throw = self.next_opcode_location();

                    self.iterator_close(self.is_async());
                    self.emit_type_error("iterator does not have a throw method");

                    let generator_delegate_resume_exit = self.next_opcode_location();
                    self.bytecode.patch_jump_two_addresses(
                        generator_delegate_resume_label,
                        (
                            generator_delegate_resume_return,
                            generator_delegate_resume_exit,
                        ),
                    );
                    self.bytecode.patch_jump_two_addresses(
                        generator_delegate_next_label,
                        (
                            generator_delegate_next_throw,
                            generator_delegate_next_return,
                        ),
                    );
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
                                self.emit_get_property_by_name(
                                    &function,
                                    &this,
                                    &this,
                                    ident.sym(),
                                );
                            }
                            PropertyAccessField::Expr(field) => {
                                let key = self.register_allocator.alloc();
                                self.compile_expr(field, &key);
                                self.bytecode.emit_get_property_by_value(
                                    function.variable(),
                                    key.variable(),
                                    this.variable(),
                                    this.variable(),
                                );
                                self.register_allocator.dealloc(key);
                            }
                        }
                    }
                    Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                        let index = self.get_or_insert_private_name(access.field());
                        self.compile_expr(access.target(), &this);
                        self.bytecode.emit_get_private_field(
                            function.variable(),
                            this.variable(),
                            index.into(),
                        );
                    }
                    expr => {
                        self.bytecode.emit_push_undefined(this.variable());
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
                        self.bytecode.emit_push_undefined(value.variable());
                    }
                    part_registers.push(value);
                    let value = self.register_allocator.alloc();
                    self.emit_push_literal(
                        Literal::String(raw.to_js_string(self.interner())),
                        &value,
                    );
                    part_registers.push(value);
                }

                let mut values = ThinVec::with_capacity(count as usize * 2);
                for r in &part_registers {
                    values.push(r.index());
                }
                self.bytecode
                    .emit_template_create(site, dst.variable(), values);
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

                self.bytecode
                    .emit_call((template.exprs().len() as u32 + 1).into());
                self.pop_into_register(dst);
            }
            Expression::ClassExpression(class) => {
                self.compile_class(class.deref().into(), Some(dst));
            }
            Expression::SuperCall(super_call) => {
                let this = self.register_allocator.alloc();
                let value = self.register_allocator.alloc();
                self.bytecode.emit_super_call_prepare(value.variable());
                self.bytecode.emit_push_undefined(this.variable());
                self.push_from_register(&this);
                self.push_from_register(&value);
                self.register_allocator.dealloc(this);
                self.register_allocator.dealloc(value);

                let contains_spread = super_call
                    .arguments()
                    .iter()
                    .any(|arg| matches!(arg, Expression::Spread(_)));

                if contains_spread {
                    let array = self.register_allocator.alloc();
                    let value = self.register_allocator.alloc();

                    self.bytecode.emit_push_new_array(array.variable());

                    for arg in super_call.arguments() {
                        self.compile_expr(arg, &value);
                        if let Expression::Spread(_) = arg {
                            self.bytecode.emit_get_iterator(value.variable());
                            self.bytecode.emit_push_iterator_to_array(array.variable());
                        } else {
                            self.bytecode
                                .emit_push_value_to_array(value.variable(), array.variable());
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
                    self.bytecode.emit_super_call_spread();
                } else {
                    self.bytecode
                        .emit_super_call((super_call.arguments().len() as u32).into());
                }
                self.pop_into_register(dst);
                self.bytecode.emit_bind_this_value(dst.variable());
            }
            Expression::ImportCall(import) => {
                self.compile_expr(import.argument(), dst);
                self.bytecode.emit_import_call(dst.variable());
            }
            Expression::NewTarget(_new_target) => {
                self.bytecode.emit_new_target(dst.variable());
            }
            Expression::ImportMeta(_import_meta) => {
                self.bytecode.emit_import_meta(dst.variable());
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
