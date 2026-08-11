use crate::parser::tests::{check_invalid_script, check_module_parser, check_script_parser};
use crate::{Parser, Source};
use boa_ast::{
    Declaration, LinearPosition, LinearSpan, ModuleItem, Span, Statement, StatementList,
    StatementListItem,
    declaration::{
        ExportDeclaration, ExportSpecifier, ImportAttribute, ImportDeclaration, ImportKind,
        LexicalDeclaration, ModuleSpecifier, ReExportKind, VarDeclaration, Variable,
    },
    expression::{
        Await, Call, Identifier, Parenthesized,
        literal::{Literal, LiteralKind},
        operator::{Binary, binary::ArithmeticOp},
    },
    function::{
        AsyncArrowFunction, AsyncFunctionDeclaration, FormalParameter, FormalParameterList,
        FunctionBody,
    },
    pattern::{ArrayPattern, ArrayPatternElement, ObjectPattern, ObjectPatternElement, Pattern},
    statement::Return,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;
use indoc::indoc;

const EMPTY_LINEAR_SPAN: LinearSpan =
    LinearSpan::new(LinearPosition::new(0), LinearPosition::new(0));
const PSEUDO_LINEAR_POS: LinearPosition = LinearPosition::new(0);

#[track_caller]
fn check_valid_module(js: &str) {
    assert!(
        Parser::new(Source::from_bytes(js))
            .parse_module(
                &boa_ast::scope::Scope::new_global(),
                &mut Interner::default()
            )
            .is_ok(),
        "expected module to parse successfully:\n{js}"
    );
}

#[track_caller]
fn check_invalid_module(js: &str) {
    assert!(
        Parser::new(Source::from_bytes(js))
            .parse_module(
                &boa_ast::scope::Scope::new_global(),
                &mut Interner::default()
            )
            .is_err(),
        "expected module to fail parsing:\n{js}"
    );
}

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a = 5;",
        vec![
            Statement::Var(VarDeclaration(
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
            .into(),
        ],
        interner,
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var yield = 5;",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::YIELD, Span::new((1, 5), (1, 10))),
                    Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "var await = 5;",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::AWAIT, Span::new((1, 5), (1, 10))),
                    Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a=5;",
        vec![
            Statement::Var(VarDeclaration(
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
            .into(),
        ],
        interner,
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn empty_var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a;",
        vec![
            Statement::Var(VarDeclaration(
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
            .into(),
        ],
        interner,
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a = 5, b, c = 6;",
        vec![
            Statement::Var(VarDeclaration(
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
            .into(),
        ],
        interner,
    );
}

/// Checks `let` declaration parsing.
#[test]
fn let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
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
            .into(),
        ],
        interner,
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let yield = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::YIELD, Span::new((1, 5), (1, 10))),
                    Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "let await = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::AWAIT, Span::new((1, 5), (1, 10))),
                    Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a=5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
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
            .into(),
        ],
        interner,
    );
}

/// Checks `let` declaration with a reserved keyword as identifier.
#[test]
fn let_declaration_reserved_keyword_identifier() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let of = 1;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("of", utf16!("of")),
                        Span::new((1, 5), (1, 7)),
                    ),
                    Some(Literal::new(1, Span::new((1, 10), (1, 11))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn empty_let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
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
            .into(),
        ],
        interner,
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a = 5, b, c = 6;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
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
            .into(),
        ],
        interner,
    );
}

/// Checks `const` declaration parsing.
#[test]
fn const_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const a = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
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
            .into(),
        ],
        interner,
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const yield = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::YIELD, Span::new((1, 7), (1, 12))),
                    Some(Literal::new(5, Span::new((1, 15), (1, 16))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "const await = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::AWAIT, Span::new((1, 7), (1, 12))),
                    Some(Literal::new(5, Span::new((1, 15), (1, 16))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const a=5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
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
            .into(),
        ],
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
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
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
            .into(),
        ],
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
            ModuleItem::ExportDeclaration(
                ExportDeclaration::List(
                    vec![
                        ExportSpecifier::new(Sym::NULL, val, false),
                        ExportSpecifier::new(Sym::TRUE, val, false),
                        ExportSpecifier::new(Sym::FALSE, val, false),
                    ]
                    .into(),
                )
                .into(),
            ),
        ],
        interner,
    );
}

