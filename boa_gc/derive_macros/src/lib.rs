use quote::quote;
use synstructure::{decl_derive, AddBounds, Structure};

decl_derive!([Trace, attributes(unsafe_ignore_trace)] => derive_trace);

fn derive_trace(mut s: Structure<'_>) -> proc_macro2::TokenStream {
    s.filter(|bi| {
        !bi.ast()
            .attrs
            .iter()
            .any(|attr| attr.path.is_ident("unsafe_ignore_trace"))
    });
    let trace_body = s.each(|bi| quote!(mark(#bi)));
    let weak_trace_body = s.each(|bi| quote!(mark(#bi, queue)));

    s.add_bounds(AddBounds::Fields);
    let trace_impl = s.unsafe_bound_impl(
        quote!(::boa_gc::Trace),
        quote! {
            #[inline] unsafe fn trace(&self) {
                #[allow(dead_code)]
                #[inline]
                unsafe fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    ::boa_gc::Trace::trace(it);
                }
                match *self { #trace_body }
            }
            #[inline] unsafe fn is_marked_ephemeron(&self) -> bool {
                false
            }
            #[inline] unsafe fn weak_trace(&self, queue: &mut Vec<GcPointer>) {
                #[allow(dead_code, unreachable_code)]
                #[inline]
                unsafe fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T, queue: &mut Vec<::gc::GcPointer>) {
                    ::boa_gc::Trace::weak_trace(it, queue)
                }
                match *self { #weak_trace_body }
            }
            #[inline] unsafe fn root(&self) {
                #[allow(dead_code)]
                #[inline]
                unsafe fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    ::boa_gc::Trace::root(it);
                }
                match *self { #trace_body }
            }
            #[inline] unsafe fn unroot(&self) {
                #[allow(dead_code)]
                #[inline]
                unsafe fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    ::boa_gc::Trace::unroot(it);
                }
                match *self { #trace_body }
            }
            #[inline] fn finalize_glue(&self) {
                ::boa_gc::Finalize::finalize(self);
                #[allow(dead_code)]
                #[inline]
                fn mark<T: ::boa_gc::Trace + ?Sized>(it: &T) {
                    ::boa_gc::Trace::finalize_glue(it);
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
