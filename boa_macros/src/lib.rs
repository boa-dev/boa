//! Macros for the Boa JavaScript engine.

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
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    LitStr, Token,
};
use synstructure::{decl_derive, AddBounds, Structure};

struct Syms(Vec<LitStr>);

impl Parse for Syms {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let parsed = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?;
        let literals = parsed.into_iter().collect();
        Ok(Self(literals))
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn static_syms(input: TokenStream) -> TokenStream {
    let literals = parse_macro_input!(input as Syms).0;

    let consts = literals.iter().enumerate().map(|(mut idx, lit)| {
        let ident = lit.value();
        let (doc, ident) = match &*ident {
            "" => (
                String::from("Symbol for the empty string."),
                String::from("EMPTY_STRING"),
            ),
            "<main>" => (
                String::from("Symbol for the `<main>` string."),
                String::from("MAIN"),
            ),
            ident => (
                format!("Symbol for the `{ident}` string.",),
                ident.to_uppercase(),
            ),
        };
        let ident = Ident::new(&ident, lit.span());
        idx += 1;
        quote! {
            #[doc = #doc]
            pub const #ident: Self = unsafe { Self::new_unchecked(#idx) };
        }
    });

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
        // FIXME: use phf when const expressions are allowed. https://github.com/rust-phf/rust-phf/issues/188
        pub(super) static COMMON_STRINGS_UTF16: ::once_cell::sync::Lazy<Set<&'static [u16]>> =
            ::once_cell::sync::Lazy::new(|| {
                let mut set = Set::with_capacity_and_hasher(COMMON_STRINGS_UTF8.len(), ::core::hash::BuildHasherDefault::default());
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
    /// Derive the Trace trait.
    derive_trace
}

fn derive_trace(mut s: Structure<'_>) -> proc_macro2::TokenStream {
    s.filter(|bi| {
        !bi.ast()
            .attrs
            .iter()
            .any(|attr| attr.path.is_ident("unsafe_ignore_trace"))
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
    /// Derive the Finalize trait.
    derive_finalize
}

#[allow(clippy::needless_pass_by_value)]
fn derive_finalize(s: Structure<'_>) -> proc_macro2::TokenStream {
    s.unbound_impl(quote!(::boa_gc::Finalize), quote!())
}