/// Checks import declaration with a single attribute.
#[test]
fn import_with_single_attribute() {
    let interner = &mut Interner::default();
    let json = interner.get_or_intern_static("json", utf16!("json"));
    let foo_json = interner.get_or_intern_static("./foo.json", utf16!("./foo.json"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));

    check_module_parser(
        r#"import json from "./foo.json" with { type: "json" };"#,
        vec![ModuleItem::ImportDeclaration(ImportDeclaration::new(
            Some(Identifier::new(json, Span::new((1, 8), (1, 12)))),
            ImportKind::DefaultOrUnnamed,
            ModuleSpecifier::new(foo_json),
            vec![ImportAttribute::new(type_sym, json)].into(),
        ))],
        interner,
    );
}

/// Checks import declaration with multiple attributes.
#[test]
fn import_with_multiple_attributes() {
    let interner = &mut Interner::default();
    let json = interner.get_or_intern_static("json", utf16!("json"));
    let foo_json = interner.get_or_intern_static("./foo.json", utf16!("./foo.json"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));
    let integrity = interner.get_or_intern_static("integrity", utf16!("integrity"));
    let hash = interner.get_or_intern_static("sha384-abc123", utf16!("sha384-abc123"));

    check_module_parser(
        r#"import json from "./foo.json" with { type: "json", integrity: "sha384-abc123" };"#,
        vec![ModuleItem::ImportDeclaration(ImportDeclaration::new(
            Some(Identifier::new(json, Span::new((1, 8), (1, 12)))),
            ImportKind::DefaultOrUnnamed,
            ModuleSpecifier::new(foo_json),
            vec![
                ImportAttribute::new(type_sym, json),
                ImportAttribute::new(integrity, hash),
            ]
            .into(),
        ))],
        interner,
    );
}

/// Checks import declaration with trailing comma in attributes.
#[test]
fn import_with_trailing_comma_attribute() {
    let interner = &mut Interner::default();
    let json = interner.get_or_intern_static("json", utf16!("json"));
    let foo_json = interner.get_or_intern_static("./foo.json", utf16!("./foo.json"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));

    check_module_parser(
        r#"import json from "./foo.json" with { type: "json", };"#,
        vec![ModuleItem::ImportDeclaration(ImportDeclaration::new(
            Some(Identifier::new(json, Span::new((1, 8), (1, 12)))),
            ImportKind::DefaultOrUnnamed,
            ModuleSpecifier::new(foo_json),
            vec![ImportAttribute::new(type_sym, json)].into(),
        ))],
        interner,
    );
}

/// Checks re-export with attributes.
#[test]
fn reexport_with_attributes() {
    let interner = &mut Interner::default();
    let foo_js = interner.get_or_intern_static("./foo.js", utf16!("./foo.js"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));
    let json = interner.get_or_intern_static("json", utf16!("json"));

    check_module_parser(
        r#"export * from "./foo.js" with { type: "json" };"#,
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::ReExport {
                kind: ReExportKind::Namespaced { name: None },
                specifier: ModuleSpecifier::new(foo_js),
                attributes: vec![ImportAttribute::new(type_sym, json)].into(),
            },
        ))],
        interner,
    );
}

/// Checks import attributes with string literal key.
#[test]
fn import_with_string_literal_key() {
    let interner = &mut Interner::default();
    let json = interner.get_or_intern_static("json", utf16!("json"));
    let foo_json = interner.get_or_intern_static("./foo.json", utf16!("./foo.json"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));

    check_module_parser(
        r#"import json from "./foo.json" with { "type": "json" };"#,
        vec![ModuleItem::ImportDeclaration(ImportDeclaration::new(
            Some(Identifier::new(json, Span::new((1, 8), (1, 12)))),
            ImportKind::DefaultOrUnnamed,
            ModuleSpecifier::new(foo_json),
            vec![ImportAttribute::new(type_sym, json)].into(),
        ))],
        interner,
    );
}

