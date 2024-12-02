//! Interop utilities between Boa and its host.

use boa_engine::module::SyntheticModuleInitializer;
use boa_engine::object::Object;
use boa_engine::value::TryFromJs;
use boa_engine::{
    Context, JsNativeError, JsResult, JsString, JsValue, Module, NativeFunction, NativeObject,
};
use std::ops::Deref;

pub use boa_engine;
use boa_gc::{GcRef, GcRefMut};
pub use boa_macros;

pub mod loaders;
pub mod macros;

/// Internal module only.
pub(crate) mod private {
    /// A sealed trait to prevent users from implementing the `IntoJsModuleFunction`
    /// and `IntoJsFunctionUnsafe` traits to their own types.
    pub trait IntoJsFunctionSealed<A, R> {}
}

/// A trait to convert a type into a JS module.
pub trait IntoJsModule {
    /// Converts the type into a JS module.
    fn into_js_module(self, context: &mut Context) -> Module;
}

impl<T: IntoIterator<Item = (JsString, NativeFunction)> + Clone> IntoJsModule for T {
    fn into_js_module(self, context: &mut Context) -> Module {
        let (names, fns): (Vec<_>, Vec<_>) = self.into_iter().unzip();
        let exports = names.clone();

        Module::synthetic(
            exports.as_slice(),
            unsafe {
                SyntheticModuleInitializer::from_closure(move |module, context| {
                    for (name, f) in names.iter().zip(fns.iter()) {
                        module
                            .set_export(name, f.clone().to_js_function(context.realm()).into())?;
                    }
                    Ok(())
                })
            },
            None,
            None,
            context,
        )
    }
}

/// A trait to convert a type into a JS function.
/// This trait does not require the implementing type to be `Copy`, which
/// can lead to undefined behaviour if it contains Garbage Collected objects.
///
/// This trait is implemented for functions with various signatures.
///
/// For example:
/// ```
/// # use boa_engine::{Context, JsValue, NativeFunction};
/// # use boa_interop::UnsafeIntoJsFunction;
/// # let mut context = Context::default();
/// let f = |a: i32, b: i32| a + b;
/// let f = unsafe { f.into_js_function_unsafe(&mut context) };
/// let result = f
///     .call(
///         &JsValue::undefined(),
///         &[JsValue::from(1), JsValue::from(2)],
///         &mut context,
///     )
///     .unwrap();
/// assert_eq!(result, JsValue::new(3));
/// ```
///
/// Since the `IntoJsFunctionUnsafe` trait is implemented for `FnMut`, you can
/// also use closures directly:
/// ```
/// # use boa_engine::{Context, JsValue, NativeFunction};
/// # use boa_interop::UnsafeIntoJsFunction;
/// # use std::cell::RefCell;
/// # use std::rc::Rc;
/// # let mut context = Context::default();
/// let mut x = Rc::new(RefCell::new(0));
/// // Because NativeFunction takes ownership of the closure,
/// // the compiler cannot be certain it won't outlive `x`, so
/// // we need to create a `Rc<RefCell>` and share it.
/// let f = unsafe {
///     let x = x.clone();
///     move |a: i32| *x.borrow_mut() += a
/// };
/// let f = unsafe { f.into_js_function_unsafe(&mut context) };
/// f.call(&JsValue::undefined(), &[JsValue::from(1)], &mut context)
///     .unwrap();
/// f.call(&JsValue::undefined(), &[JsValue::from(4)], &mut context)
///     .unwrap();
/// assert_eq!(*x.borrow(), 5);
/// ```
pub trait UnsafeIntoJsFunction<Args, Ret>: private::IntoJsFunctionSealed<Args, Ret> {
    /// Converts the type into a JS function.
    ///
    /// # Safety
    /// This function is unsafe to ensure the callee knows the risks of using this trait.
    /// The implementing type must not contain any garbage collected objects.
    unsafe fn into_js_function_unsafe(self, context: &mut Context) -> NativeFunction;
}

