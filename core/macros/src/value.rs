use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket};
use syn::{braced, bracketed, parse2, Expr, Ident, LitStr, Token};

/// The key can be an identifier (which will be stringified), or an actual string
/// literal.
enum Key {
    Bracketed(Expr),
    Ident(Ident),
    StringLiteral(LitStr),
}

impl Parse for Key {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Bracket) {
            let content;
            let _bracket = bracketed!(content in input);
            Ok(Key::Bracketed(content.parse()?))
        } else if input.peek(Ident) {
            Ok(Key::Ident(input.parse()?))
        } else if input.peek(LitStr) {
            Ok(Key::StringLiteral(input.parse()?))
        } else {
            Err(input.error("Expected a field name"))
        }
    }
}

/// A value, which itself can recursively be an object, an array, a literal or
/// an expression.
enum Value {
    Object(Object),
    Array(Array),
    String(LitStr),
    Expr(Expr),
}

impl Value {
    fn output(&self, context: Option<&Ident>) -> syn::Result<TokenStream> {
        match self {
            Value::Object(o) => o.output(context).map(|o| {
                quote! {
                    ::boa_engine::JsValue::from( #o )
                }
            }),
            Value::Array(a) => a.output(context),
            Value::String(str) => Ok(quote! {
                ::boa_engine::JsValue::from( ::boa_macros::js_str!( #str ) )
            }),
            Value::Expr(e) => Ok(quote! {
                ::boa_engine::JsValue::from( #e )
            }),
        }
    }
}

impl Parse for Value {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Brace) {
            Ok(Self::Object(input.parse()?))
        } else if input.peek(Bracket) {
            Ok(Self::Array(input.parse()?))
        } else if input.peek(LitStr) {
            Ok(Self::String(input.parse()?))
        } else {
            Ok(Self::Expr(input.parse()?))
        }
    }
}

/// An object is built of multiple key-value pairs.
struct KeyValue {
    key: Key,
    _colon: Token![:],
    value: Value,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            _colon: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// An object declaration.
struct Object {
    _brace: Brace,
    fields: Punctuated<KeyValue, Token![,]>,
}

impl Object {
    fn output(&self, context: Option<&Ident>) -> syn::Result<TokenStream> {
        let Some(c_ident) = context else {
            return Err(syn::Error::new(
                Span::call_site(),
                "Need to specify a context identifier.",
            ));
        };

        let fields: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| match &field.key {
                Key::Bracketed(expr) => (
                    quote! {
                        ::boa_engine::property::PropertyKey::from( #expr )
                    },
                    &field.value,
                ),
                Key::Ident(ident) => {
                    let ident = ident.to_string();
                    (quote! { ::boa_engine::js_string!( #ident ) }, &field.value)
                }
                Key::StringLiteral(literal) => (
                    quote! { ::boa_engine::js_string!( #literal ) },
                    &field.value,
                ),
            })
            .map(|(key, value)| {
                let value = value.output(context)?;

                Ok(quote! {
                    o.set( #key, #value, false, #c_ident )
                     .expect("Cannot set property");
                })
            })
            .collect::<syn::Result<_>>()?;

        Ok(quote! {
            {
                let o = ::boa_engine::JsObject::with_null_proto();
                #(#fields)*
                o
            }
        })
    }
}

impl Parse for Object {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        let brace = braced!(content in input);

        Ok(Self {
            _brace: brace,
            fields: Punctuated::parse_terminated(&content)?,
        })
    }
}

/// An array declaration.
struct Array {
    _bracket: Bracket,
    elems: Punctuated<Value, Token![,]>,
}

impl Array {
    fn output(&self, context: Option<&Ident>) -> syn::Result<TokenStream> {
        let items: Vec<TokenStream> = self
            .elems
            .iter()
            .map(|item| item.output(context))
            .collect::<syn::Result<_>>()?;

        let Some(c_ident) = context else {
            return Err(syn::Error::new(
                Span::call_site(),
                "Need to specify a context identifier.",
            ));
        };

        Ok(quote! {
            ::boa_engine::JsValue::from(
                ::boa_engine::object::builtins::JsArray::from_iter( [ #(#items),* ], #c_ident )
            )
        })
    }
}

impl Parse for Array {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        let bracket = bracketed!(content in input);
        Ok(Self {
            _bracket: bracket,
            elems: Punctuated::parse_terminated(&content)?,
        })
    }
}

/// The result of parsing the full `js_value!()` macro arguments.
struct JsValue {
    value: Value,
    context_ident: Option<Ident>,
}

impl JsValue {
    fn output(&self) -> TokenStream {
        self.value
            .output(self.context_ident.as_ref())
            .unwrap_or_else(|err| err.to_compile_error())
    }
}

impl Parse for JsValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let value = input.parse()?;
        let mut context_ident = None;

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
            context_ident = Some(input.parse()?);
        }

        Ok(Self {
            value,
            context_ident,
        })
    }
}

/// The result of parsing the `js_object!()` arguments.
struct JsObject {
    value: Object,
    context_ident: Option<Ident>,
}

impl JsObject {
    fn output(&self) -> TokenStream {
        self.value
            .output(self.context_ident.as_ref())
            .unwrap_or_else(|err| err.to_compile_error())
    }
}

impl Parse for JsObject {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let value = input.parse()?;
        let mut context_ident = None;

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
            context_ident = Some(input.parse()?);
        }

        Ok(Self {
            value,
            context_ident,
        })
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn js_object_impl(input: TokenStream) -> TokenStream {
    parse2::<JsObject>(input).map_or_else(|e| e.to_compile_error(), |v| v.output())
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn js_value_impl(input: TokenStream) -> TokenStream {
    parse2::<JsValue>(input).map_or_else(|e| e.to_compile_error(), |v| v.output())
}
