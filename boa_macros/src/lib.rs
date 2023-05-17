//! Macros for the Boa JavaScript engine.
//!
#![doc = include_str!("../../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,

    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Data, DeriveInput, Expr, ExprLit, Fields, FieldsNamed, Ident, Lit, LitStr, Token,
};
use synstructure::{decl_derive, AddBounds, Structure};

struct Static {
    literal: LitStr,
    ident: Ident,
}

impl Parse for Static {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let expr = Expr::parse(input)?;
        match expr {
            Expr::Tuple(expr) => {
                let mut elems = expr.elems.iter().cloned();
                let literal = elems
                    .next()
                    .ok_or_else(|| syn::Error::new_spanned(&expr, "invalid empty tuple"))?;
                let ident = elems.next();
                if elems.next().is_some() {
                    return Err(syn::Error::new_spanned(
                        &expr,
                        "invalid tuple with more than two elements",
                    ));
                }
                let Expr::Lit(ExprLit {
                    lit: Lit::Str(literal), ..
                }) = literal else {
                    return Err(syn::Error::new_spanned(
                        literal,
                        "expected an UTF-8 string literal",
                    ));
                };

                let ident = if let Some(ident) = ident {
                    syn::parse2::<Ident>(ident.into_token_stream())?
                } else {
                    Ident::new(&literal.value().to_uppercase(), literal.span())
                };

                Ok(Self { literal, ident })
            }
            Expr::Lit(expr) => match expr.lit {
                Lit::Str(str) => Ok(Self {
                    ident: Ident::new(&str.value().to_uppercase(), str.span()),
                    literal: str,
                }),
                _ => Err(syn::Error::new_spanned(
                    expr,
                    "expected an UTF-8 string literal",
                )),
            },
            _ => Err(syn::Error::new_spanned(
                expr,
                "expected a string literal or a tuple expression",
            )),
        }
    }
}

struct Syms(Vec<Static>);

impl Parse for Syms {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let parsed = Punctuated::<Static, Token![,]>::parse_terminated(input)?;
        let literals = parsed.into_iter().collect();
        Ok(Self(literals))
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn static_syms(input: TokenStream) -> TokenStream {
    let literals = parse_macro_input!(input as Syms).0;

    let consts = literals.iter().enumerate().map(|(mut idx, lit)| {
        let doc = format!(
            "Symbol for the \"{}\" string.",
            lit.literal
                .value()
                .replace('<', r"\<")
                .replace('>', r"\>")
                .replace('*', r"\*")
        );
        let ident = &lit.ident;
        idx += 1;
        quote! {
            #[doc = #doc]
            pub const #ident: Self = unsafe { Self::new_unchecked(#idx) };
        }
    });

    let literals = literals.iter().map(|lit| &lit.literal).collect::<Vec<_>>();

    let caches = quote! {
        type Set<T> = ::indexmap::IndexSet<T, ::core::hash::BuildHasherDefault<::rustc_hash::FxHasher>>;

        /// Ordered set of commonly used static `UTF-8` strings.
        ///
        /// # Note
        ///
        /// `COMMON_STRINGS_UTF8`, `COMMON_STRINGS_UTF16` and the constants
        /// defined in [`Sym`] must always be in sync.
        pub(super) static COMMON_STRINGS_UTF8: ::phf::OrderedSet<&'static str> = {
            const COMMON_STRINGS: ::phf::OrderedSet<&'static str> = ::phf::phf_ordered_set! {
                #(#literals),*
            };
            // A `COMMON_STRINGS` of size `usize::MAX` would cause an overflow on our `Interner`
            ::static_assertions::const_assert!(COMMON_STRINGS.len() < usize::MAX);
            COMMON_STRINGS
        };

        /// Ordered set of commonly used static `UTF-16` strings.
        ///
        /// # Note
        ///
        /// `COMMON_STRINGS_UTF8`, `COMMON_STRINGS_UTF16` and the constants
        /// defined in [`Sym`] must always be in sync.
        // FIXME: use phf when const expressions are allowed.
        // <https://github.com/rust-phf/rust-phf/issues/188>
        pub(super) static COMMON_STRINGS_UTF16: ::once_cell::sync::Lazy<Set<&'static [u16]>> =
            ::once_cell::sync::Lazy::new(|| {
                let mut set = Set::with_capacity_and_hasher(
                    COMMON_STRINGS_UTF8.len(),
                    ::core::hash::BuildHasherDefault::default()
                );
                #(
                    set.insert(::boa_macros::utf16!(#literals));
                )*
                set
            });
    };

    quote! {
        impl Sym {
            #(#consts)*
        }
        #caches
    }
    .into()
}

/// Construct a utf-16 array literal from a utf-8 [`str`] literal.
#[proc_macro]
pub fn utf16(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);
    let utf8 = literal.value();
    let utf16 = utf8.encode_utf16().collect::<Vec<_>>();
    quote! {
        [#(#utf16),*].as_slice()
    }
    .into()
}

decl_derive! {
    [Trace, attributes(unsafe_ignore_trace)] =>
    /// Derive the `Trace` trait.
    derive_trace
}