/// The safe equivalent of the [`UnsafeIntoJsFunction`] trait.
/// This can only be used on closures that have the `Copy` trait.
///
/// Since this function is implemented for `Fn(...)` closures, we can use
/// it directly when defining a function:
/// ```
/// # use boa_engine::{Context, JsValue, NativeFunction};
/// # use boa_interop::IntoJsFunctionCopied;
/// # let mut context = Context::default();
/// let f = |a: i32, b: i32| a + b;
/// let f = f.into_js_function_copied(&mut context);
/// let result = f
///     .call(
///         &JsValue::undefined(),
///         &[JsValue::from(1), JsValue::from(2)],
///         &mut context,
///     )
///     .unwrap();
/// assert_eq!(result, JsValue::new(3));
/// ```
pub trait IntoJsFunctionCopied<Args, Ret>: private::IntoJsFunctionSealed<Args, Ret> + Copy {
    /// Converts the type into a JS function.
    fn into_js_function_copied(self, context: &mut Context) -> NativeFunction;
}

/// Create a Rust value from a JS argument. This trait is used to
/// convert arguments from JS to Rust types. It allows support
/// for optional arguments or rest arguments.
pub trait TryFromJsArgument<'a>: Sized {
    /// Try to convert a JS argument into a Rust value, returning the
    /// value and the rest of the arguments to be parsed.
    ///
    /// # Errors
    /// Any parsing errors that may occur during the conversion.
    fn try_from_js_argument(
        this: &'a JsValue,
        rest: &'a [JsValue],
        context: &mut Context,
    ) -> JsResult<(Self, &'a [JsValue])>;
}

impl<'a, T: TryFromJs> TryFromJsArgument<'a> for T {
    fn try_from_js_argument(
        _: &'a JsValue,
        rest: &'a [JsValue],
        context: &mut Context,
    ) -> JsResult<(Self, &'a [JsValue])> {
        match rest.split_first() {
            Some((first, rest)) => Ok((first.try_js_into(context)?, rest)),
            None => T::try_from_js(&JsValue::undefined(), context).map(|v| (v, rest)),
        }
    }
}

/// An argument that would be ignored in a JS function. This is equivalent of typing
/// `()` in Rust functions argument, but more explicit.
#[derive(Debug, Clone, Copy)]
pub struct Ignore;

impl<'a> TryFromJsArgument<'a> for Ignore {
    fn try_from_js_argument(
        _this: &'a JsValue,
        rest: &'a [JsValue],
        _: &mut Context,
    ) -> JsResult<(Self, &'a [JsValue])> {
        Ok((Ignore, &rest[1..]))
    }
}

/// An argument that when used in a JS function will empty the list
/// of JS arguments as `JsValue`s. This can be used for having the
/// rest of the arguments in a function. It should be the last
/// argument of your function, before the `Context` argument if any.
///
/// For example,
/// ```
/// # use boa_engine::{Context, JsValue};
/// # use boa_interop::{IntoJsFunctionCopied, JsRest};
/// # let mut context = Context::default();
/// let sums = (|args: JsRest, context: &mut Context| -> i32 {
///     args.iter()
///         .map(|i| i.try_js_into::<i32>(context).unwrap())
///         .sum::<i32>()
/// })
/// .into_js_function_copied(&mut context);
///
/// let result = sums
///     .call(
///         &JsValue::undefined(),
///         &[JsValue::from(1), JsValue::from(2), JsValue::from(3)],
///         &mut context,
///     )
///     .unwrap();
/// assert_eq!(result, JsValue::new(6));
/// ```
#[derive(Debug, Clone)]
pub struct JsRest<'a>(pub &'a [JsValue]);

#[allow(unused)]
impl<'a> JsRest<'a> {
    /// Consumes the `JsRest` and returns the inner list of `JsValue`.
    #[must_use]
    pub fn into_inner(self) -> &'a [JsValue] {
        self.0
    }

    /// Transforms the `JsRest` into a `Vec<JsValue>`.
    #[must_use]
    pub fn to_vec(self) -> Vec<JsValue> {
        self.0.to_vec()
    }

    /// Returns an iterator over the inner list of `JsValue`.
    pub fn iter(&self) -> impl Iterator<Item = &JsValue> {
        self.0.iter()
    }

    /// Returns the length of the inner list of `JsValue`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the inner list of `JsValue` is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> From<&'a [JsValue]> for JsRest<'a> {
    fn from(values: &'a [JsValue]) -> Self {
        Self(values)
    }
}

