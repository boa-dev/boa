//! Tests for the scope analysis of the AST.

#![allow(unused_crate_dependencies)]

use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::Identifier,
    function::{FormalParameter, FormalParameterList, FunctionBody, FunctionDeclaration},
    scope::Scope,
    statement::Block,
    Declaration, Expression, Script, Statement, StatementList, StatementListItem,
};
use boa_interner::Interner;
use boa_string::JsString;

#[test]
fn empty_script_is_ok() {
    let scope = &Scope::new_global();
    let mut script = Script::default();
    let ok = script.analyze_scope(scope, &Interner::new());
    assert!(ok);
    assert_eq!(scope.num_bindings(), 0);
}

#[test]
fn script_global_let() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let a = interner.get_or_intern("a");
    let mut script = Script::new(StatementList::new(
        [Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(a.into(), None)]
                .try_into()
                .unwrap(),
        ))
        .into()],
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner);
    assert!(ok);
    assert_eq!(scope.num_bindings(), 1);
    let a = scope.get_binding_reference(&JsString::from("a")).unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(!a.local());
}

#[test]
fn script_global_const() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let a = interner.get_or_intern("a");
    let mut script = Script::new(StatementList::new(
        [Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(a.into(), None)]
                .try_into()
                .unwrap(),
        ))
        .into()],
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner);
    assert!(ok);
    assert_eq!(scope.num_bindings(), 1);
    let a = scope.get_binding_reference(&JsString::from("a")).unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(!a.local());
}

#[test]
fn script_block_let() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let a = interner.get_or_intern("a");
    let mut script = Script::new(StatementList::new(
        [Statement::Block(Block::from(vec![Declaration::Lexical(
            LexicalDeclaration::Let(
                vec![Variable::from_identifier(a.into(), None)]
                    .try_into()
                    .unwrap(),
            ),
        )
        .into()]))
        .into()],
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner);
    assert!(ok);
    assert_eq!(scope.num_bindings(), 0);
    let StatementListItem::Statement(Statement::Block(block)) =
        script.statements().first().unwrap()
    else {
        panic!("Expected a block statement");
    };
    let scope = block.scope().unwrap();
    assert_eq!(scope.num_bindings(), 1);
    let a = scope.get_binding_reference(&JsString::from("a")).unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(a.local());
}

#[test]
fn script_function_mapped_arguments_not_accessed() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let f = interner.get_or_intern("f");
    let a = interner.get_or_intern("a");
    let mut script = Script::new(StatementList::new(
        [Declaration::FunctionDeclaration(FunctionDeclaration::new(
            f.into(),
            FormalParameterList::from_parameters(vec![FormalParameter::new(
                Variable::from_identifier(a.into(), None),
                false,
            )]),
            FunctionBody::new(
                [Declaration::Lexical(LexicalDeclaration::Let(
                    vec![Variable::from_identifier(a.into(), None)]
                        .try_into()
                        .unwrap(),
                ))
                .into()],
                false,
            ),
        ))
        .into()],
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner);
    assert!(ok);
    assert_eq!(scope.num_bindings(), 0);
    let StatementListItem::Declaration(Declaration::FunctionDeclaration(f)) =
        script.statements().first().unwrap()
    else {
        panic!("Expected a block statement");
    };
    assert_eq!(f.scopes().function_scope().num_bindings(), 2);
    assert_eq!(f.scopes().parameters_eval_scope(), None);
    assert_eq!(f.scopes().parameters_scope(), None);
    assert_eq!(f.scopes().lexical_scope().unwrap().num_bindings(), 1);
    let arguments = f
        .scopes()
        .function_scope()
        .get_binding_reference(&JsString::from("arguments"))
        .unwrap();
    assert!(!f.scopes().arguments_object_accessed());
    assert!(!arguments.is_global_object());
    assert!(arguments.is_lexical());
    assert!(arguments.local());
    let a = f
        .scopes()
        .function_scope()
        .get_binding_reference(&JsString::from("a"))
        .unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(a.local());
    let a = f
        .scopes()
        .lexical_scope()
        .unwrap()
        .get_binding_reference(&JsString::from("a"))
        .unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(a.local());
}

#[test]
fn script_function_mapped_arguments_accessed() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let f = interner.get_or_intern("f");
    let a = interner.get_or_intern("a");
    let arguments = interner.get_or_intern("arguments");
    let mut script = Script::new(StatementList::new(
        [Declaration::FunctionDeclaration(FunctionDeclaration::new(
            f.into(),
            FormalParameterList::from_parameters(vec![FormalParameter::new(
                Variable::from_identifier(a.into(), None),
                false,
            )]),
            FunctionBody::new(
                [
                    Declaration::Lexical(LexicalDeclaration::Let(
                        vec![Variable::from_identifier(a.into(), None)]
                            .try_into()
                            .unwrap(),
                    ))
                    .into(),
                    Statement::Expression(Expression::Identifier(Identifier::new(arguments)))
                        .into(),
                ],
                false,
            ),
        ))
        .into()],
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner);
    assert!(ok);
    assert_eq!(scope.num_bindings(), 0);
    let StatementListItem::Declaration(Declaration::FunctionDeclaration(f)) =
        script.statements().first().unwrap()
    else {
        panic!("Expected a block statement");
    };
    assert!(f.scopes().arguments_object_accessed());
    assert_eq!(f.scopes().function_scope().num_bindings(), 2);
    assert_eq!(f.scopes().parameters_eval_scope(), None);
    assert_eq!(f.scopes().parameters_scope(), None);
    assert_eq!(f.scopes().lexical_scope().unwrap().num_bindings(), 1);
    let arguments = f
        .scopes()
        .function_scope()
        .get_binding_reference(&JsString::from("arguments"))
        .unwrap();
    assert!(!arguments.is_global_object());
    assert!(arguments.is_lexical());
    assert!(arguments.local());
    let a = f
        .scopes()
        .function_scope()
        .get_binding_reference(&JsString::from("a"))
        .unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(!a.local());
    let a = f
        .scopes()
        .lexical_scope()
        .unwrap()
        .get_binding_reference(&JsString::from("a"))
        .unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(a.local());
}
