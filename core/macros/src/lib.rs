//! Macros for the Boa JavaScript engine.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]

use crate::utils::RenameScheme;
use cow_utils::CowUtils;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{ToTokens, quote};
use syn::{
    Data, DeriveInput, Expr, ExprLit, Fields, FieldsNamed, Ident, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};
use synstructure::{AddBounds, Structure, decl_derive};

mod embedded_module_loader;

mod class;
mod module;
mod utils;
mod value;

/// The `js_value!` macro creates a `JsValue` instance based on a JSON-like DSL.
#[proc_macro]
pub fn js_value(input: TokenStream) -> TokenStream {
    value::js_value_impl(proc_macro2::TokenStream::from(input)).into()
}

/// Create a `JsObject` object from a simpler DSL that resembles JSON.
#[proc_macro]
pub fn js_object(input: TokenStream) -> TokenStream {
    value::js_object_impl(proc_macro2::TokenStream::from(input)).into()
}

/// Implementation of the inner iterator of the `embed_module!` macro. All
/// arguments are required.
///
/// # Warning
/// This should not be used directly as is, and instead should be used through
/// the `embed_module!` macro in `boa_interop` for convenience.
#[proc_macro]
pub fn embed_module_inner(input: TokenStream) -> TokenStream {
    embedded_module_loader::embed_module_impl(input)
}

/// `boa_class` proc macro attribute that applies to an `impl XYZ` block and
/// add a `[boa_engine::JsClass]` implementation for it.
///
/// It will transform functions in the `impl ...` block as follow (by default, see
/// below):
/// 1. `fn some_method(&self, ...) -> ... {}` will be added as class methods with
///    the name `some_method`, borrowing the object for the ref. This is dangerous
///    if the function execute/eval JavaScript back (potentially leading to a
///    `BorrowError`).
/// 2. `fn some_method(&mut self, ...) -> ... {}` will be added as class methods,
///    similar to the above but borrowing as mutable at runtime.
/// 3. `fn some_method(...) -> ... {}` (no self mention) will be added as a
///    static method.
/// 4. `#[boa(constructor)] fn ...(...) -> Self {}` (or returning `JsResult<Self>`)
///    will be used as the constructor of the class. If no constructor is declared,
///    `Default::default()` will be used instead. If the `Default` trait is not
///    defined for the type, an error will happen.
/// 5. `#[boa(getter)]`
///
/// To change this behaviour, you can use the following attributes on the function
/// declarations:
/// 1. `#[boa(rename = "...")]` renames the function in JavaScript with the string.
/// 2. `#[boa(getter)]` will declare a getter accessor.
/// 2. `#[boa(setter)]` will declare a setter accessor.
/// 3. `#[boa(static)]` will declare a static method.
/// 4. `#[boa(method)]` will declare a method.
/// 5. `#[boa(constructor)]` will declare a constructor.
/// 6. `#[boa(length = 123)]` sets the length of the function in JavaScript (ie. its
///    number of arguments accepted).
///
/// Multiple of those attributes can be added to a single method.
///
/// The top level `boa_class` supports the following:
/// 1. `#[boa_class(rename = "...")]` sets the name of the class in JavaScript.
/// 2. `#[boa(rename_all = "camelCase")]` will change the naming scheme of verbatim
///    to using "camelCase" or "none".
///
/// # Warning
/// This should not be used directly as is, and instead should be used through
/// the `embed_module!` macro in `boa_interop` for convenience.
#[proc_macro_attribute]
pub fn boa_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    class::class_impl(attr, item)
}