impl<'a> IntoIterator for JsRest<'a> {
    type Item = &'a JsValue;
    type IntoIter = std::slice::Iter<'a, JsValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().iter()
    }
}

/// An argument that when used in a JS function will capture all
/// the arguments that can be converted to `T`. The first argument
/// that cannot be converted to `T` will stop the conversion.
///
/// For example,
/// ```
/// # use boa_engine::{Context, JsValue};
/// # use boa_interop::{IntoJsFunctionCopied, JsAll};
/// # let mut context = Context::default();
/// let sums = (|args: JsAll<i32>, context: &mut Context| -> i32 { args.iter().sum() })
///     .into_js_function_copied(&mut context);
///
/// let result = sums
///     .call(
///         &JsValue::undefined(),
///         &[
///             JsValue::from(1),
///             JsValue::from(2),
///             JsValue::from(3),
///             JsValue::Boolean(true),
///             JsValue::from(4),
///         ],
///         &mut context,
///     )
///     .unwrap();
/// assert_eq!(result, JsValue::new(6));
/// ```
#[derive(Debug, Clone)]
pub struct JsAll<T: TryFromJs>(pub Vec<T>);

impl<T: TryFromJs> JsAll<T> {
    /// Consumes the `JsAll` and returns the inner list of `T`.
    #[must_use]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    /// Returns an iterator over the inner list of `T`.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the inner list of `T`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut()
    }

    /// Returns the length of the inner list of `T`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the inner list of `T` is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a, T: TryFromJs> TryFromJsArgument<'a> for JsAll<T> {
    fn try_from_js_argument(
        _this: &'a JsValue,
        mut rest: &'a [JsValue],
        context: &mut Context,
    ) -> JsResult<(Self, &'a [JsValue])> {
        let mut values = Vec::new();

        while !rest.is_empty() {
            match rest[0].try_js_into(context) {
                Ok(value) => {
                    values.push(value);
                    rest = &rest[1..];
                }
                Err(_) => break,
            }
        }
        Ok((JsAll(values), rest))
    }
}

/// Captures the `this` value in a JS function. Although this can be
/// specified multiple times as argument, it will always be filled
/// with clone of the same value.
#[derive(Debug, Clone)]
pub struct JsThis<T: TryFromJs>(pub T);

impl<'a, T: TryFromJs> TryFromJsArgument<'a> for JsThis<T> {
    fn try_from_js_argument(
        this: &'a JsValue,
        rest: &'a [JsValue],
        context: &mut Context,
    ) -> JsResult<(Self, &'a [JsValue])> {
        Ok((JsThis(this.try_js_into(context)?), rest))
    }
}

impl<T: TryFromJs> Deref for JsThis<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Captures a class instance from the `this` value in a JS function. The class
/// will be a non-mutable reference of Rust type `T`, if it is an instance of `T`.
///
/// To have more flexibility on the parsing of the `this` value, you can use the
/// [`JsThis`] capture instead.
#[derive(Debug, Clone)]
pub struct JsClass<T: NativeObject> {
    inner: boa_engine::JsObject<T>,
}

impl<T: NativeObject> JsClass<T> {
    /// Borrow a reference to the class instance of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently borrowed.
    ///
    /// This does not panic if the type is wrong, as the type is checked
    /// during the construction of the `JsClass` instance.
    #[must_use]
    pub fn borrow(&self) -> GcRef<'_, T> {
        GcRef::map(self.inner.borrow(), |obj| obj.data())
    }

    /// Borrow a mutable reference to the class instance of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[must_use]
    pub fn borrow_mut(&self) -> GcRefMut<'_, Object<T>, T> {
        GcRefMut::map(self.inner.borrow_mut(), |obj| obj.data_mut())
    }
}

impl<T: NativeObject + Clone> JsClass<T> {
    /// Clones the inner class instance.
    ///
    /// # Panics
    ///
    /// Panics if the inner object is currently borrowed mutably.
    #[must_use]
    pub fn clone_inner(&self) -> T {
        self.inner.borrow().data().clone()
    }
}

