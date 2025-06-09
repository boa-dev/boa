use proc_macro2::{Ident, Span as Span2};
use quote::ToTokens;
use std::fmt::Display;
use std::str::FromStr;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, ExprLit, Lit, MetaNameValue};

pub(crate) type SpannedResult<T> = Result<T, (Span2, String)>;

/// A function to make it easier to return error messages.
pub(crate) fn error<T>(span: &impl Spanned, message: impl Display) -> SpannedResult<T> {
    Err((span.span(), message.to_string()))
}

/// Look (and remove from AST) a `path` version of the attribute `boa`, e.g. `#[boa(something)]`.
pub(crate) fn take_path_attr(attrs: &mut Vec<Attribute>, name: &str) -> bool {
    if let Some((i, _)) = attrs
        .iter()
        .enumerate()
        .filter(|(_, a)| a.path().is_ident("boa"))
        .filter_map(|(i, a)| a.meta.require_list().ok().map(|nv| (i, nv)))
        .filter_map(|(i, m)| m.parse_args_with(Ident::parse_any).ok().map(|p| (i, p)))
        .find(|(_, path)| path == name)
    {
        attrs.remove(i);
        true
    } else {
        false
    }
}

/// Look (and remove from AST) for a `#[boa(name = ...)]` attribute, where `...`
/// is a literal. The validation of the literal's type should be done separately.
pub(crate) fn take_name_value_attr(attrs: &mut Vec<Attribute>, name: &str) -> Option<Lit> {
    if let Some((i, lit)) = attrs
        .iter()
        .enumerate()
        .filter(|(_, a)| a.meta.path().is_ident("boa"))
        .filter_map(|(i, a)| a.meta.require_list().ok().map(|nv| (i, nv)))
        .filter_map(|(i, a)| {
            syn::parse2::<MetaNameValue>(a.tokens.to_token_stream())
                .ok()
                .map(|nv| (i, nv))
        })
        .filter(|(_, nv)| nv.path.is_ident(name))
        .find_map(|(i, nv)| match &nv.value {
            Expr::Lit(ExprLit { lit, .. }) => Some((i, lit.clone())),
            _ => None,
        })
    {
        attrs.remove(i);
        Some(lit)
    } else {
        None
    }
}

/// Take the length name-value from the list of attributes.
pub(crate) fn take_length_from_attrs(attrs: &mut Vec<Attribute>) -> SpannedResult<Option<usize>> {
    match take_name_value_attr(attrs, "length") {
        None => Ok(None),
        Some(lit) => match lit {
            Lit::Int(int) if int.base10_parse::<usize>().is_ok() => int
                .base10_parse::<usize>()
                .map(Some)
                .map_err(|e| (int.span(), format!("Invalid literal: {e}"))),
            l => error(&l, "Invalid literal type. Was expecting a number")?,
        },
    }
}

pub(crate) fn take_name_value_string(
    attrs: &mut Vec<Attribute>,
    name: &str,
) -> SpannedResult<Option<String>> {
    match take_name_value_attr(attrs, name) {
        None => Ok(None),
        Some(lit) => match lit {
            Lit::Str(s) => Ok(Some(s.value())),
            l => Err((
                l.span(),
                "Invalid literal type. Was expecting a string".to_string(),
            )),
        },
    }
}

/// Take the last `#[boa(error = "...")]` statement if found, remove it from the list
/// of attributes, and return the literal string.
pub(crate) fn take_error_from_attrs(attrs: &mut Vec<Attribute>) -> SpannedResult<Option<String>> {
    take_name_value_string(attrs, "error")
}

#[derive(Copy, Clone, Debug, Default)]
pub(crate) enum RenameScheme {
    #[default]
    None,
    CamelCase,
    PascalCase,
}

impl FromStr for RenameScheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("none") {
            Ok(Self::None)
        } else if s.eq_ignore_ascii_case("camelcase") {
            Ok(Self::CamelCase)
        } else if s.eq_ignore_ascii_case("pascalcase") {
            Ok(Self::PascalCase)
        } else {
            Err(format!(
                r#"Invalid rename scheme: {s:?}. Accepted values are "none" or "camelCase"."#
            ))
        }
    }
}

impl RenameScheme {
    pub(crate) fn from_attrs(attrs: &mut Vec<Attribute>) -> SpannedResult<Self> {
        Self::from_named_attrs(attrs, "rename").map(|rs| rs.unwrap_or(RenameScheme::None))
    }