/// `boa_module` proc macro attribute for declaring a `boa_engine::Module` based
/// on a Rust module. The original Rust module will also be exposed as is.
///
/// This macro exports two functions out of the existing module (and those
/// functions must not exist in the declared module):
///
/// ## `boa_module(realm: Option<Realm>, context: &mut Context) -> JsResult<Module>`
/// Create a JavaScript module from the rust module's content.
///
/// ## `boa_register(realm: Option<Realm>, context: &mut Context) -> JsResult<()>`
/// Register the constants, classes and functions from the module in the global
/// scope of the Realm (if specified) or the context (if no realm).
#[proc_macro_attribute]
pub fn boa_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    module::module_impl(attr, item)
}

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
                    Ident::new(&literal.value().cow_to_uppercase(), literal.span())
                };

                Ok(Self { literal, ident })
            }
            Expr::Lit(expr) => match expr.lit {
                Lit::Str(str) => Ok(Self {
                    ident: Ident::new(&str.value().cow_to_uppercase(), str.span()),
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
                .cow_replace('<', r"\<")
                .cow_replace('>', r"\>")
                .cow_replace('*', r"\*")
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

/// Convert a utf8 string literal to a `JsString`
#[proc_macro]
pub fn js_str(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);

    let mut is_latin1 = true;
    let codepoints = literal
        .value()
        .encode_utf16()
        .map(|x| {
            if x > u8::MAX.into() {
                is_latin1 = false;
            }
            Literal::u16_unsuffixed(x)
        })
        .collect::<Vec<_>>();
    if is_latin1 {
        quote! {
            ::boa_engine::string::JsStr::latin1([#(#codepoints),*].as_slice())
        }
    } else {
        quote! {
            ::boa_engine::string::JsStr::utf16([#(#codepoints),*].as_slice())
        }
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

    let renaming = match RenameScheme::from_named_attrs(&mut input.attrs.clone(), "rename_all") {
        Ok(renaming) => renaming.unwrap_or(RenameScheme::None),
        Err((span, msg)) => return syn::Error::new(span, msg).to_compile_error().into(),
    };

    let conv = generate_conversion(fields, renaming).unwrap_or_else(to_compile_errors);

    let type_name = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl ::boa_engine::value::TryFromJs for #type_name {
            fn try_from_js(value: &boa_engine::JsValue, context: &mut boa_engine::Context)
                -> boa_engine::JsResult<Self> {
                let o = value.as_object().ok_or_else(|| ::boa_engine::JsError::from(
                    ::boa_engine::JsNativeError::typ()
                        .with_message("value is not an object")
                ))?;
                #conv
            }
        }
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}

/// Generates the conversion field by field.
fn generate_conversion(
    fields: FieldsNamed,
    rename: RenameScheme,
) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
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

        field_list.push(name.clone());

        let mut from_js_with = None;
        let mut field_name = rename.rename(format!("{name}"));
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
                } else if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    field_name = value.parse::<LitStr>()?.value();
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

        let error_str = format!("cannot get property {name} of value");
        final_fields.push(quote! {
            let #name = match props.get(&::boa_engine::js_string!(#field_name).into()) {
                Some(pd) => pd.value().ok_or_else(|| ::boa_engine::JsError::from(
                        ::boa_engine::JsNativeError::typ().with_message(#error_str)
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

/// Derives the `TryIntoJs` trait, with the `#[boa()]` attribute.
///
/// # Panics
///
/// It will panic if the user tries to derive the `TryIntoJs` trait in an `enum` or a tuple struct.
#[proc_macro_derive(TryIntoJs, attributes(boa))]
pub fn derive_try_into_js(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(data) = input.data else {
        panic!("you can only derive TryFromJs for structs");
    };
    // TODO: Enums ?

    let Fields::Named(fields) = data.fields else {
        panic!("you can only derive TryFromJs for named-field structs")
    };

    let renaming = match RenameScheme::from_named_attrs(&mut input.attrs.clone(), "rename_all") {
        Ok(renaming) => renaming.unwrap_or(RenameScheme::None),
        Err((span, msg)) => return syn::Error::new(span, msg).to_compile_error().into(),
    };

    let props = generate_obj_properties(fields, renaming)
        .map_err(|err| vec![err])
        .unwrap_or_else(to_compile_errors);

    let type_name = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl ::boa_engine::value::TryIntoJs for #type_name {
            fn try_into_js(&self, context: &mut boa_engine::Context) -> boa_engine::JsResult<boa_engine::JsValue> {
                let obj = boa_engine::JsObject::default();
                #props
                boa_engine::JsResult::Ok(obj.into())
            }
        }
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}

/// Generates property creation for object.
fn generate_obj_properties(
    fields: FieldsNamed,
    renaming: RenameScheme,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    use syn::spanned::Spanned;

    let mut prop_ctors = Vec::with_capacity(fields.named.len());

    for field in fields.named {
        let span = field.span();
        let name = field.ident.ok_or_else(|| {
            syn::Error::new(
                span,
                "you can only derive `TryIntoJs` for named-field structs",
            )
        })?;

        let mut into_js_with = None;
        let mut prop_key = renaming.rename(format!("{name}"));
        let mut skip = false;

        for attr in field
            .attrs
            .into_iter()
            .filter(|attr| attr.path().is_ident("boa"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("into_js_with") {
                    let value = meta.value()?;
                    into_js_with = Some(value.parse::<LitStr>()?);
                    Ok(())
                } else if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    prop_key = value.parse::<LitStr>()?.value();
                    Ok(())
                } else if meta.path.is_ident("skip") & meta.input.is_empty() {
                    skip = true;
                    Ok(())
                } else {
                    Err(meta.error(
                        "invalid syntax in the `#[boa()]` attribute. \
                              Note that this attribute only accepts the following syntax: \
                            \n* `#[boa(into_js_with = \"fully::qualified::path\")]`\
                            \n* `#[boa(rename = \"jsPropertyName\")]` \
                            \n* `#[boa(skip)]` \
                            ",
                    ))
                }
            })?;
        }

        if skip {
            continue;
        }

        let value = if let Some(into_js_with) = into_js_with {
            let into_js_with = Ident::new(&into_js_with.value(), into_js_with.span());
            quote! { #into_js_with(&self.#name, context)? }
        } else {
            quote! { boa_engine::value::TryIntoJs::try_into_js(&self.#name, context)? }
        };
        prop_ctors.push(quote! {
            obj.create_data_property_or_throw(boa_engine::js_string!(#prop_key), #value, context)?;
        });
    }

    Ok(quote! { #(#prop_ctors)* })
}
