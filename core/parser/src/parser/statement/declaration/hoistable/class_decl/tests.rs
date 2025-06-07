use crate::parser::tests::check_script_parser;
use boa_ast::{
    declaration::{LexicalDeclaration, Variable, VariableList},
    expression::{
        access::{PropertyAccess, SimplePropertyAccess},
        literal::Literal,
        Call, Identifier,
    },
    function::{
        ClassDeclaration, ClassElement, ClassFieldDefinition, ClassMethodDefinition,
        FormalParameterList, FunctionBody, FunctionExpression,
    },
    property::{MethodDefinitionKind, PropertyName},
    Declaration, Expression, Span, Statement, StatementList, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;
use indoc::indoc;

#[test]
fn check_async_ordinary_method() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::MethodDefinition(ClassMethodDefinition::new(
        boa_ast::function::ClassElementName::PropertyName(PropertyName::Literal(
            interner.get_or_intern_static("async", utf16!("async")),
        )),
        FormalParameterList::default(),
        FunctionBody::new(StatementList::default(), Span::new((2, 13), (2, 16))),
        MethodDefinitionKind::Ordinary,
        false,
        boa_ast::LinearPosition::default(),
    ))];

    check_script_parser(
        indoc! {r#"
            class A {
                async() { }
            }
        "#},
        [Declaration::ClassDeclaration(ClassDeclaration::new(
            interner.get_or_intern_static("A", utf16!("A")).into(),
            None,
            None,
            elements.into(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_field_initialization() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::FieldDefinition(ClassFieldDefinition::new(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        Some(Literal::new(1, Span::new((3, 7), (3, 8))).into()),
    ))];

    check_script_parser(
        indoc! {"
            class A {
                async
                = 1
            }
        "},
        [Declaration::ClassDeclaration(ClassDeclaration::new(
            interner.get_or_intern_static("A", utf16!("A")).into(),
            None,
            None,
            elements.into(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_async_field() {
    let interner = &mut Interner::default();

    let elements = vec![ClassElement::FieldDefinition(ClassFieldDefinition::new(
        PropertyName::Literal(interner.get_or_intern_static("async", utf16!("async"))),
        None,
    ))];

    check_script_parser(
        "class A {
            async
         }
        ",
        [Declaration::ClassDeclaration(ClassDeclaration::new(
            interner.get_or_intern_static("A", utf16!("A")).into(),
            None,
            None,
            elements.into(),
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

    let constructor = FunctionExpression::new(
        Some(interner.get_or_intern_static("A", utf16!("A")).into()),
        FormalParameterList::default(),
        FunctionBody::new(
            StatementList::new(
                [Statement::Expression(console).into()],
                boa_ast::LinearPosition::new(0),
                false,
            ),
            Span::new((2, 19), (4, 6)),
        ),
        None,
        false,
        Span::new((2, 5), (4, 6)),
    );

    let class = ClassDeclaration::new(
        interner.get("A").unwrap().into(),
        None,
        Some(constructor),
        Box::default(),
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
        indoc! {r#"
            class A {
                constructor() {
                    console.log(new.target.name);
                }
            }
            const a = new A();
        "#},
        script,
        interner,
    );
}
