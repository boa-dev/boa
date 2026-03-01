mod assign;
mod binary;
mod object_literal;
mod unary;
mod update;

use std::ops::Deref;

use super::{Access, Callable, NodeKind, Register, ToJsString};
use crate::vm::opcode::{
    Await, BindThisValue, Call, ConcatToString, GeneratorDelegateNext, GeneratorDelegateResume,
    GeneratorNext, GeneratorYield, GetAsyncIterator, GetIterator, GetPrivateField,
    GetPropertyByValue, ImportCall, ImportMeta, IteratorResult, IteratorValue, Jump, NewTarget,
    Pop, PushElisionToArray, PushFalse, PushIteratorToArray, PushNewArray, PushNull, PushRegexp,
    PushTrue, PushUndefined, PushValueToArray, SuperCall, SuperCallPrepare, SuperCallSpread,
    TemplateCreate,
};
use crate::{
    bytecompiler::{ByteCompiler, Literal},
    vm::GeneratorResumeKind,
};
use boa_ast::{
    Expression,
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        literal::{
            Literal as AstLiteral, LiteralKind as AstLiteralKind, TemplateElement, TemplateLiteral,
        },
        operator::Conditional,
    },
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
            AstLiteralKind::Bool(true) => PushTrue::emit(self, dst.variable()),
            AstLiteralKind::Bool(false) => PushFalse::emit(self, dst.variable()),
            AstLiteralKind::Null => PushNull::emit(self, dst.variable()),
            AstLiteralKind::Undefined => PushUndefined::emit(self, dst.variable()),
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
        ConcatToString::emit(self, dst.variable(), values);
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
                PushRegexp::emit(
                    self,
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

                PushNewArray::emit(self, dst.variable());

                for element in literal.as_ref() {
                    if let Some(element) = element {
                        self.compile_expr(element, &value);
                        if let Expression::Spread(_) = element {
                            GetIterator::emit(self, value.variable());
                            PushIteratorToArray::emit(self, value.variable());
                        } else {
                            PushValueToArray::emit(self, value.variable(), dst.variable());
                        }
                    } else {
                        PushElisionToArray::emit(self, dst.variable());
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
                Await::emit(self, dst.variable());
                let resume_kind = self.register_allocator.alloc();
                self.pop_into_register(&resume_kind);
                self.pop_into_register(dst);
                GeneratorNext::emit(self, resume_kind.variable(), dst.variable());
                self.register_allocator.dealloc(resume_kind);
            }
            Expression::Yield(r#yield) => {
                if let Some(expr) = r#yield.target() {
                    self.compile_expr(expr, dst);
                } else {
                    PushUndefined::emit(self, dst.variable());
                }

                if r#yield.delegate() {
                    if self.is_async() {
                        GetAsyncIterator::emit(self, dst.variable());
                    } else {
                        GetIterator::emit(self, dst.variable());
                    }

                    let resume_kind = self.register_allocator.alloc();
                    let is_return = self.register_allocator.alloc();
                    PushUndefined::emit(self, dst.variable());
                    self.emit_resume_kind(GeneratorResumeKind::Normal, &resume_kind);

                    let start_address = self.next_opcode_location();

                    let generator_delegate_next_label = self.next_opcode_location();
                    GeneratorDelegateNext::emit(
                        self,
                        Self::DUMMY_ADDRESS,
                        Self::DUMMY_ADDRESS,
                        dst.variable(),
                        resume_kind.variable(),
                        is_return.variable(),
                    );

                    if self.is_async() {
                        Await::emit(self, dst.variable());
                        self.pop_into_register(&resume_kind);
                        self.pop_into_register(dst);
                    } else {
                        self.emit_resume_kind(GeneratorResumeKind::Normal, &resume_kind);
                    }

                    let generator_delegate_resume_label = self.next_opcode_location();
                    GeneratorDelegateResume::emit(
                        self,
                        Self::DUMMY_ADDRESS,
                        Self::DUMMY_ADDRESS,
                        dst.variable(),
                        resume_kind.variable(),
                        is_return.variable(),
                    );

                    if self.is_async() {
                        IteratorValue::emit(self, dst.variable());
                        self.async_generator_yield(dst, &resume_kind);
                    } else {
                        IteratorResult::emit(self, dst.variable());
                        GeneratorYield::emit(self, dst.variable());
                        self.pop_into_register(&resume_kind);
                        self.pop_into_register(dst);
                    }
                    Jump::emit(self, start_address);

                    self.register_allocator.dealloc(resume_kind);
                    self.register_allocator.dealloc(is_return);

                    let generator_delegate_resume_return = self.next_opcode_location();
                    let generator_delegate_next_return = self.next_opcode_location();

                    if self.is_async() {
                        Await::emit(self, dst.variable());
                        Pop::emit(self);
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
                                self.emit_get_property_by_name(&function, None, &this, ident.sym());
                            }
                            PropertyAccessField::Expr(field) => {
                                let key = self.register_allocator.alloc();
                                self.compile_expr(field, &key);
                                GetPropertyByValue::emit(
                                    self,
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
                        GetPrivateField::emit(
                            self,
                            function.variable(),
                            this.variable(),
                            index.into(),
                        );
                    }
                    expr => {
                        PushUndefined::emit(self, this.variable());
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
                        PushUndefined::emit(self, value.variable());
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
                TemplateCreate::emit(self, site, dst.variable(), values);
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

                Call::emit(self, (template.exprs().len() as u32 + 1).into());
                self.pop_into_register(dst);
            }
            Expression::ClassExpression(class) => {
                self.compile_class(class.deref().into(), Some(dst));
            }
            Expression::SuperCall(super_call) => {
                let this = self.register_allocator.alloc();
                let value = self.register_allocator.alloc();
                SuperCallPrepare::emit(self, value.variable());
                PushUndefined::emit(self, this.variable());
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

                    PushNewArray::emit(self, array.variable());

                    for arg in super_call.arguments() {
                        self.compile_expr(arg, &value);
                        if let Expression::Spread(_) = arg {
                            GetIterator::emit(self, value.variable());
                            PushIteratorToArray::emit(self, array.variable());
                        } else {
                            PushValueToArray::emit(self, value.variable(), array.variable());
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
                    SuperCallSpread::emit(self);
                } else {
                    SuperCall::emit(self, (super_call.arguments().len() as u32).into());
                }
                self.pop_into_register(dst);
                BindThisValue::emit(self, dst.variable());
            }
            Expression::ImportCall(import) => {
                self.compile_expr(import.specifier(), dst);
                let options = self.register_allocator.alloc();
                if let Some(opts) = import.options() {
                    self.compile_expr(opts, &options);
                } else {
                    PushUndefined::emit(self, options.variable());
                }
                ImportCall::emit(self, dst.variable(), options.variable());
                self.register_allocator.dealloc(options);
            }
            Expression::NewTarget(_new_target) => {
                NewTarget::emit(self, dst.variable());
            }
            Expression::ImportMeta(_import_meta) => {
                ImportMeta::emit(self, dst.variable());
            }
            Expression::Optional(opt) => {
                let this = self.register_allocator.alloc();
                self.compile_optional_preserve_this(opt, &this, dst);
                self.register_allocator.dealloc(this);
            }
            Expression::Parenthesized(parenthesized) => {
                self.compile_expr(parenthesized.expression(), dst);
            }
        }
    }
}
