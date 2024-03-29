//! Implementations of the `IntoJsFunction` trait for various function signatures.

use crate::{IntoJsFunction, TryFromJsArgument};
use boa_engine::{Context, JsValue, NativeFunction};
use std::cell::RefCell;

/// A token to represent the context argument in the function signature.
/// This should not be used directly and has no external meaning.
#[derive(Debug, Copy, Clone)]
pub struct ContextArgToken();

macro_rules! impl_into_js_function {
    ($($id: ident: $t: ident),*) => {
        impl<$($t,)* R, T> IntoJsFunction<($($t,)*), R> for T
        where
            $($t: TryFromJsArgument + 'static,)*
            R: Into<JsValue>,
            T: FnMut($($t,)*) -> R + 'static,
        {
            #[allow(unused_variables)]
            fn into_js_function(self, _context: &mut Context) -> NativeFunction {
                let s = RefCell::new(self);
                unsafe {
                    NativeFunction::from_closure(move |this, args, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        let r = s.borrow_mut()( $($id),* );
                        Ok(r.into())
                    })
                }
            }
        }

        impl<$($t,)* R, T> IntoJsFunction<($($t,)* ContextArgToken), R> for T
        where
            $($t: TryFromJsArgument + 'static,)*
            R: Into<JsValue>,
            T: FnMut($($t,)* &mut Context) -> R + 'static,
        {
            #[allow(unused_variables)]
            fn into_js_function(self, _context: &mut Context) -> NativeFunction {
                let s = RefCell::new(self);
                unsafe {
                    NativeFunction::from_closure(move |this, args, ctx| {
                        let rest = args;
                        $(
                            let ($id, rest) = $t::try_from_js_argument(this, rest, ctx)?;
                        )*
                        let r = s.borrow_mut()( $($id,)* ctx);
                        Ok(r.into())
                    })
                }
            }
        }
    };
}

impl<R, T> IntoJsFunction<(), R> for T
where
    R: Into<JsValue>,
    T: FnMut() -> R + 'static,
{
    fn into_js_function(self, _context: &mut Context) -> NativeFunction {
        let s = RefCell::new(self);
        unsafe {
            NativeFunction::from_closure(move |_this, _args, _ctx| {
                let r = s.borrow_mut()();
                Ok(r.into())
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
