use crate::parser::tests::check_parser;
use boa_ast::{
    expression::{access::SimplePropertyAccess, Call, Identifier},
    Expression, Statement,
};
use boa_interner::Interner;
use boa_macros::utf16;

macro_rules! check_call_property_identifier {
    ($property:literal) => {{
        let interner = &mut Interner::default();
        check_parser(
            format!("a().{}", $property).as_str(),
            vec![Statement::Expression(Expression::PropertyAccess(
                SimplePropertyAccess::new(
                    Call::new(
                        Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                        Box::default(),
                    )
                    .into(),
                    interner.get_or_intern_static($property, utf16!($property)),
                )
                .into(),
            ))
            .into()],
            interner,
        );
    }};
}

#[test]
fn check_call_properties() {
    check_call_property_identifier!("prop");
    check_call_property_identifier!("true");
    check_call_property_identifier!("false");
    check_call_property_identifier!("null");
    check_call_property_identifier!("let");
}

macro_rules! check_member_property_identifier {
    ($property:literal) => {{
        let interner = &mut Interner::default();
        check_parser(
            format!("a.{}", $property).as_str(),
            vec![Statement::Expression(Expression::PropertyAccess(
                SimplePropertyAccess::new(
                    Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                    interner.get_or_intern_static($property, utf16!($property)),
                )
                .into(),
            ))
            .into()],
            interner,
        );
    }};
}

#[test]
fn check_member_properties() {
    check_member_property_identifier!("prop");
    check_member_property_identifier!("true");
    check_member_property_identifier!("false");
    check_member_property_identifier!("null");
    check_member_property_identifier!("let");
}
