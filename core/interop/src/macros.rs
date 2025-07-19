//! Module declaring interop macro rules.

/// Declare a JavaScript class, in a simpler way.
///
/// This can make declaration of JavaScript classes easier by using a hybrid
/// declarative approach. The class itself follows a closer syntax to JavaScript
/// while the method arguments/results and bodies are written in Rust.
///
/// This only declares the Boa interop parts of the class. The actual type must
/// be declared separately as a Rust type, along with necessary derives and
/// traits.
///
/// # Allowed declarations (in order):
///
/// ## Any number of JS fields
/// ```ignore
/// public <field_name>(<field_args>) -> <field_ty> { <field_body> }
/// ```
/// Declare public fields on the JavaScript prototype at construction. This is optional.
/// Those fields can be overwritten on the object itself.
///
/// ## Any number of properties
/// ```ignore
/// property <field_getset_name> [as "<js_field_name>"] {
///     get(<field_getset_get_args>) -> <field_getset_get_ty> { <field_getset_get_body> }
///     set(<field_getset_set_args>) [-> JsResult<()>] { <field_getset_set_body> }
/// }
/// ```
/// Declare a getter and/or a setter on a JavaScript class property. This is optional.
/// Both get and set are optional, but at least one must be present. The `set` method
/// must either return the unit type or a `JsResult<...>`. The value returned will be
/// ignored, only errors will be used.
///
/// Using the `as` keyword, you can set the name of the property in JavaScript that
/// would otherwise not be possible in Rust.
///
/// ## Required JavaScript Constructor
/// ```ignore
/// constructor(<ctor_args>) { <constructor_body> }
/// ```
/// Declares the JS constructor for the class. This is required, but could throw if creating
/// the object fails.
/// The body MUST return `JsResult<Self>`.
///
/// ## An optional init function
/// ```ignore
/// fn init(class: &mut ClassBuilder) -> JsResult<()> { <init_body> }
/// ```
/// Declare a block of code to add at the end of the implementation's init function.
///
/// ## Any number of methods
/// ```ignore
/// fn <method_name> [as <js_method_name>](<fn_args>) -> <result_type> { <method_body> }
/// ```
/// Declare methods on the class. This is optional.
///
/// Using the `as` keyword, you can set the name of the property in JavaScript that
/// would otherwise not be possible in Rust.
///
/// ----
/// # Example
///
/// Here's an example using the animal class declared in [`boa_engine::class`]:
/// ```
/// # use boa_engine::{JsString, JsData, js_string};
/// # use boa_gc::{Finalize, Trace};
/// use boa_engine::interop::{Ignore, JsClass};
/// use boa_interop::{js_class};
///
/// #[derive(Clone, Trace, Finalize, JsData)]
/// pub enum Animal {
///     Cat,
///     Dog,
///     Other,
/// }
///
/// js_class! {
///     // Implement [`Class`] trait for the `Animal` enum.
///     class Animal {
///         // This sets a field on the JavaScript object. The arguments to
///         // `init` are the arguments passed to the constructor. This
///         // function MUST return the value to be set on the field. If this
///         // returns a `JsResult`, it will be unwrapped and error out during
///         // construction of the object.
///         public age(_name: Ignore, age: i32) -> i32 {
///             age
///         }
///
///         // This is called when a new instance of the class is created in
///         // JavaScript, e.g. `new Animal("cat")`.
///         // This method is mandatory and MUST return `JsResult<Self>`.
///         constructor(name: String) {
///             match name.as_str() {
///                 "cat" => Ok(Animal::Cat),
///                 "dog" => Ok(Animal::Dog),
///                 _ => Ok(Animal::Other),
///             }
///         }
///
///         // Declare a function on the class itself.
///         // There is a current limitation using `self` in methods, so the
///         // instance must be accessed using an actual argument.
///         fn speak(this: JsClass<Animal>) -> JsString {
///             match *this.borrow() {
///                 Animal::Cat => js_string!("meow"),
///                 Animal::Dog => js_string!("woof"),
///                 Animal::Other => js_string!(r"¯\_(ツ)_/¯"),
///             }
///         }
///     }
/// }
///
/// fn main() {
///#    use boa_engine::{Context, JsString, Source, js_str};
///
///     let mut context = Context::default();
///
///     context.register_global_class::<Animal>().unwrap();
///
///     let result = context.eval(Source::from_bytes(r#"
///         let pet = new Animal("dog", 3);
///
///         `My pet is ${pet.age} years old. Right, buddy? - ${pet.speak()}!`
///     "#)).expect("Could not evaluate script");
///
///     assert_eq!(
///         result.as_string().unwrap(),
///         js_str!("My pet is 3 years old. Right, buddy? - woof!")
///     );
/// }
/// ```
#[macro_export]
macro_rules! js_class {
    (
    class $class_name: ident $(as $class_js_name: literal)? {
        $(
            $(#[$field_attr: meta])*
            public $field_name: ident
                ( $( $field_arg: ident: $field_arg_type: ty ),* $(,)? ) -> $field_ty: ty
                $field_body: block
        )*

        $(
            $(#[$field_prop_attr: meta])*
            property $field_prop_name: ident $(as $field_prop_js_name: literal)? {
                $(
                    $(#[$field_prop_get_attr: meta])*
                    $(fn)? get( $( $field_prop_get_arg: ident: $field_prop_get_arg_type: ty ),* $(,)? ) -> $field_prop_get_ty: ty
                    $field_prop_get_body: block
                )?

                $(
                    $(#[$field_prop_set_attr: meta])*
                    $(fn)? set( $( $field_prop_set_arg: ident: $field_prop_set_arg_type: ty ),* $(,)? )
                        $( -> $field_prop_set_ty: ty )?
                        $field_prop_set_body: block
                )?
            }
        )*


        $(#[$constructor_attr: meta])*
        constructor( $( $ctor_arg: ident: $ctor_arg_ty: ty ),* $(,)? )
            $constructor_body: block

        $(
            $(#[$init_attr: meta])*
            init($init_class_builder_name: ident : &mut ClassBuilder $(,)?) -> JsResult<()>
                $init_body: block
        )?

        $(
            $(#[$method_attr: meta])*
            fn $method_name: ident $( as $method_js_name: literal )?
                ( $( $fn_arg: ident: $fn_arg_type: ty ),* $(,)? )
                $(-> $result_type: ty)?
                $method_body: block
        )*
    }
    ) => {
        impl $crate::boa_engine::class::Class for $class_name {

            const NAME: &'static str = $crate::__js_class_name!($class_name, $($class_js_name)?);

            const LENGTH: usize = $crate::__count!( $( $ctor_arg )* );

            #[allow(clippy::items_after_statements)]
            fn init(class: &mut $crate::boa_engine::class::ClassBuilder<'_>) -> $crate::boa_engine::JsResult<()> {
                // Add properties.
                $(
                    // Declare a function so that the compiler prevents duplicated names.
                    #[allow(dead_code)]
                    fn $field_prop_name() {}

                    $crate::__get_set_decl!(
                        class,
                        $field_prop_name,
                        $($field_prop_js_name)?,
                        $(
                        @get( $( $field_prop_get_arg: $field_prop_get_arg_type ),* ) -> $field_prop_get_ty
                            $field_prop_get_body,
                        )?
                        $(
                        @set( $( $field_prop_set_arg: $field_prop_set_arg_type ),* )
                            $( -> $field_prop_set_ty )?
                            $field_prop_set_body
                        )?
                    );
                )*

                // Add all methods to the class.
                $(
                    fn $method_name ( $($fn_arg: $fn_arg_type),* ) -> $( $result_type )?
                        $method_body

                    let function = $crate::IntoJsFunctionCopied::into_js_function_copied(
                        $method_name,
                        class.context(),
                    );

                    let function_name = $crate::__js_class_name!($method_name, $($method_js_name)?);

                    class.method(
                        $crate::boa_engine::JsString::from(function_name),
                        $crate::__count!($( $fn_arg )*),
                        function,
                    );
                )*

                // Add the init body, if any.
                $({
                    let $init_class_builder_name = class;
                    let result: $crate::boa_engine::JsResult<()> = $init_body;

                    result?;
                })?

                Ok(())
            }

            #[allow(unused_variables)]
            fn data_constructor(
                new_target: &$crate::boa_engine::JsValue,
                args: &[$crate::boa_engine::JsValue],
                context: &mut $crate::boa_engine::Context,
            ) -> $crate::boa_engine::JsResult<$class_name> {
                let rest = args;
                $(
                    let ($ctor_arg, rest) : ($ctor_arg_ty, _) = $crate::boa_engine::interop::TryFromJsArgument::try_from_js_argument(new_target, rest, context)?;
                )*

                $constructor_body
            }

            fn object_constructor(
                instance: &$crate::boa_engine::JsObject,
                args: &[$crate::boa_engine::JsValue],
                context: &mut $crate::boa_engine::Context
            ) -> $crate::boa_engine::JsResult<()> {
                // Public JS fields first.
                $(
                    fn $field_name ( $($field_arg: $field_arg_type),* ) -> $field_ty
                        $field_body

                    let function = $crate::IntoJsFunctionCopied::into_js_function_copied(
                        $field_name,
                        context,
                    );

                    instance.set(
                        $crate::boa_engine::JsString::from(stringify!($field_name)),
                        function.call(&$crate::boa_engine::JsValue::undefined(), args, context)?,
                        false,
                        context
                    )?;
                )*

                Ok(())
            }

        }
    }
}

/// Internal macro to get the JavaScript class name.
#[macro_export]
macro_rules! __js_class_name {
    ($class_name: ident, $class_js_name: literal) => {
        $class_js_name
    };
    ($class_name: ident,) => {
        stringify!($class_name)
    };
}

/// Internal macro to get the JavaScript class length.
#[macro_export]
macro_rules! __count {
    () => (0);
    ($_: ident $($rest: ident)*) => {
        1 + $crate::__count!($($rest)*)
    };
}

/// Internal macro to declare a getter/setter name.
#[macro_export]
macro_rules! __get_set_decl {
    (
        $class: ident,
        $field_name: ident,
        $( $js_field_name: literal )?,
        @get( $( $get_arg: ident: $get_arg_type: ty ),* ) -> $get_ty: ty
            $get_body: block,
    ) => {
        let function = |$( $get_arg: $get_arg_type ),*| -> $get_ty { $get_body };
        let function_get =
            $crate::IntoJsFunctionCopied::into_js_function_copied(function, $class.context())
                .to_js_function($class.context().realm());

        let field_name = $crate::__js_class_name!($field_name, $($js_field_name)?);
        $class.accessor(
            $crate::boa_engine::JsString::from(field_name),
            Some(function_get),
            None,
            $crate::boa_engine::property::Attribute::CONFIGURABLE
                | $crate::boa_engine::property::Attribute::NON_ENUMERABLE,
        );
    };
    (
        $class: ident,
        $field_name: ident,
        $( $js_field_name: literal )?,
        @set( $( $set_arg: ident: $set_arg_type: ty ),* )
            $( -> $field_prop_set_ty: ty )?
            $set_body: block
    ) => {
        let function = |$( $set_arg: $set_arg_type ),*| $(-> $field_prop_set_ty)? { $set_body };
        let function_set =
            $crate::IntoJsFunctionCopied::into_js_function_copied(function, $class.context())
                .to_js_function($class.context().realm());

        let field_name = $crate::__js_class_name!($field_name, $($js_field_name)?);
        $class.accessor(
            $crate::boa_engine::JsString::from(field_name),
            None,
            Some(function_set),
            $crate::boa_engine::property::Attribute::CONFIGURABLE
                | $crate::boa_engine::property::Attribute::NON_ENUMERABLE,
        );
    };
    (
        $class: ident,
        $field_name: ident,
        $( $js_field_name: literal )?,
        @get( $( $get_arg: ident: $get_arg_type: ty ),* ) -> $get_ty: ty
            $get_body: block,
        @set( $( $set_arg: ident: $set_arg_type: ty ),* )
            $( -> $field_prop_set_ty: ty )?
            $set_body: block
    ) => {
        let function_get =
            $crate::IntoJsFunctionCopied::into_js_function_copied(
                |$( $get_arg: $get_arg_type ),*| -> $get_ty { $get_body },
                $class.context()
            ).to_js_function($class.context().realm());
        let function_set =
            $crate::IntoJsFunctionCopied::into_js_function_copied(
                |$( $set_arg: $set_arg_type ),*| $(-> $field_prop_set_ty)? { $set_body },
                $class.context()
            ).to_js_function($class.context().realm());

        let field_name = $crate::__js_class_name!($field_name, $($js_field_name)?);
        $class.accessor(
            $crate::boa_engine::JsString::from(field_name),
            Some(function_get),
            Some(function_set),
            $crate::boa_engine::property::Attribute::CONFIGURABLE
                | $crate::boa_engine::property::Attribute::NON_ENUMERABLE,
        );

    };
    (
        $class: ident,
        $field_name: ident,
        $( $js_field_name: literal )?,
    ) => {
        compile_error!("Property must have at least a getter or a setter");
    };
}

// We allow too many lines. This test is straightforward but has a lot of boilerplate
// still.
#[test]
#[allow(clippy::too_many_lines)]
fn js_class_test() {
    use crate::IntoJsFunctionCopied;
    use crate::{js_class, loaders};
    use boa_engine::interop::JsClass;
    use boa_engine::property::Attribute;
    use boa_engine::{Context, JsData, JsError, JsResult, Module, Source, js_string};
    use boa_gc::{Finalize, Trace};
    use std::rc::Rc;

    #[derive(Debug, Clone, Default, Trace, Finalize, JsData)]
    struct Test {
        f1: u32,
        f2: u32,
        f3: u32,
    }

    js_class! {
        class Test {
            public fp() -> u32 {
                10
            }

            property f1 {
                get(this: JsClass<Test>) -> u32 {
                    this.borrow().f1
                }
            }

            property f2 {
                set(this: JsClass<Test>, new_value: u32) {
                    this.borrow_mut().f1 = new_value;
                }
            }

            property f3 {
                get(this: JsClass<Test>) -> u32 {
                    this.borrow().f3
                }

                set(this: JsClass<Test>, new_value: u32) {
                    this.borrow_mut().f3 = new_value;
                }
            }

            property f4 {
                set() -> JsResult<()> {
                    Err(JsError::from_opaque(boa_engine::JsString::from("Cannot set f4.").into()))
                }
            }

            // Just to test the branch with both get, set and return value.
            property f5 {
                get() -> u8 { 1 }
                set() -> () { }
            }

            constructor() {
                Ok(Test::default())
            }

            init(class: &mut ClassBuilder) -> JsResult<()> {
                let get_value_getter = (|this: JsClass<Test>| {
                    this.borrow().f2
                })
                    .into_js_function_copied(class.context())
                    .to_js_function(class.context().realm());

                let set_value_setter = (|this: JsClass<Test>, new_value: u32| {
                    this.borrow_mut().f2 = new_value;
                })
                    .into_js_function_copied(class.context())
                    .to_js_function(class.context().realm());

                class.accessor(
                    js_string!("value2"),
                    Some(get_value_getter),
                    Some(set_value_setter),
                    Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
                );
                Ok(())
            }
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
            function assert_eq(name, actual, expected) {
                if (expected !== actual) {
                    throw `Assertion failed: ${name} - expected ${expected}, got ${actual}`;
                }
            }

            let t = new Test();
            assert_eq('fp', t.fp, 10);

            assert_eq('value2', t.value2, 0);
            t.value2 = 123;
            assert_eq('value2', t.value2, 123);

            // This should do nothing in Rust as this is a JS property and not a getter/setter.
            t.fp = 123;
            assert_eq('fp (set)', t.fp, 123);

            // Test separate getter/setter.
            assert_eq('f1', t.f1, 0);
            t.f2 = 1;
            assert_eq('f2', t.f1, 1);

            // Test same getter/setter.
            assert_eq('f3', t.f3, 0);
            t.f3 = 456;
            assert_eq('f3 (set)', t.f3, 456);

            // Test exception on setter.
            try {
                t.f4 = 123;
                throw 'Expected an exception';
            } catch (e) {
                if (e !== 'Cannot set f4.') {
                    throw e;
                }
            }
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    // Checking if the final promise didn't return an error.
    assert!(
        promise_result.state().as_fulfilled().is_some(),
        "module didn't execute successfully! Promise: {:?} ({:?})",
        promise_result.state(),
        promise_result
            .state()
            .as_rejected()
            .map(|r| r.to_json(&mut context))
    );
}