/// Checks that duplicate attribute keys are rejected.
#[test]
fn import_duplicate_attribute_key() {
    assert!(
        Parser::new(Source::from_bytes(
            r#"import json from "./foo.json" with { type: "json", type: "css" };"#
        ))
        .parse_module(
            &boa_ast::scope::Scope::new_global(),
            &mut Interner::default()
        )
        .is_err()
    );
}

/// Checks that non-string attribute values are rejected.
#[test]
fn import_non_string_attribute_value() {
    let scope = boa_ast::scope::Scope::new_global();

    assert!(
        Parser::new(Source::from_bytes(
            r#"import json from "./foo.json" with { type: json };"#
        ))
        .parse_module(&scope, &mut Interner::default())
        .is_err()
    );
    assert!(
        Parser::new(Source::from_bytes(
            r#"import json from "./foo.json" with { type: 123 };"#
        ))
        .parse_module(&scope, &mut Interner::default())
        .is_err()
    );
    assert!(
        Parser::new(Source::from_bytes(
            r#"import json from "./foo.json" with { type: true };"#
        ))
        .parse_module(&scope, &mut Interner::default())
        .is_err()
    );
}

/// Checks `using` declaration parsing.
#[test]
fn using_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "using x = resource;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Using(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("x", utf16!("x")),
                        Span::new((1, 7), (1, 8)),
                    ),
                    Some(
                        Identifier::new(
                            interner.get_or_intern_static("resource", utf16!("resource")),
                            Span::new((1, 11), (1, 19)),
                        )
                        .into(),
                    ),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `using` declaration with multiple bindings.
#[test]
fn using_declaration_multiple() {
    let interner = &mut Interner::default();
    check_script_parser(
        "using a = res1, b = res2;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Using(
                vec![
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("a", utf16!("a")),
                            Span::new((1, 7), (1, 8)),
                        ),
                        Some(
                            Identifier::new(
                                interner.get_or_intern_static("res1", utf16!("res1")),
                                Span::new((1, 11), (1, 15)),
                            )
                            .into(),
                        ),
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("b", utf16!("b")),
                            Span::new((1, 17), (1, 18)),
                        ),
                        Some(
                            Identifier::new(
                                interner.get_or_intern_static("res2", utf16!("res2")),
                                Span::new((1, 21), (1, 25)),
                            )
                            .into(),
                        ),
                    ),
                ]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks that `using` declaration without initializer fails.
#[test]
fn using_declaration_no_init() {
    check_invalid_script("using x;");
}

/// Checks `await using` declaration parsing in async function.
#[test]
fn await_using_declaration() {
    let interner = &mut Interner::default();
    // await using is only valid in async contexts, so we test it inside an async function
    // We just verify it parses without error
    let source = Source::from_bytes("async function f() { await using x = resource; }");
    let mut parser = Parser::new(source);
    let scope = boa_ast::scope::Scope::new_global();
    let result = parser.parse_script(&scope, interner);
    assert!(
        result.is_ok(),
        "Failed to parse await using in async function: {:?}",
        result.err()
    );
}

/// Checks that `await using` declaration without initializer fails.
#[test]
fn await_using_declaration_no_init() {
    // Test in async function context
    check_invalid_script("async function f() { await using x; }");
}

/// Checks that `await using` is only valid in async contexts.
#[test]
fn await_using_requires_async_context() {
    // Should fail in non-async context (top-level script)
    check_invalid_script("await using x = resource;");
}

