//! Macros for the Boa JavaScript engine.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]

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
                    lit: Lit::Str(literal),
                    ..
                }) = literal
                else {
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
    [Trace, attributes(boa_gc, unsafe_ignore_trace)] =>
    /// Derive the `Trace` trait.
    derive_trace
}

/// Derives the `Trace` trait.
#[allow(clippy::too_many_lines)]
fn derive_trace(mut s: Structure<'_>) -> proc_macro2::TokenStream {
    struct EmptyTrace {
        copy: bool,
        drop: bool,
    }

    impl Parse for EmptyTrace {
        fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
            let i: Ident = input.parse()?;

            if i != "empty_trace" && i != "unsafe_empty_trace" && i != "unsafe_no_drop" {
                let msg = format!(
                    "expected token \"empty_trace\", \"unsafe_empty_trace\" or \"unsafe_no_drop\", found {i:?}"
                );
                return Err(syn::Error::new_spanned(i.clone(), msg));
            }

            Ok(Self {
                copy: i == "empty_trace",
                drop: i == "empty_trace" || i != "unsafe_no_drop",
            })
        }
    }

    let mut drop = true;

    for attr in &s.ast().attrs {
        if attr.path().is_ident("boa_gc") {
            let trace = match attr.parse_args::<EmptyTrace>() {
                Ok(t) => t,
                Err(e) => return e.into_compile_error(),
            };

            if trace.copy {
                s.add_where_predicate(syn::parse_quote!(Self: Copy));
            }

            if !trace.drop {
                drop = false;
                continue;
            }

            return s.unsafe_bound_impl(
                quote!(::boa_gc::Trace),
                quote! {
                    #[inline(always)]
                    unsafe fn trace(&self, _tracer: &mut ::boa_gc::Tracer) {}
                    #[inline(always)]
                    unsafe fn trace_non_roots(&self) {}
                    #[inline]
                    fn run_finalizer(&self) {
                        ::boa_gc::Finalize::finalize(self)
                    }
                },
            );
        }
    }

    s.filter(|bi| {
        !bi.ast()
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("unsafe_ignore_trace"))
    });
    let trace_body = s.each(|bi| quote!(::boa_gc::Trace::trace(#bi, tracer)));
    let trace_other_body = s.each(|bi| quote!(mark(#bi)));

    s.add_bounds(AddBounds::Fields);
    let trace_impl = s.unsafe_bound_impl(
        quote!(::boa_gc::Trace),
        quote! {
            #[inline]
            unsafe fn trace(&self, tracer: &mut ::boa_gc::Tracer) {
                #[allow(dead_code)]
                let mut mark = |it: &dyn ::boa_gc::Trace| {
                    // SAFETY: The implementor must ensure that `trace` is correctly implemented.
                    unsafe {
                        ::boa_gc::Trace::trace(it, tracer);
                    }
                };
                match *self { #trace_body }
            }
            #[inline]
            unsafe fn trace_non_roots(&self) {
                #[allow(dead_code)]
                fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    // SAFETY: The implementor must ensure that `trace_non_roots` is correctly implemented.
                    unsafe {
                        ::boa_gc::Trace::trace_non_roots(it);
                    }
                }
                match *self { #trace_other_body }
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
                match *self { #trace_other_body }
            }
        },
    );

    // We also implement drop to prevent unsafe drop implementations on this
    // type and encourage people to use Finalize. This implementation will
    // call `Finalize::finalize` if it is safe to do so.
    let drop_impl = if drop {
        s.unbound_impl(
            quote!(::core::ops::Drop),
            quote! {
                #[allow(clippy::inline_always)]
                #[inline(always)]
                fn drop(&mut self) {
                    if ::boa_gc::finalizer_safe() {
                        ::boa_gc::Finalize::finalize(self);
                    }
                }
            },
        )
    } else {
        quote!()
    };

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

decl_derive! {
    [JsData] =>
    /// Derive the `JsData` trait.
    derive_js_data
}

/// Derives the `JsData` trait.
#[allow(clippy::needless_pass_by_value)]
fn derive_js_data(s: Structure<'_>) -> proc_macro2::TokenStream {
    s.unbound_impl(quote!(::boa_engine::JsData), quote!())
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
        impl ::boa_engine::value::TryFromJs for #type_name {
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

        final_fields.push(quote! {
            let #name = match props.get(&::boa_engine::js_string!(#name_str).into()) {
                Some(pd) => pd.value().ok_or_else(|| ::boa_engine::JsError::from(
                        boa_engine::JsNativeError::typ().with_message(#error_str)
                    ))?.clone().try_js_into(context)?,
                None => ::boa_engine::JsValue::undefined().try_js_into(context)?,
            };
        });

        if let Some(method) = from_js_with {
            let ident = Ident::new(&method.value(), method.span());
            final_fields.push(quote! {
                let #name = #ident(&#name, context)?;
            });
        }
    }

    // TODO: this could possibly skip accessors. Consider using `JsObject::get` instead.
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
