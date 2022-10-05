use crate::{
    string::utf16,
    syntax::{
        ast::node::{field::GetConstField, Call, Identifier},
        parser::tests::check_parser,
    },
};
use boa_interner::Interner;

macro_rules! check_call_property_identifier {
    ($property:literal) => {{
        let mut interner = Interner::default();
        check_parser(
            format!("a().{}", $property).as_str(),
            vec![GetConstField::new(
                Call::new(
                    Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
                    vec![],
                ),
                interner.get_or_intern_static($property, utf16!($property)),
            )
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
        let mut interner = Interner::default();
        check_parser(
            format!("a.{}", $property).as_str(),
            vec![GetConstField::new(
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
                interner.get_or_intern_static($property, utf16!($property)),
            )
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
