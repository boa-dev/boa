use crate::syntax::{
    ast::{
        node::{Block, Catch, Finally, Identifier, Try, VarDecl, VarDeclList},
        Const,
    },
    parser::tests::{check_invalid, check_parser},
};

#[test]
fn check_inline_with_empty_try_catch() {
    check_parser(
        "try { } catch(e) {}",
        vec![Try::new(vec![], Some(Catch::new("e", vec![])), None).into()],
    );
}

#[test]
fn check_inline_with_var_decl_inside_try() {
    check_parser(
        "try { var x = 1; } catch(e) {}",
        vec![Try::new(
            vec![VarDeclList::from(vec![VarDecl::new("x", Some(Const::from(1).into()))]).into()],
            Some(Catch::new("e", vec![])),
            None,
        )
        .into()],
    );
}

#[test]
fn check_inline_with_var_decl_inside_catch() {
    check_parser(
        "try { var x = 1; } catch(e) { var x = 1; }",
        vec![Try::new(
            vec![VarDeclList::from(vec![VarDecl::new("x", Some(Const::from(1).into()))]).into()],
            Some(Catch::new(
                "e",
                vec![
                    VarDeclList::from(vec![VarDecl::new("x", Some(Const::from(1).into()))]).into(),
                ],
            )),
            None,
        )
        .into()],
    );
}

#[test]
fn check_inline_with_empty_try_catch_finally() {
    check_parser(
        "try {} catch(e) {} finally {}",
        vec![Try::new(
            vec![],
            Some(Catch::new("e", vec![])),
            Some(Finally::from(vec![])),
        )
        .into()],
    );
}

#[test]
fn check_inline_with_empty_try_finally() {
    check_parser(
        "try {} finally {}",
        vec![Try::new(vec![], None, Some(Finally::from(vec![]))).into()],
    );
}

#[test]
fn check_inline_with_empty_try_var_decl_in_finally() {
    check_parser(
        "try {} finally { var x = 1; }",
        vec![Try::new(
            vec![],
            None,
            Some(Finally::from(vec![VarDeclList::from(vec![VarDecl::new(
                "x",
                Some(Const::from(1).into()),
            )])
            .into()])),
        )
        .into()],
    );
}

#[test]
fn check_inline_empty_try_paramless_catch() {
    check_parser(
        "try {} catch { var x = 1; }",
        vec![Try::new(
            Block::from(vec![]),
            Some(Catch::new::<_, Identifier, _>(
                None,
                vec![
                    VarDeclList::from(vec![VarDecl::new("x", Some(Const::from(1).into()))]).into(),
                ],
            )),
            None,
        )
        .into()],
    );
}

#[test]
fn check_inline_invalid_catch() {
    check_invalid("try {} catch");
}

#[test]
fn check_inline_invalid_catch_without_closing_paren() {
    check_invalid("try {} catch(e {}");
}

#[test]
fn check_inline_invalid_catch_parameter() {
    check_invalid("try {} catch(1) {}");
}

#[test]
fn check_invalide_try_no_catch_finally() {
    check_invalid("try {} let a = 10;");
}