impl<'a, T: NativeObject + 'static> TryFromJsArgument<'a> for JsClass<T> {
    fn try_from_js_argument(
        this: &'a JsValue,
        rest: &'a [JsValue],
        _context: &mut Context,
    ) -> JsResult<(Self, &'a [JsValue])> {
        if let Some(object) = this.as_object() {
            if let Ok(inner) = object.clone().downcast::<T>() {
                return Ok((JsClass { inner }, rest));
            }
        }

        Err(JsNativeError::typ()
            .with_message("invalid this for class method")
            .into())
    }
}

/// Captures a [`ContextData`] data from the [`Context`] as a JS function argument,
/// based on its type.
///
/// The host defined type must implement [`Clone`], otherwise the borrow
/// checker would not be able to ensure the safety of the context while
/// making the function call. Because of this, it is recommended to use
/// types that are cheap to clone.
///
/// For example,
/// ```
/// # use boa_engine::{Context, Finalize, JsData, JsValue, Trace};
/// use boa_interop::{ContextData, IntoJsFunctionCopied};
///
/// #[derive(Clone, Debug, Finalize, JsData, Trace)]
/// struct CustomHostDefinedStruct {
///     #[unsafe_ignore_trace]
///     pub counter: usize,
/// }
/// let mut context = Context::default();
/// context.insert_data(CustomHostDefinedStruct { counter: 123 });
/// let f = (|ContextData(host): ContextData<CustomHostDefinedStruct>| host.counter + 1)
///     .into_js_function_copied(&mut context);
///
/// assert_eq!(
///     f.call(&JsValue::undefined(), &[], &mut context),
///     Ok(JsValue::new(124))
/// );
/// ```
#[derive(Debug, Clone)]
pub struct ContextData<T: Clone>(pub T);

impl<'a, T: NativeObject + Clone> TryFromJsArgument<'a> for ContextData<T> {
    fn try_from_js_argument(
        _this: &'a JsValue,
        rest: &'a [JsValue],
        context: &mut Context,
    ) -> JsResult<(Self, &'a [JsValue])> {
        match context.get_data::<T>() {
            Some(value) => Ok((ContextData(value.clone()), rest)),
            None => Err(JsNativeError::typ()
                .with_message("Context data not found")
                .into()),
        }
    }
}

// Implement `IntoJsFunction` for functions with a various list of
// arguments.
mod into_js_function_impls;

