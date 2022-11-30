use crate::{
    bytecompiler::{ByteCompiler, Literal},
    vm::Opcode,
    JsResult,
};

use boa_ast::{
    expression::{
        literal::{Literal as AstLiteral, TemplateElement, TemplateLiteral},
        operator::Conditional, access::{PropertyAccessField, PropertyAccess},
    },
    Expression,
};

mod assign;
mod binary;
mod object_literal;
mod unary;

use assign::compile_assign;
use binary::compile_binary;
use boa_interner::Sym;
use object_literal::compile_object_literal;
use unary::compile_unary;

use super::{Callable, NodeKind, Access};

fn compile_literal(
    byte_compiler: &mut ByteCompiler<'_>,
    lit: &AstLiteral,
    use_expr: bool,
) {
    match lit {
        AstLiteral::String(v) => byte_compiler.emit_push_literal(Literal::String(
            byte_compiler
                .interner()
                .resolve_expect(*v)
                .into_common(false),
        )),
        AstLiteral::Int(v) => byte_compiler.emit_push_integer(*v),
        AstLiteral::Num(v) => byte_compiler.emit_push_rational(*v),
        AstLiteral::BigInt(v) => {
            byte_compiler.emit_push_literal(Literal::BigInt(v.clone().into()));
        }
        AstLiteral::Bool(true) => byte_compiler.emit(Opcode::PushTrue, &[]),
        AstLiteral::Bool(false) => byte_compiler.emit(Opcode::PushFalse, &[]),
        AstLiteral::Null => byte_compiler.emit(Opcode::PushNull, &[]),
        AstLiteral::Undefined => byte_compiler.emit(Opcode::PushUndefined, &[]),
    }

    if !use_expr {
        byte_compiler.emit(Opcode::Pop, &[]);
    }
}

fn compile_conditional(
    byte_compiler: &mut ByteCompiler<'_>,
    op: &Conditional,
    use_expr: bool,
) -> JsResult<()> {
    byte_compiler.compile_expr(op.condition(), true)?;
    let jelse = byte_compiler.jump_if_false();
    byte_compiler.compile_expr(op.if_true(), true)?;
    let exit = byte_compiler.jump();
    byte_compiler.patch_jump(jelse);
    byte_compiler.compile_expr(op.if_false(), true)?;
    byte_compiler.patch_jump(exit);

    if !use_expr {
        byte_compiler.emit(Opcode::Pop, &[]);
    };

    Ok(())
}

fn compile_template_literal(
    byte_compiler: &mut ByteCompiler<'_>,
    template_literal: &TemplateLiteral,
    use_expr: bool,
) -> JsResult<()> {
    for element in template_literal.elements() {
        match element {
            TemplateElement::String(s) => byte_compiler.emit_push_literal(Literal::String(
                byte_compiler
                    .interner()
                    .resolve_expect(*s)
                    .into_common(false),
            )),
            TemplateElement::Expr(expr) => {
                byte_compiler.compile_expr(expr, true)?;
            }
        }
    }

    byte_compiler.emit(
        Opcode::ConcatToString,
        &[template_literal.elements().len() as u32],
    );

    if !use_expr {
        byte_compiler.emit(Opcode::Pop, &[]);
    }

    Ok(())
}

