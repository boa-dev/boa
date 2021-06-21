use crate::syntax::{
    ast::node::{ClassDecl, Declaration, DeclarationList, FunctionDecl},
    parser::{class::ClassField, tests::check_parser, Parser},
};

/// Checks for empty class parsing (making sure the keyword works)
#[test]
fn check_empty() {
    check_parser(
        "class Empty {}",
        vec![ClassDecl::new(Box::from("Empty"), None, vec![], vec![]).into()],
    );
}

/// Checks for a constructor being parsed in a class
#[test]
fn check_basic() {
    check_parser(
        r#"
        class Basic {
            constructor() {
                let val;
            }
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Basic"),
            Some(FunctionDecl::new(
                Box::from("constructor"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            )),
            vec![],
            vec![],
        )
        .into()],
    );
}

/// Checks for a static method being parsed in a class
#[test]
fn check_static() {
    check_parser(
        r#"
        class Basic {
            static say_hello() {
                let val;
            }
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Basic"),
            None,
            vec![],
            vec![ClassField::Method(FunctionDecl::new(
                Box::from("say_hello"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            ))],
        )
        .into()],
    );
}

/// Checks for multiple functions being parsed.
#[test]
fn check_multi() {
    check_parser(
        r#"
        class Multi {
            constructor() {
                let val;
            }
            say_hello() {}
            say_hello_again() {}
        }
        "#,
        vec![ClassDecl::new(
            Box::from("Multi"),
            Some(FunctionDecl::new(
                Box::from("constructor"),
                vec![],
                vec![
                    DeclarationList::Let(vec![Declaration::new("val", None)].into_boxed_slice())
                        .into(),
                ],
            )),
            vec![
                ClassField::Method(FunctionDecl::new(Box::from("say_hello"), vec![], vec![])),
                ClassField::Method(FunctionDecl::new(
                    Box::from("say_hello_again"),
                    vec![],
                    vec![],
                )),
            ],
            vec![],
        )
        .into()],
    );
}

/// Checks for getters/setters being parsed.
#[test]
fn check_getter_setter() {
    check_parser(
        r#"
        class GetterSetter {
            get a() {}
            set a() {}
        }
        "#,
        vec![ClassDecl::new(
            Box::from("GetterSetter"),
            None,
            vec![
                ClassField::Getter(FunctionDecl::new(Box::from("a"), vec![], vec![])),
                ClassField::Setter(FunctionDecl::new(Box::from("a"), vec![], vec![])),
            ],
            vec![],
        )
        .into()],
    );
}

/// Checks for duplicate function names
#[test]
fn check_name_errors() {
    let js = r#"
        class SameFunction {
            hello() {}
            hello() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    // This is the only situation where the same name is valid
    let js = r#"
        class GetterSetterSameName {
            get hello() { return 5; }
            set hello(v) {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_ok());

    let js = r#"
        class FunctionSameNameAsGetter {
            hello() {}
            get hello() { return 5; }
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class FunctionSameNameAsGetter {
            hello() {}
            set hello(a) {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    // Static and non static names share the same rules as above
    let js = r#"
        class StaticNonStaticSameName {
            hello() {}
            static hello() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    // Prototype is a reserved word for static methods
    let js = r#"
        class StaticPrototype {
            static prototype() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticPrototype {
            static prototype = 5;
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticPrototype {
            static get prototype() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticPrototype {
            static set prototype() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticPrototype {
            set a(val) {}
            static get a() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticPrototype {
            static set a(val) {}
            get a() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticPrototype {
            get a() {}
            static set a(val) {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticPrototype {
            static get a() {}
            set a(val) {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticPrototype {
            get a() {}
            set a(val) {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_ok());

    let js = r#"
        class StaticPrototype {
            static get a() {}
            static set a(val) {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_ok());
}

/// Checks for all constructor errors (there are a lot).
#[test]
fn check_constructor_errors() {
    let js = r#"
        class MultiConstructor {
            constructor() {}
            constructor() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class GetterConstructor {
            get constructor() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class SetterConstructor {
            set constructor() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class StaticConstructor {
            static constructor() {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class ConstructorField {
            constructor = 5;
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());

    let js = r#"
        class ConstructorField {
            static constructor = 5;
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());
}

/// Checks for all getter/setter argument errors
#[test]
fn check_getter() {
    let js = r#"
        class GettersTakeNoArgs {
            get a(v) {}
        }
        "#;
    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());
}