/// Checks that line terminator between `await` and `using` is rejected.
/// Per spec: <https://arai-a.github.io/ecma262-compare/snapshot.html?pr=3000#prod-AwaitUsingDeclarationHead>
/// There must be [no `LineTerminator` here] between the keywords.
#[test]
fn await_using_no_line_terminator() {
    // Line terminator between await and using should fail
    check_invalid_script("async function f() { await\nusing x = resource; }");

    // Without line terminator should succeed
    let interner = &mut Interner::default();
    let source = Source::from_bytes("async function f() { await using x = resource; }");
    let mut parser = Parser::new(source);
    let scope = boa_ast::scope::Scope::new_global();
    let result = parser.parse_script(&scope, interner);
    assert!(
        result.is_ok(),
        "Failed to parse await using without line terminator: {:?}",
        result.err()
    );
}

/// Checks that destructuring patterns are rejected for `using` declarations.
/// Per spec: <https://tc39.es/proposal-explicit-resource-management/>
/// The grammar uses ~Pattern parameter which means destructuring is NOT allowed.
#[test]
fn using_no_destructuring_object() {
    check_invalid_script("using {x, y} = resource;");
}

/// Checks that destructuring patterns are rejected for `using` declarations.
#[test]
fn using_no_destructuring_array() {
    check_invalid_script("using [a, b] = resource;");
}

/// Checks that destructuring patterns are rejected for `await using` declarations.
#[test]
fn await_using_no_destructuring_object() {
    check_invalid_script("async function f() { await using {x, y} = resource; }");
}

/// Checks that destructuring patterns are rejected for `await using` declarations.
#[test]
fn await_using_no_destructuring_array() {
    check_invalid_script("async function f() { await using [a, b] = resource; }");
}

/// Checks that `using let` is rejected (let is not allowed as a bound name).
#[test]
fn using_let_rejected() {
    check_invalid_script("using let = resource;");
}

/// Checks that duplicate names in `using` declarations are rejected.
#[test]
fn using_duplicate_names() {
    check_invalid_script("using x = r1, x = r2;");
}

/// Checks that `await using` with duplicate names is rejected.
#[test]
fn await_using_duplicate_names() {
    check_invalid_script("async function f() { await using x = r1, x = r2; }");
}

/// Checks that `using` works with valid identifiers.
#[test]
fn using_valid_identifiers() {
    let interner = &mut Interner::default();
    check_script_parser(
        "using x = resource, y = resource2;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Using(
                vec![
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("x", utf16!("x")),
                            Span::new((1, 7), (1, 8)),
                        ),
                        Some(
                            Identifier::new(
                                interner.get_or_intern_static("resource", utf16!("resource")),
                                Span::new((1, 11), (1, 19)),
                            )
                            .into(),
                        ),
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("y", utf16!("y")),
                            Span::new((1, 21), (1, 22)),
                        ),
                        Some(
                            Identifier::new(
                                interner.get_or_intern_static("resource2", utf16!("resource2")),
                                Span::new((1, 25), (1, 34)),
                            )
                            .into(),
                        ),
                    ),
                ]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks that `await using` works with valid identifiers in async context.
#[test]
fn await_using_valid_identifiers() {
    let interner = &mut Interner::default();
    let source = Source::from_bytes("async function f() { await using x = r1, y = r2; }");
    let mut parser = Parser::new(source);
    let scope = boa_ast::scope::Scope::new_global();
    let result = parser.parse_script(&scope, interner);
    assert!(
        result.is_ok(),
        "Failed to parse await using with multiple bindings: {:?}",
        result.err()
    );
}

/// `export default async (x) => x;` must parse as a default `AssignmentExpression`.
#[test]
fn export_default_async_arrow_parenthesized_param() {
    let interner = &mut Interner::default();
    let x = interner.get_or_intern_static("x", utf16!("x"));
    check_module_parser(
        "export default async (x) => x;",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAssignmentExpression(
                AsyncArrowFunction::new(
                    None,
                    FormalParameterList::from(FormalParameter::new(
                        Variable::from_identifier(
                            Identifier::new(x, Span::new((1, 23), (1, 24))),
                            None,
                        ),
                        false,
                    )),
                    FunctionBody::new(
                        StatementList::new(
                            [StatementListItem::Statement(
                                Statement::Return(Return::new(Some(
                                    Identifier::new(x, Span::new((1, 29), (1, 30))).into(),
                                )))
                                .into(),
                            )],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        Span::new((1, 29), (1, 30)),
                    ),
                    EMPTY_LINEAR_SPAN,
                    Span::new((1, 16), (1, 30)),
                )
                .into(),
            ),
        ))],
        interner,
    );
}