pub(crate) fn compile_expr_impl(
    byte_compiler: &mut ByteCompiler<'_>,
    expr: &Expression,
    use_expr: bool,
) -> JsResult<()> {
    match expr {
        Expression::Literal(lit) => compile_literal(byte_compiler, lit, use_expr),
        Expression::Unary(unary) => compile_unary(byte_compiler, unary, use_expr)?,
        Expression::Binary(binary) => compile_binary(byte_compiler, binary, use_expr)?,
        Expression::Assign(assign) => compile_assign(byte_compiler, assign, use_expr)?,
        Expression::ObjectLiteral(object) => {
            compile_object_literal(byte_compiler, object, use_expr)?;
        }
        Expression::Identifier(name) => {
            byte_compiler.access_get(Access::Variable { name: *name }, use_expr)?;
        }
        Expression::PropertyAccess(access) => {
            byte_compiler.access_get(Access::Property { access }, use_expr)?;
        }
        Expression::Conditional(op) => {
            compile_conditional(byte_compiler, op, use_expr)?
        }
        Expression::ArrayLiteral(array) => {
            byte_compiler.emit_opcode(Opcode::PushNewArray);
            byte_compiler.emit_opcode(Opcode::PopOnReturnAdd);

            for element in array.as_ref() {
                if let Some(element) = element {
                    byte_compiler.compile_expr(element, true)?;
                    if let Expression::Spread(_) = element {
                        byte_compiler.emit_opcode(Opcode::InitIterator);
                        byte_compiler.emit_opcode(Opcode::PushIteratorToArray);
                    } else {
                        byte_compiler.emit_opcode(Opcode::PushValueToArray);
                    }
                } else {
                    byte_compiler.emit_opcode(Opcode::PushElisionToArray);
                }
            }

            byte_compiler.emit_opcode(Opcode::PopOnReturnSub);
            if !use_expr {
                byte_compiler.emit(Opcode::Pop, &[]);
            }
        }
        Expression::This => {
            byte_compiler.access_get(Access::This, use_expr)?;
        }
        Expression::Spread(spread) => byte_compiler.compile_expr(spread.target(), true)?,
        Expression::Function(function) => {
            byte_compiler.function(function.into(), NodeKind::Expression, use_expr)?;
        }
        Expression::ArrowFunction(function) => {
            byte_compiler.function(function.into(), NodeKind::Expression, use_expr)?;
        }
        Expression::AsyncArrowFunction(function) => {
            byte_compiler.function(function.into(), NodeKind::Expression, use_expr)?;
        }
        Expression::Generator(function) => {
            byte_compiler.function(function.into(), NodeKind::Expression, use_expr)?;
        }
        Expression::AsyncFunction(function) => {
            byte_compiler.function(function.into(), NodeKind::Expression, use_expr)?;
        }
        Expression::AsyncGenerator(function) => {
            byte_compiler.function(function.into(), NodeKind::Expression, use_expr)?;
        }
        Expression::Call(call) => byte_compiler.call(Callable::Call(call), use_expr)?,
        Expression::New(new) => byte_compiler.call(Callable::New(new), use_expr)?,
        Expression::TemplateLiteral(template_literal) => {
            compile_template_literal(byte_compiler, template_literal, use_expr)?;
        }
        Expression::Await(expr) => {
            byte_compiler.compile_expr(expr.target(), true)?;
            byte_compiler.emit_opcode(Opcode::Await);
            byte_compiler.emit_opcode(Opcode::GeneratorNext);
            if !use_expr {
                byte_compiler.emit_opcode(Opcode::Pop);
            }
        }
        Expression::Yield(r#yield) => {
            if let Some(expr) = r#yield.target() {
                byte_compiler.compile_expr(expr, true)?;
            } else {
                byte_compiler.emit_opcode(Opcode::PushUndefined);
            }

            if r#yield.delegate() {
                if byte_compiler.in_async_generator {
                    byte_compiler.emit_opcode(Opcode::InitIteratorAsync);
                } else {
                    byte_compiler.emit_opcode(Opcode::InitIterator);
                }
                byte_compiler.emit_opcode(Opcode::PushUndefined);
                let start_address = byte_compiler.next_opcode_location();
                let start = byte_compiler.emit_opcode_with_operand(Opcode::GeneratorNextDelegate);
                byte_compiler.emit(Opcode::Jump, &[start_address]);
                byte_compiler.patch_jump(start);
            } else if byte_compiler.in_async_generator {
                byte_compiler.emit_opcode(Opcode::Await);
                byte_compiler.emit_opcode(Opcode::AsyncGeneratorNext);
                let jump_return = byte_compiler.emit_opcode_with_operand(Opcode::JumpIfFalse);
                let jump = byte_compiler.emit_opcode_with_operand(Opcode::JumpIfFalse);
                byte_compiler.emit_opcode(Opcode::Yield);
                byte_compiler.emit_opcode(Opcode::GeneratorNext);
                byte_compiler.patch_jump(jump);
                byte_compiler.emit_opcode(Opcode::Await);
                byte_compiler.emit_opcode(Opcode::GeneratorNext);
                byte_compiler.patch_jump(jump_return);
            } else {
                byte_compiler.emit_opcode(Opcode::Yield);
                byte_compiler.emit_opcode(Opcode::GeneratorNext);
            }

            if !use_expr {
                byte_compiler.emit_opcode(Opcode::Pop);
            }
        }
        Expression::TaggedTemplate(template) => {
            match template.tag() {
                Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                    byte_compiler.compile_expr(access.target(), true)?;
                    byte_compiler.emit(Opcode::Dup, &[]);
                    match access.field() {
                        PropertyAccessField::Const(field) => {
                            let index = byte_compiler.get_or_insert_name((*field).into());
                            byte_compiler.emit(Opcode::GetPropertyByName, &[index]);
                        }
                        PropertyAccessField::Expr(field) => {
                            byte_compiler.compile_expr(field, true)?;
                            byte_compiler.emit(Opcode::GetPropertyByValue, &[]);
                        }
                    }
                }
                Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                    byte_compiler.compile_expr(access.target(), true)?;
                    byte_compiler.emit(Opcode::Dup, &[]);
                    let index = byte_compiler.get_or_insert_name(access.field().into());
                    byte_compiler.emit(Opcode::GetPrivateField, &[index]);
                }
                expr => {
                    byte_compiler.compile_expr(expr, true)?;
                    byte_compiler.emit_opcode(Opcode::This);
                    byte_compiler.emit_opcode(Opcode::Swap);
                }
            }

            byte_compiler.emit_opcode(Opcode::PushNewArray);
            for cooked in template.cookeds() {
                if let Some(cooked) = cooked {
                    byte_compiler.emit_push_literal(Literal::String(
                        byte_compiler
                            .interner()
                            .resolve_expect(*cooked)
                            .into_common(false),
                    ));
                } else {
                    byte_compiler.emit_opcode(Opcode::PushUndefined);
                }
                byte_compiler.emit_opcode(Opcode::PushValueToArray);
            }
            byte_compiler.emit_opcode(Opcode::Dup);

            byte_compiler.emit_opcode(Opcode::PushNewArray);
            for raw in template.raws() {
                byte_compiler.emit_push_literal(Literal::String(
                    byte_compiler
                        .interner()
                        .resolve_expect(*raw)
                        .into_common(false),
                ));
                byte_compiler.emit_opcode(Opcode::PushValueToArray);
            }

            let index = byte_compiler.get_or_insert_name(Sym::RAW.into());
            byte_compiler.emit(Opcode::SetPropertyByName, &[index]);
            byte_compiler.emit(Opcode::Pop, &[]);

            for expr in template.exprs() {
                byte_compiler.compile_expr(expr, true)?;
            }

            byte_compiler.emit(Opcode::Call, &[(template.exprs().len() + 1) as u32]);
        }
        Expression::Class(class) => byte_compiler.class(class, true)?,
        Expression::SuperCall(super_call) => {
            let contains_spread = super_call
                .arguments()
                .iter()
                .any(|arg| matches!(arg, Expression::Spread(_)));

            if contains_spread {
                byte_compiler.emit_opcode(Opcode::PushNewArray);
                for arg in super_call.arguments() {
                    byte_compiler.compile_expr(arg, true)?;
                    if let Expression::Spread(_) = arg {
                        byte_compiler.emit_opcode(Opcode::InitIterator);
                        byte_compiler.emit_opcode(Opcode::PushIteratorToArray);
                    } else {
                        byte_compiler.emit_opcode(Opcode::PushValueToArray);
                    }
                }
            } else {
                for arg in super_call.arguments() {
                    byte_compiler.compile_expr(arg, true)?;
                }
            }

            if contains_spread {
                byte_compiler.emit_opcode(Opcode::SuperCallSpread);
            } else {
                byte_compiler.emit(Opcode::SuperCall, &[super_call.arguments().len() as u32]);
            }

            if !use_expr {
                byte_compiler.emit_opcode(Opcode::Pop);
            }
        }
        Expression::NewTarget => {
            if use_expr {
                byte_compiler.emit_opcode(Opcode::PushNewTarget);
            }
        }
        Expression::Optional(opt) => {
            byte_compiler.compile_optional_preserve_this(opt)?;

            byte_compiler.emit_opcode(Opcode::Swap);
            byte_compiler.emit_opcode(Opcode::Pop);

            if !use_expr {
                byte_compiler.emit_opcode(Opcode::Pop);
            }
        }
        // TODO: try to remove this variant somehow
        Expression::FormalParameterList(_) => unreachable!(),
    }

    Ok(())
}