    pub(crate) fn from_named_attrs(
        attrs: &mut Vec<Attribute>,
        name: &str,
    ) -> SpannedResult<Option<Self>> {
        match take_name_value_attr(attrs, name) {
            None => Ok(None),
            Some(Lit::Str(lit_str)) => Self::from_str(lit_str.value().as_str())
                .map_err(|e| (lit_str.span(), e))
                .map(Some),
            Some(lit) => Err((
                lit.span(),
                "Invalid attribute value literal, expected a string.".to_string(),
            )),
        }
    }

    fn camel_case(s: &str) -> String {
        #[derive(PartialEq)]
        enum State {
            First,
            NextOfUpper,
            NextOfContinuedUpper(char),
            NextOfSepMark,
            Other,
        }

        let mut result = String::with_capacity(s.len());
        let mut state = State::First;

        for ch in s.chars() {
            let is_upper = ch.is_ascii_uppercase();
            let is_lower = ch.is_ascii_lowercase();

            match (&state, is_upper, is_lower) {
                (State::First, true, false) => {
                    state = State::NextOfUpper;
                    result.push(ch.to_ascii_lowercase());
                }
                (State::First, false, true) => {
                    state = State::Other;
                    result.push(ch);
                }
                (State::First, false, false) => {}
                (State::NextOfUpper, true, false) => {
                    state = State::NextOfContinuedUpper(ch);
                }
                (State::NextOfUpper, false, true) => {
                    state = State::First;
                    result.push(ch);
                }
                (State::NextOfContinuedUpper(last), true, false) => {
                    result.push(last.to_ascii_lowercase());
                    state = State::NextOfContinuedUpper(ch);
                }
                (State::NextOfContinuedUpper(last), false, true) => {
                    result.push(last.to_ascii_uppercase());
                    state = State::First;
                    result.push(ch);
                }
                (State::NextOfContinuedUpper(last), false, false) => {
                    result.push(last.to_ascii_lowercase());
                    state = State::NextOfSepMark;
                }
                (State::NextOfSepMark, true, false) => {
                    state = State::NextOfUpper;
                    result.push(ch);
                }
                (State::NextOfSepMark, false, true) | (State::Other, true, false) => {
                    state = State::NextOfUpper;
                    result.push(ch.to_ascii_uppercase());
                }
                (State::Other, false, true) => {
                    result.push(ch);
                }
                (_, false, false) => {
                    state = State::NextOfSepMark;
                }
                (_, _, _) => {}
            }
        }

        if let State::NextOfContinuedUpper(last) = state {
            result.push(last.to_ascii_lowercase());
        }

        result
    }

    fn pascal_case(s: &str) -> String {
        let mut result = Self::camel_case(s);
        if let Some(ch) = result.get_mut(..1) {
            ch.make_ascii_uppercase();
        }
        result
    }

    pub(crate) fn rename(self, s: String) -> String {
        match self {
            Self::None => s,
            Self::CamelCase => Self::camel_case(&s),
            Self::PascalCase => Self::pascal_case(&s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RenameScheme;
    use test_case::test_case;

    #[rustfmt::skip]
    #[test_case("HelloWorld", "helloWorld" ; "camel_case_1")]
    #[test_case("Hello_World", "helloWorld" ; "camel_case_2")]
    #[test_case("hello_world", "helloWorld" ; "camel_case_3")]
    #[test_case("__hello_world__", "helloWorld" ; "camel_case_4")]
    #[test_case("HELLOWorld", "helloWorld" ; "camel_case_5")]
    #[test_case("helloWORLD", "helloWorld" ; "camel_case_6")]
    #[test_case("HELLO_WORLD", "helloWorld" ; "camel_case_7")]
    fn camel_case(input: &str, expected: &str) {
        assert_eq!(RenameScheme::camel_case(input).as_str(), expected);
    }

    #[rustfmt::skip]
    #[test_case("HelloWorld", "HelloWorld" ; "pascal_case_1")]
    #[test_case("Hello_World", "HelloWorld" ; "pascal_case_2")]
    #[test_case("hello_world", "HelloWorld" ; "pascal_case_3")]
    #[test_case("__hello_world__", "HelloWorld" ; "pascal_case_4")]
    #[test_case("HELLOWorld", "HelloWorld" ; "pascal_case_5")]
    #[test_case("helloWORLD", "HelloWorld" ; "pascal_case_6")]
    #[test_case("HELLO_WORLD", "HelloWorld" ; "pascal_case_7")]
    fn pascal_case(input: &str, expected: &str) {
        assert_eq!(RenameScheme::pascal_case(input).as_str(), expected);
    }
}