/// `export default async x => x;` must parse as a default `AssignmentExpression`.
#[test]
fn export_default_async_arrow_identifier_param() {
    let interner = &mut Interner::default();
    let x = interner.get_or_intern_static("x", utf16!("x"));
    check_module_parser(
        "export default async x => x;",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAssignmentExpression(
                AsyncArrowFunction::new(
                    None,
                    FormalParameterList::from(FormalParameter::new(
                        Variable::from_identifier(
                            Identifier::new(x, Span::new((1, 22), (1, 23))),
                            None,
                        ),
                        false,
                    )),
                    FunctionBody::new(
                        StatementList::new(
                            [StatementListItem::Statement(
                                Statement::Return(Return::new(Some(
                                    Identifier::new(x, Span::new((1, 27), (1, 28))).into(),
                                )))
                                .into(),
                            )],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        Span::new((1, 27), (1, 28)),
                    ),
                    EMPTY_LINEAR_SPAN,
                    Span::new((1, 16), (1, 28)),
                )
                .into(),
            ),
        ))],
        interner,
    );
}

/// `export default async () => 1;` must parse as a default `AssignmentExpression`.
#[test]
fn export_default_async_arrow_no_params() {
    let interner = &mut Interner::default();
    check_module_parser(
        "export default async () => 1;",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAssignmentExpression(
                AsyncArrowFunction::new(
                    None,
                    FormalParameterList::default(),
                    FunctionBody::new(
                        StatementList::new(
                            [StatementListItem::Statement(
                                Statement::Return(Return::new(Some(
                                    Literal::new(1, Span::new((1, 28), (1, 29))).into(),
                                )))
                                .into(),
                            )],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        Span::new((1, 28), (1, 29)),
                    ),
                    EMPTY_LINEAR_SPAN,
                    Span::new((1, 16), (1, 29)),
                )
                .into(),
            ),
        ))],
        interner,
    );
}

/// Parenthesized async arrows remain default export expressions.
#[test]
fn export_default_parenthesized_async_arrow() {
    let interner = &mut Interner::default();
    let x = interner.get_or_intern_static("x", utf16!("x"));
    check_module_parser(
        "export default (async (x)=>x);",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAssignmentExpression(
                Parenthesized::new(
                    AsyncArrowFunction::new(
                        None,
                        FormalParameterList::from(FormalParameter::new(
                            Variable::from_identifier(
                                Identifier::new(x, Span::new((1, 24), (1, 25))),
                                None,
                            ),
                            false,
                        )),
                        FunctionBody::new(
                            StatementList::new(
                                [StatementListItem::Statement(
                                    Statement::Return(Return::new(Some(
                                        Identifier::new(x, Span::new((1, 28), (1, 29))).into(),
                                    )))
                                    .into(),
                                )],
                                PSEUDO_LINEAR_POS,
                                false,
                            ),
                            Span::new((1, 28), (1, 29)),
                        ),
                        EMPTY_LINEAR_SPAN,
                        Span::new((1, 17), (1, 29)),
                    )
                    .into(),
                    Span::new((1, 16), (1, 30)),
                )
                .into(),
            ),
        ))],
        interner,
    );
}

/// Regression: `export default async function() {}` must remain a hoistable declaration.
#[test]
fn export_default_async_function_declaration() {
    let interner = &mut Interner::default();
    check_module_parser(
        "export default async function() {}",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAsyncFunctionDeclaration(AsyncFunctionDeclaration::new(
                Identifier::new(Sym::DEFAULT, Span::new((1, 30), (1, 31))),
                FormalParameterList::default(),
                FunctionBody::new(StatementList::default(), Span::new((1, 33), (1, 35))),
                EMPTY_LINEAR_SPAN,
            )),
        ))],
        interner,
    );
}

