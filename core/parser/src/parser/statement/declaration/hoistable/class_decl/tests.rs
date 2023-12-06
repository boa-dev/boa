use crate::parser::tests::check_script_parser;
use boa_ast::{
    declaration::{LexicalDeclaration, Variable, VariableList},
    expression::{
        access::{PropertyAccess, SimplePropertyAccess},
        literal::Literal,
        Call, Identifier,
    },
    function::{Class, ClassElement, FormalParameterList, Function, FunctionBody},
    property::{MethodDefinition, PropertyName},
    Declaration, Expression, Statement, StatementList, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_async_ordinary_method() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::MethodDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        MethodDefinition::Ordinary(Function::new(
            None,
            FormalParameterList::default(),
            FunctionBody::default(),
        )),
    )];

    check_script_parser(
        "class A {
            async() { }
         }
        ",
        [Declaration::Class(Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A")).into()),
            None,
            None,
            elements.into(),
            true,
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_field_initialization() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::FieldDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        Some(Literal::from(1).into()),
    )];

    check_script_parser(
        "class A {
            async
              = 1
         }
        ",
        [Declaration::Class(Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A")).into()),
            None,
            None,
            elements.into(),
            true,
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_field() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::FieldDefinition(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        None,
    )];

    check_script_parser(
        "class A {
            async
         }
        ",
        [Declaration::Class(Class::new(
            Some(interner.get_or_intern_static("A", utf16!("A")).into()),
            None,
            None,
            elements.into(),
            true,
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_new_target_with_property_access() {
    let interner = &mut Interner::default();

    let new_target = Expression::PropertyAccess(
        SimplePropertyAccess::new(
            Expression::NewTarget,
            interner.get_or_intern_static("name", utf16!("name")),
        )
        .into(),
    );

    let console = Expression::Call(Call::new(
        PropertyAccess::Simple(SimplePropertyAccess::new(
            Identifier::from(interner.get_or_intern_static("console", utf16!("console"))).into(),
            interner.get_or_intern_static("log", utf16!("log")),
        ))
        .into(),
        [new_target].into(),
    ));

    let constructor = Function::new(
        Some(interner.get_or_intern_static("A", utf16!("A")).into()),
        FormalParameterList::default(),
        FunctionBody::new(StatementList::new(
            [Statement::Expression(console).into()],
            false,
        )),
    );

    let class = Class::new(
        Some(interner.get("A").unwrap().into()),
        None,
        Some(constructor),
        Box::default(),
        true,
    );

    let instantiation = Expression::New(
        Call::new(
            Identifier::from(interner.get("A").unwrap()).into(),
            Box::default(),
        )
        .into(),
    );

    let const_decl = LexicalDeclaration::Const(
        VariableList::new(
            [Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(instantiation),
            )]
            .into(),
        )
        .unwrap(),
    );

    let script = [
        StatementListItem::Declaration(class.into()),
        StatementListItem::Declaration(const_decl.into()),
    ];

    check_script_parser(
        r#"
            class A {
                constructor() {
                    console.log(new.target.name);
                }
            }
            const a = new A();
        "#,
        script,
        interner,
    );
}
