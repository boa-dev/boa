use crate::syntax::{
    ast::node::{field::GetConstField, Call, Identifier},
    parser::tests::check_parser,
};
use boa_interner::Interner;

#[track_caller]
fn check_call_property_identifier(property_name: &'static str) {
    let mut interner = Interner::default();
    check_parser(
        format!("a().{}", property_name).as_str(),
        vec![GetConstField::new(
            Call::new(Identifier::new(interner.get_or_intern_static("a")), vec![]),
            interner.get_or_intern_static(property_name),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_call_properties() {
    check_call_property_identifier("prop");
    check_call_property_identifier("true");
    check_call_property_identifier("false");
    check_call_property_identifier("null");
    check_call_property_identifier("let");
}

#[track_caller]
fn check_member_property_identifier(property_name: &'static str) {
    let mut interner = Interner::default();
    check_parser(
        format!("a.{}", property_name).as_str(),
        vec![GetConstField::new(
            Identifier::new(interner.get_or_intern_static("a")),
            interner.get_or_intern_static(property_name),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_member_properties() {
    check_member_property_identifier("prop");
    check_member_property_identifier("true");
    check_member_property_identifier("false");
    check_member_property_identifier("null");
    check_member_property_identifier("let");
}
