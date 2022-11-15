use crate::{
    bytecompiler::{ByteCompiler, Literal},
    vm::Opcode,
    JsResult,
};

use boa_ast::expression::{
    literal::{Literal as AstLiteral, TemplateElement, TemplateLiteral},
    operator::Conditional,
};

mod assign;
mod binary;
mod object_literal;
mod unary;

pub(crate) use assign::compile_assign;
pub(crate) use binary::compile_binary;
pub(crate) use object_literal::compile_object_literal;
pub(crate) use unary::compile_unary;

pub(crate) fn compile_literal<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
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

pub(crate) fn compile_conditional<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
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

pub(crate) fn compile_template_literal<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
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
