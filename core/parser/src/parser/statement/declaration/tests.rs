use crate::parser::tests::{check_invalid_script, check_module_parser, check_script_parser};
use boa_ast::{
    declaration::{
        ExportDeclaration, ExportSpecifier, LexicalDeclaration, VarDeclaration, Variable,
    },
    expression::{
        literal::{Literal, LiteralKind},
        Identifier,
    },
    Declaration, ModuleItem, Span, Statement,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;
use indoc::indoc;

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a = 5;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 5), (1, 6)),
                ),
                Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var yield = 5;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                Identifier::new(Sym::YIELD, Span::new((1, 5), (1, 10))),
                Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "var await = 5;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                Identifier::new(Sym::AWAIT, Span::new((1, 5), (1, 10))),
                Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a=5;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 5), (1, 6)),
                ),
                Some(Literal::new(5, Span::new((1, 7), (1, 8))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn empty_var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 5), (1, 6)),
                ),
                None,
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a = 5, b, c = 6;",
        vec![Statement::Var(VarDeclaration(
            vec![
                Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
                ),
                Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("b", utf16!("b")),
                        Span::new((1, 12), (1, 13)),
                    ),
                    None,
                ),
                Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("c", utf16!("c")),
                        Span::new((1, 15), (1, 16)),
                    ),
                    Some(Literal::new(6, Span::new((1, 19), (1, 20))).into()),
                ),
            ]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing.
#[test]
fn let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 5), (1, 6)),
                ),
                Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let yield = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(Sym::YIELD, Span::new((1, 5), (1, 10))),
                Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "let await = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(Sym::AWAIT, Span::new((1, 5), (1, 10))),
                Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a=5;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 5), (1, 6)),
                ),
                Some(Literal::new(5, Span::new((1, 7), (1, 8))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn empty_let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 5), (1, 6)),
                ),
                None,
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a = 5, b, c = 6;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![
                Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
                ),
                Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("b", utf16!("b")),
                        Span::new((1, 12), (1, 13)),
                    ),
                    None,
                ),
                Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("c", utf16!("c")),
                        Span::new((1, 15), (1, 16)),
                    ),
                    Some(Literal::new(6, Span::new((1, 19), (1, 20))).into()),
                ),
            ]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing.
#[test]
fn const_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const a = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 7), (1, 8)),
                ),
                Some(Literal::new(5, Span::new((1, 11), (1, 12))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const yield = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                Identifier::new(Sym::YIELD, Span::new((1, 7), (1, 12))),
                Some(Literal::new(5, Span::new((1, 15), (1, 16))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "const await = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                Identifier::new(Sym::AWAIT, Span::new((1, 7), (1, 12))),
                Some(Literal::new(5, Span::new((1, 15), (1, 16))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const a=5;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 7), (1, 8)),
                ),
                Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks empty `const` declaration parsing.
#[test]
fn empty_const_declaration() {
    check_invalid_script("const a;");
}

/// Checks multiple `const` declarations.
#[test]
fn multiple_const_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const a = 5, c = 6;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![
                Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 7), (1, 8)),
                    ),
                    Some(Literal::new(5, Span::new((1, 11), (1, 12))).into()),
                ),
                Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("c", utf16!("c")),
                        Span::new((1, 14), (1, 15)),
                    ),
                    Some(Literal::new(6, Span::new((1, 18), (1, 19))).into()),
                ),
            ]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `LexicalDeclaration` early errors.
#[test]
fn lexical_declaration_early_errors() {
    check_invalid_script("let let = 0");
    check_invalid_script("let a = 0, a = 0");
    check_invalid_script("const a = 0, a = 0");
    check_invalid_script("for (let let = 0; ; ) {}");
    check_invalid_script("for (let a = 0, a = 0; ; ) {}");
    check_invalid_script("for (const a = 0, a = 0; ; ) {}");
}

/// Checks module exports with reserved keywords
#[test]
fn module_export_reserved() {
    let interner = &mut Interner::default();
    let val = interner.get_or_intern_static("val", utf16!("val"));
    check_module_parser(
        indoc! {"
            const val = null;
            export { val as null, val as true, val as false };
        "},
        vec![
            ModuleItem::StatementListItem(
                Declaration::Lexical(LexicalDeclaration::Const(
                    vec![Variable::from_identifier(
                        Identifier::new(val, Span::new((1, 7), (1, 10))),
                        Some(Literal::new(LiteralKind::Null, Span::new((1, 13), (1, 17))).into()),
                    )]
                    .try_into()
                    .unwrap(),
                ))
                .into(),
            ),
            ModuleItem::ExportDeclaration(ExportDeclaration::List(
                vec![
                    ExportSpecifier::new(Sym::NULL, val, false),
                    ExportSpecifier::new(Sym::TRUE, val, false),
                    ExportSpecifier::new(Sym::FALSE, val, false),
                ]
                .into(),
            )),
        ],
        interner,
    );
}
