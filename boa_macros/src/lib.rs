use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use synstructure::{decl_derive, AddBounds, Structure};

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

decl_derive!([Trace, attributes(unsafe_ignore_trace)] => derive_trace);

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
                #[inline]
                fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    unsafe {
                        ::boa_gc::Trace::trace(it);
                    }
                }
                match *self { #trace_body }
            }
            #[inline]
            unsafe fn weak_trace(&self) {
                #[allow(dead_code, unreachable_code)]
                #[inline]
                fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    unsafe {
                        ::boa_gc::Trace::weak_trace(it)
                    }
                }
                match *self { #trace_body }
            }
            #[inline]
            unsafe fn root(&self) {
                #[allow(dead_code)]
                #[inline]
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
                #[inline]
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
                #[inline]
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
        quote!(::std::ops::Drop),
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

decl_derive!([Finalize] => derive_finalize);

fn derive_finalize(s: Structure<'_>) -> proc_macro2::TokenStream {
    s.unbound_impl(quote!(::boa_gc::Finalize), quote!())
}