/// Valid `export default` async arrow forms.
#[test]
fn export_default_async_arrow_forms_valid() {
    check_valid_module("export default async () => 1;");
    check_valid_module("export default async x => x;");
    check_valid_module("export default async (x) => x;");
    check_valid_module("export default async (a,b)=>a+b;");
    check_valid_module("export default async (...args)=>args;");
    check_valid_module("export default async ({a})=>a;");
    check_valid_module("export default async ([a])=>a;");
    check_valid_module("export default async ({a}, [b])=>a+b;");
    check_valid_module("export default async (\na,\nb\n)=>a+b;");
    check_valid_module("export default async x=>{\nawait foo();\n};");
    check_valid_module("export default async ()=>await foo();");
    check_valid_module("export default (async (x)=>x);");
    check_valid_module("export default (async ()=>1);");
    check_valid_module("export default async /* comment */ (x)=>x;");
    check_valid_module("export default\nasync (x)=>x;");
    check_valid_module("export default async (a, b) => a + b;");
    check_valid_module("export default async (...args) => args.length;");
    check_valid_module("export default async ({a}) => a;");
    check_valid_module("export default async ([a]) => a;");
    check_valid_module("export default async ({a, b}, [c]) => a + b + c;");
    check_valid_module(
        "export default async (
    a,
    b,
    c
) => a + b + c;",
    );
    check_valid_module(
        "export default async x => {
    await foo(x);
};",
    );
}

/// Malformed `export default` async / arrow forms must still fail.
#[test]
fn export_default_async_arrow_forms_invalid() {
    check_invalid_module("export default async");
    check_invalid_module("export default async (");
    check_invalid_module("export default async =>");
    check_invalid_module("export default async (x)");
    check_invalid_module("export default async (x,");
    check_invalid_module("export default async function");
    check_invalid_module("export default async function (");
    check_invalid_module("export default async function {}");
    check_invalid_module("export default async () >");
    check_invalid_module("export default async () =");
    check_invalid_module("export default async () ->");
}

/// Existing default-export and async forms must keep working.
#[test]
fn export_default_async_arrow_regressions() {
    check_valid_module("export default async function () {}");
    check_valid_module("export default async function foo() {}");
    check_valid_module("export default async function*() {}");
    check_valid_module("export default async function* bar() {}");
    check_valid_module("const fn = async (x) => x;\nexport default fn;");
    check_valid_module("export const fn = async (x) => x;");
    check_valid_module("export default function(){}");
    check_valid_module("export default class {}");
    check_valid_module("export default 123;");
    check_valid_module("export default foo;");
    check_valid_module("export default foo()");
    check_valid_module("export default foo ? a : b;");
    check_valid_module("export default (a + b);");
    check_valid_module("export default new Foo();");
    check_valid_module("export default async;");

    check_valid_module("const fn = async ()=>{};");
    check_valid_module("const fn = async function(){};");
    check_valid_module("export const fn = async ()=>{};");
    check_valid_module("({ async m() {} });");
    check_valid_module("({ async *g() {} });");
    check_valid_module("async function* gen() {}");
    check_valid_module("() => {};");
    check_valid_module("async () => {};");
}

