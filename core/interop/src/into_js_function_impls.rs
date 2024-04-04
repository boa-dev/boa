//! Implementations of the `IntoJsFunction` trait for various function signatures.

use std::cell::RefCell;

use boa_engine::{Context, NativeFunction};

use crate::{IntoJsFunction, IntoJsFunctionUnsafe, TryFromJsArgument, TryIntoJsResult};

/// A token to represent the context argument in the function signature.
/// This should not be used directly and has no external meaning.
#[derive(Debug, Copy, Clone)]
pub struct ContextArgToken;

macro_rules! impl_into_js_function {
    ($($id: ident: $t: ident),*) => {
        unsafe impl<$($t,)* R, T> IntoJsFunctionUnsafe<($($t,)*), R> for T
        where
            $($t: TryFromJsArgument + 'static,)*
            R: TryIntoJsResult,
            T: FnMut($($t,)*) -> R + 'static,
        {
            #[allow(unused_variables)]
            unsafe fn into_js_function_unsafe(self, _context: &mut Context) -> NativeFunction {
                let s = RefCell::new(self);
                unsafe {
                    NativeFunction::from_closure(move |this, args, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        let r = s.borrow_mut()( $($id),* );
                        r.try_into_js_result(ctx)
                    })
                }
            }
        }

        unsafe impl<$($t,)* R, T> IntoJsFunctionUnsafe<($($t,)* ContextArgToken), R> for T
        where
            $($t: TryFromJsArgument + 'static,)*
            R: TryIntoJsResult,
            T: FnMut($($t,)* &mut Context) -> R + 'static,
        {
            #[allow(unused_variables)]
            unsafe fn into_js_function_unsafe(self, _context: &mut Context) -> NativeFunction {
                let s = RefCell::new(self);
                unsafe {
                    NativeFunction::from_closure(move |this, args, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        let r = s.borrow_mut()( $($id,)* ctx);
                        r.try_into_js_result(ctx)
                    })
                }
            }
        }

        // Safe versions for `Fn(..) -> ...`.
        impl<$($t,)* R, T> IntoJsFunction<($($t,)*), R> for T
        where
            $($t: TryFromJsArgument + 'static,)*
            R: TryIntoJsResult,
            T: Fn($($t,)*) -> R + 'static + Copy,
        {
            #[allow(unused_variables)]
            fn into_js_function(self, _context: &mut Context) -> NativeFunction {
                let s = self;
                unsafe {
                    NativeFunction::from_closure(move |this, args, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        let r = s( $($id),* );
                        r.try_into_js_result(ctx)
                    })
                }
            }
        }

        impl<$($t,)* R, T> IntoJsFunction<($($t,)* ContextArgToken), R> for T
        where
            $($t: TryFromJsArgument + 'static,)*
            R: TryIntoJsResult,
            T: Fn($($t,)* &mut Context) -> R + 'static + Copy,
        {
            #[allow(unused_variables)]
            fn into_js_function(self, _context: &mut Context) -> NativeFunction {
                let s = self;
                unsafe {
                    NativeFunction::from_closure(move |this, args, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        let r = s( $($id,)* ctx);
                        r.try_into_js_result(ctx)
                    })
                }
            }
        }
    };
}

unsafe impl<R, T> IntoJsFunctionUnsafe<(), R> for T
where
    R: TryIntoJsResult,
    T: FnMut() -> R + 'static,
{
    unsafe fn into_js_function_unsafe(self, _context: &mut Context) -> NativeFunction {
        let s = RefCell::new(self);
        unsafe {
            NativeFunction::from_closure(move |_this, _args, ctx| {
                let r = s.borrow_mut()();
                r.try_into_js_result(ctx)
            })
        }
    }
}

impl<R, T> IntoJsFunction<(), R> for T
where
    R: TryIntoJsResult,
    T: Fn() -> R + 'static + Copy,
{
    fn into_js_function(self, _context: &mut Context) -> NativeFunction {
        let s = self;
        unsafe {
            NativeFunction::from_closure(move |_this, _args, ctx| {
                let r = s();
                r.try_into_js_result(ctx)
            })
        }
    }
}

// Currently implemented up to 12 arguments. The empty argument list
// is implemented separately above.
// Consider that JsRest and JsThis are part of this list, but Context
// is not, as it is a special specialization of the template.
impl_into_js_function!(a: A);
impl_into_js_function!(a: A, b: B);
impl_into_js_function!(a: A, b: B, c: C);
impl_into_js_function!(a: A, b: B, c: C, d: D);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K);
impl_into_js_function!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L);