/// Derives the `Trace` trait.
fn derive_trace(mut s: Structure<'_>) -> proc_macro2::TokenStream {
    s.filter(|bi| {
        !bi.ast()
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("unsafe_ignore_trace"))
    });
    let trace_body = s.each(|bi| quote!(mark(#bi)));

    s.add_bounds(AddBounds::Fields);
    let trace_impl = s.unsafe_bound_impl(
        quote!(::boa_gc::Trace),
        quote! {
            #[inline]
            unsafe fn trace(&self) {
                #[allow(dead_code)]
                fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    unsafe {
                        ::boa_gc::Trace::trace(it);
                    }
                }
                match *self { #trace_body }
            }
            #[inline]
            unsafe fn root(&self) {
                #[allow(dead_code)]
                fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    unsafe {
                        ::boa_gc::Trace::root(it);
                    }
                }
                match *self { #trace_body }
            }
            #[inline]
            unsafe fn unroot(&self) {
                #[allow(dead_code)]
                fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    unsafe {
                        ::boa_gc::Trace::unroot(it);
                    }
                }
                match *self { #trace_body }
            }
            #[inline]
            fn run_finalizer(&self) {
                ::boa_gc::Finalize::finalize(self);
                #[allow(dead_code)]
                fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    unsafe {
                        ::boa_gc::Trace::run_finalizer(it);
                    }
                }
                match *self { #trace_body }
            }
        },
    );

    // We also implement drop to prevent unsafe drop implementations on this
    // type and encourage people to use Finalize. This implementation will
    // call `Finalize::finalize` if it is safe to do so.
    let drop_impl = s.unbound_impl(
        quote!(::core::ops::Drop),
        quote! {
            fn drop(&mut self) {
                if ::boa_gc::finalizer_safe() {
                    ::boa_gc::Finalize::finalize(self);
                }
            }
        },
    );

    quote! {
        #trace_impl
        #drop_impl
    }
}

decl_derive! {
    [Finalize] =>
    /// Derive the `Finalize` trait.
    derive_finalize
}

/// Derives the `Finalize` trait.
#[allow(clippy::needless_pass_by_value)]
fn derive_finalize(s: Structure<'_>) -> proc_macro2::TokenStream {
    s.unbound_impl(quote!(::boa_gc::Finalize), quote!())
}

/// Derives the `TryFromJs` trait, with the `#[boa()]` attribute.
///
/// # Panics
///
/// It will panic if the user tries to derive the `TryFromJs` trait in an `enum` or a tuple struct.
#[proc_macro_derive(TryFromJs, attributes(boa))]
pub fn derive_try_from_js(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(data) = input.data else {
        panic!("you can only derive TryFromJs for structs");
    };

    let Fields::Named(fields) = data.fields else {
        panic!("you can only derive TryFromJs for named-field structs")
    };

    let conv = generate_conversion(fields).unwrap_or_else(to_compile_errors);

    let type_name = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl boa_engine::value::TryFromJs for #type_name {
            fn try_from_js(value: &boa_engine::JsValue, context: &mut boa_engine::Context)
                -> boa_engine::JsResult<Self> {
                match value {
                    boa_engine::JsValue::Object(o) => {#conv},
                    _ => Err(boa_engine::JsError::from(
                        boa_engine::JsNativeError::typ()
                            .with_message("cannot convert value to a #type_name")
                    )),
                }
            }
        }
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}

/// Generates the conversion field by field.
fn generate_conversion(fields: FieldsNamed) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    use syn::spanned::Spanned;

    let mut field_list = Vec::with_capacity(fields.named.len());
    let mut final_fields = Vec::with_capacity(fields.named.len());

    for field in fields.named {
        let span = field.span();
        let name = field.ident.ok_or_else(|| {
            vec![syn::Error::new(
                span,
                "you can only derive `TryFromJs` for named-field structs",
            )]
        })?;

        let name_str = format!("{name}");
        field_list.push(name.clone());

        let error_str = format!("cannot get property {name_str} of value");

        let mut from_js_with = None;
        if let Some(attr) = field
            .attrs
            .into_iter()
            .find(|attr| attr.path().is_ident("boa"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("from_js_with") {
                    let value = meta.value()?;
                    from_js_with = Some(value.parse::<LitStr>()?);
                    Ok(())
                } else {
                    Err(meta.error(
                        "invalid syntax in the `#[boa()]` attribute. \
                              Note that this attribute only accepts the following syntax: \
                            `#[boa(from_js_with = \"fully::qualified::path\")]`",
                    ))
                }
            })
            .map_err(|err| vec![err])?;
        }

        if let Some(method) = from_js_with {
            let ident = Ident::new(&method.value(), method.span());
            final_fields.push(quote! {
                let #name = #ident(props.get(&#name_str.into()).ok_or_else(|| {
                    boa_engine::JsError::from(
                        boa_engine::JsNativeError::typ().with_message(#error_str)
                    )
                })?.value().ok_or_else(|| {
                    boa_engine::JsError::from(
                        boa_engine::JsNativeError::typ().with_message(#error_str)
                    )
                })?, context)?;
            });
        } else {
            final_fields.push(quote! {
                let #name = props.get(&#name_str.into()).ok_or_else(|| {
                    boa_engine::JsError::from(
                        boa_engine::JsNativeError::typ().with_message(#error_str)
                    )
                })?.value().ok_or_else(|| {
                    boa_engine::JsError::from(
                        boa_engine::JsNativeError::typ().with_message(#error_str)
                    )
                })?.clone().try_js_into(context)?;
            });
        }
    }

    Ok(quote! {
        let o = o.borrow();
        let props = o.properties();
        #(#final_fields)*
        Ok(Self {
            #(#field_list),*
        })
    })
}

/// Generates a list of compile errors.
#[allow(clippy::needless_pass_by_value)]
fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