#[test]
#[allow(clippy::missing_panics_doc)]
fn into_js_module() {
    use boa_engine::{js_string, JsValue, Source};
    use boa_gc::{Gc, GcRefCell};
    use std::cell::RefCell;
    use std::rc::Rc;

    type ResultType = Gc<GcRefCell<JsValue>>;

    let loader = Rc::new(loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let foo_count = Rc::new(RefCell::new(0));
    let bar_count = Rc::new(RefCell::new(0));
    let dad_count = Rc::new(RefCell::new(0));

    context.insert_data(Gc::new(GcRefCell::new(JsValue::undefined())));

    let module = unsafe {
        vec![
            (
                js_string!("foo"),
                {
                    let counter = foo_count.clone();
                    move || {
                        *counter.borrow_mut() += 1;
                        let result = *counter.borrow();
                        result
                    }
                }
                .into_js_function_unsafe(&mut context),
            ),
            (
                js_string!("bar"),
                UnsafeIntoJsFunction::into_js_function_unsafe(
                    {
                        let counter = bar_count.clone();
                        move |i: i32| {
                            *counter.borrow_mut() += i;
                        }
                    },
                    &mut context,
                ),
            ),
            (
                js_string!("dad"),
                UnsafeIntoJsFunction::into_js_function_unsafe(
                    {
                        let counter = dad_count.clone();
                        move |args: JsRest<'_>, context: &mut Context| {
                            *counter.borrow_mut() += args
                                .into_iter()
                                .map(|i| i.try_js_into::<i32>(context).unwrap())
                                .sum::<i32>();
                        }
                    },
                    &mut context,
                ),
            ),
            (
                js_string!("send"),
                (move |value: JsValue, ContextData(result): ContextData<ResultType>| {
                    *result.borrow_mut() = value;
                })
                .into_js_function_copied(&mut context),
            ),
        ]
    }
    .into_js_module(&mut context);

    loader.register(js_string!("test"), module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            let result = test.foo();
            test.foo();
            for (let i = 1; i <= 5; i++) {
                test.bar(i);
            }
            for (let i = 1; i < 5; i++) {
                test.dad(1, 2, 3);
            }

            test.send(result);
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs();

    // Checking if the final promise didn't return an error.
    assert!(
        promise_result.state().as_fulfilled().is_some(),
        "module didn't execute successfully! Promise: {:?}",
        promise_result.state()
    );

    let result = context.get_data::<ResultType>().unwrap().borrow().clone();

    assert_eq!(*foo_count.borrow(), 2);
    assert_eq!(*bar_count.borrow(), 15);
    assert_eq!(*dad_count.borrow(), 24);
    assert_eq!(result.try_js_into(&mut context), Ok(1u32));
}

#[test]
fn can_throw_exception() {
    use boa_engine::{js_string, JsError, JsValue, Source};
    use std::rc::Rc;

    let loader = Rc::new(loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let module = vec![(
        js_string!("doTheThrow"),
        IntoJsFunctionCopied::into_js_function_copied(
            |message: JsValue| -> JsResult<()> { Err(JsError::from_opaque(message)) },
            &mut context,
        ),
    )]
    .into_js_module(&mut context);

    loader.register(js_string!("test"), module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            try {
                test.doTheThrow('javascript');
            } catch(e) {
                throw 'from ' + e;
            }
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs();

    // Checking if the final promise didn't return an error.
    assert_eq!(
        promise_result.state().as_rejected(),
        Some(&JsString::from("from javascript").into())
    );
}

#[test]
fn class() {
    use boa_engine::class::{Class, ClassBuilder};
    use boa_engine::property::Attribute;
    use boa_engine::{js_string, JsValue, Source};
    use boa_macros::{Finalize, JsData, Trace};
    use std::rc::Rc;

    #[derive(Debug, Trace, Finalize, JsData)]
    struct Test {
        value: i32,
    }

    impl Test {
        #[allow(clippy::needless_pass_by_value)]
        fn get_value(this: JsClass<Test>) -> i32 {
            this.borrow().value
        }

        #[allow(clippy::needless_pass_by_value)]
        fn set_value(this: JsClass<Test>, new_value: i32) {
            (*this.borrow_mut()).value = new_value;
        }
    }

    impl Class for Test {
        const NAME: &'static str = "Test";

        fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
            let get_value = Self::get_value.into_js_function_copied(class.context());
            class.method(js_string!("getValue"), 0, get_value);
            let set_value = Self::set_value.into_js_function_copied(class.context());
            class.method(js_string!("setValue"), 1, set_value);

            let get_value_getter = Self::get_value
                .into_js_function_copied(class.context())
                .to_js_function(class.context().realm());
            let set_value_setter = Self::set_value
                .into_js_function_copied(class.context())
                .to_js_function(class.context().realm());
            class.accessor(
                js_string!("value_get"),
                Some(get_value_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            );
            class.accessor(
                js_string!("value_set"),
                None,
                Some(set_value_setter),
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            );

            Ok(())
        }

        fn data_constructor(
            _new_target: &JsValue,
            _args: &[JsValue],
            _context: &mut Context,
        ) -> JsResult<Self> {
            Ok(Self { value: 123 })
        }
    }

    let loader = Rc::new(loaders::HashMapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    context.register_global_class::<Test>().unwrap();

    let source = Source::from_bytes(
        r"
            let t = new Test();
            if (t.getValue() != 123) {
                throw 'invalid value';
            }
            t.setValue(456);
            if (t.getValue() != 456) {
                throw 'invalid value 456';
            }
            if (t.value_get != 456) {
                throw 'invalid value 456';
            }
            t.value_set = 789;
            if (t.getValue() != 789) {
                throw 'invalid value 789';
            }
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs();

    // Checking if the final promise didn't return an error.
    assert!(
        promise_result.state().as_fulfilled().is_some(),
        "module didn't execute successfully! Promise: {:?}",
        promise_result.state()
    );
}