/// Object/array parameter async arrows keep the expected AST shape.
#[test]
fn export_default_async_arrow_destructuring_params() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_module_parser(
        "export default async ({a})=>a;",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAssignmentExpression(
                AsyncArrowFunction::new(
                    None,
                    FormalParameterList::from(FormalParameter::new(
                        Variable::from_pattern(
                            Pattern::from(ObjectPattern::new(
                                vec![ObjectPatternElement::SingleName {
                                    ident: Identifier::new(a, Span::new((1, 24), (1, 25))),
                                    name: Identifier::new(a, Span::new((1, 24), (1, 25))).into(),
                                    default_init: None,
                                }]
                                .into(),
                                Span::new((1, 23), (1, 26)),
                            )),
                            None,
                        ),
                        false,
                    )),
                    FunctionBody::new(
                        StatementList::new(
                            [StatementListItem::Statement(
                                Statement::Return(Return::new(Some(
                                    Identifier::new(a, Span::new((1, 29), (1, 30))).into(),
                                )))
                                .into(),
                            )],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        Span::new((1, 29), (1, 30)),
                    ),
                    EMPTY_LINEAR_SPAN,
                    Span::new((1, 16), (1, 30)),
                )
                .into(),
            ),
        ))],
        interner,
    );

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_module_parser(
        "export default async ([a])=>a;",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAssignmentExpression(
                AsyncArrowFunction::new(
                    None,
                    FormalParameterList::from(FormalParameter::new(
                        Variable::from_pattern(
                            Pattern::from(ArrayPattern::new(
                                vec![ArrayPatternElement::SingleName {
                                    ident: Identifier::new(a, Span::new((1, 24), (1, 25))),
                                    default_init: None,
                                }]
                                .into(),
                                Span::new((1, 23), (1, 26)),
                            )),
                            None,
                        ),
                        false,
                    )),
                    FunctionBody::new(
                        StatementList::new(
                            [StatementListItem::Statement(
                                Statement::Return(Return::new(Some(
                                    Identifier::new(a, Span::new((1, 29), (1, 30))).into(),
                                )))
                                .into(),
                            )],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        Span::new((1, 29), (1, 30)),
                    ),
                    EMPTY_LINEAR_SPAN,
                    Span::new((1, 16), (1, 30)),
                )
                .into(),
            ),
        ))],
        interner,
    );
}

/// Concise await body and binary multi-param async arrows.
#[test]
fn export_default_async_arrow_await_and_multi_param() {
    let interner = &mut Interner::default();
    let foo = interner.get_or_intern_static("foo", utf16!("foo"));
    check_module_parser(
        "export default async ()=>await foo();",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAssignmentExpression(
                AsyncArrowFunction::new(
                    None,
                    FormalParameterList::default(),
                    FunctionBody::new(
                        StatementList::new(
                            [StatementListItem::Statement(
                                Statement::Return(Return::new(Some(
                                    Await::new(
                                        Box::new(
                                            Call::new(
                                                Identifier::new(foo, Span::new((1, 32), (1, 35)))
                                                    .into(),
                                                Box::default(),
                                                Span::new((1, 35), (1, 37)),
                                            )
                                            .into(),
                                        ),
                                        Span::new((1, 26), (1, 37)),
                                    )
                                    .into(),
                                )))
                                .into(),
                            )],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        Span::new((1, 26), (1, 37)),
                    ),
                    EMPTY_LINEAR_SPAN,
                    Span::new((1, 16), (1, 37)),
                )
                .into(),
            ),
        ))],
        interner,
    );

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    let b = interner.get_or_intern_static("b", utf16!("b"));
    check_module_parser(
        "export default async (a,b)=>a+b;",
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::DefaultAssignmentExpression(
                AsyncArrowFunction::new(
                    None,
                    FormalParameterList::from(vec![
                        FormalParameter::new(
                            Variable::from_identifier(
                                Identifier::new(a, Span::new((1, 23), (1, 24))),
                                None,
                            ),
                            false,
                        ),
                        FormalParameter::new(
                            Variable::from_identifier(
                                Identifier::new(b, Span::new((1, 25), (1, 26))),
                                None,
                            ),
                            false,
                        ),
                    ]),
                    FunctionBody::new(
                        StatementList::new(
                            [StatementListItem::Statement(
                                Statement::Return(Return::new(Some(
                                    Binary::new(
                                        ArithmeticOp::Add.into(),
                                        Identifier::new(a, Span::new((1, 29), (1, 30))).into(),
                                        Identifier::new(b, Span::new((1, 31), (1, 32))).into(),
                                    )
                                    .into(),
                                )))
                                .into(),
                            )],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        Span::new((1, 29), (1, 32)),
                    ),
                    EMPTY_LINEAR_SPAN,
                    Span::new((1, 16), (1, 32)),
                )
                .into(),
            ),
        ))],
        interner,
    );
}
