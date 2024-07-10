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
///   ```ignore
///   public <field_name>(<field_args>) -> <field_ty> { <field_body> }
///   ```
/// Declare public fields on the JavaScript prototype at construction. This is optional.
/// Those fields can be overwritten on the object itself.
///
/// ## Any number of getters
/// ```ignore
/// getter <field_get_name>(<field_get_args>) -> <field_get_ty> { <field_get_body> }
/// ```
/// Declare public getters on the JavaScript class property. This is optional.
/// To declare both a getter and a setter, use getset instead.
///
/// ## Any number of setters
/// ```ignore
/// setter <field_set_name>(<field_set_args>) { <field_set_body> }
/// ```
/// Declare public setters on the JavaScript class property. This is optional.
/// To declare both a getter and a setter, use getset instead.
///
/// ## Any number of getters and setters for the same property name
/// ```ignore
/// getset <field_getset_name> {
///     get(<field_getset_get_args>) -> <field_getset_get_ty> { <field_getset_get_body> }
///     set(<field_getset_set_args>) { <field_getset_set_body> }
/// }
/// ```
/// Declare both a getter and a setter on the JavaScript class property. This is optional.
/// To declare only a getter or a setter, use getter or setter instead.
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
/// fn <method_name>(<fn_args>) -> <result_type> { <method_body> }
/// ```
/// Declare methods on the class. This is optional.
///
/// ----
/// # Example
///
/// Here's an example using the animal class declared in [`boa_engine::class`]:
/// ```
/// # use boa_engine::{JsString, JsData, js_string};
/// # use boa_gc::{Finalize, Trace};
/// use boa_interop::{js_class, Ignore, JsClass};
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
///         &js_str!("My pet is 3 years old. Right, buddy? - woof!")
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
                ( $( $field_arg: ident: $field_arg_type: ty ),* ) -> $field_ty: ty
                $field_body: block
        )*

        $(
            $(#[$field_get_attr: meta])*
            getter $field_get_name: ident
                ( $( $field_get_arg: ident: $field_get_arg_type: ty ),* ) -> $field_get_ty: ty
                $field_get_body: block
        )*

        $(
            $(#[$field_set_attr: meta])*
            setter $field_set_name: ident
                ( $( $field_set_arg: ident: $field_set_arg_type: ty ),* )
                $field_set_body: block
        )*

        $(
            $(#[$field_getset_attr: meta])*
            getset $field_getset_name: ident {
                fn get ( $( $field_getset_get_arg: ident: $field_getset_get_arg_type: ty ),* ) -> $field_getset_get_ty: ty
                $field_getset_get_body: block
                fn set ( $( $field_getset_set_arg: ident: $field_getset_set_arg_type: ty ),* )
                $field_getset_set_body: block
            }
        )*

        $(#[$constructor_attr: meta])*
        constructor( $( $ctor_arg: ident: $ctor_arg_ty: ty ),* )
            $constructor_body: block

        $(
            $(#[$init_attr: meta])*
            init($init_class_builder_name: ident : &mut ClassBuilder) -> JsResult<()>
                $init_body: block
        )?

        $(
            $(#[$method_attr: meta])*
            fn $method_name: ident $( as $method_js_name: literal )?
                ( $( $fn_arg: ident: $fn_arg_type: ty ),* )
                $(-> $result_type: ty)?
                $method_body: block
        )*
    }
    ) => {
        impl $crate::boa_engine::class::Class for $class_name {

            const NAME: &'static str = $crate::__js_class_name!($class_name, $($class_js_name)?);

            const LENGTH: usize = $crate::__count!( $( $ctor_arg )* );

            fn init(class: &mut $crate::boa_engine::class::ClassBuilder<'_>) -> $crate::boa_engine::JsResult<()> {
                // Add getters.
                $(
                    fn $field_get_name ( $($field_get_arg: $field_get_arg_type),* ) -> $field_get_ty
                        $field_get_body

                    let function = $crate::IntoJsFunctionCopied::into_js_function_copied(
                        $field_get_name,
                        class.context(),
                    ).to_js_function(class.context().realm());

                    class.accessor(
                        $crate::boa_engine::JsString::from(stringify!($field_get_name)),
                        Some(function),
                        None,
                        $crate::boa_engine::property::Attribute::CONFIGURABLE
                        | $crate::boa_engine::property::Attribute::NON_ENUMERABLE,
                    );
                )*

                // Add setters.
                $(
                    fn $field_set_name ( $($field_set_arg: $field_set_arg_type),* ) -> ()
                        $field_set_body

                    let function = $crate::IntoJsFunctionCopied::into_js_function_copied(
                        $field_set_name,
                        class.context(),
                    );
                    let function = $crate::boa_engine::object::FunctionObjectBuilder::new(class.context().realm(), function)
                        .name($crate::boa_engine::JsString::from(concat!("set ", stringify!($field_set_name))))
                        .length(1)
                        .build();

                    class.accessor(
                        $crate::boa_engine::JsString::from(stringify!($field_set_name)),
                        None,
                        Some(function),
                        $crate::boa_engine::property::Attribute::CONFIGURABLE
                        | $crate::boa_engine::property::Attribute::NON_ENUMERABLE,
                    );
                )*

                // Add getters+setters.
                // Use the field name for the getter, to prevent duplicate names.
                // Rust does not allow two functions with the same name in the same scope.
                $(
                    fn $field_getset_name( $($field_getset_get_arg: $field_getset_get_arg_type),* ) -> $field_getset_get_ty
                        $field_getset_get_body

                    let function_g = $crate::IntoJsFunctionCopied::into_js_function_copied(
                            $field_getset_name,
                            class.context(),
                        ).to_js_function(class.context().realm());
                    let function_s = $crate::IntoJsFunctionCopied::into_js_function_copied(
                        |$($field_getset_set_arg: $field_getset_set_arg_type),*|
                        $field_getset_set_body,
                        class.context(),
                    ).to_js_function(class.context().realm());

                    class.accessor(
                        $crate::boa_engine::JsString::from(stringify!($field_getset_name)),
                        Some(function_g),
                        Some(function_s),
                        $crate::boa_engine::property::Attribute::CONFIGURABLE
                        | $crate::boa_engine::property::Attribute::NON_ENUMERABLE,
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
                    let ($ctor_arg, rest) : ($ctor_arg_ty, _) = $crate::TryFromJsArgument::try_from_js_argument(new_target, rest, context)?;
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

#[test]
fn js_class_test() {
    use crate::IntoJsFunctionCopied;
    use crate::{js_class, loaders, JsClass};
    use boa_engine::property::Attribute;
    use boa_engine::{js_string, Context, JsData, Module, Source};
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

            getter f1(this: JsClass<Test>) -> u32 {
                this.borrow().f1
            }

            setter f2(this: JsClass<Test>, new_value: u32) {
                this.borrow_mut().f1 = new_value;
            }

            getset f3 {
                fn get(this: JsClass<Test>) -> u32 {
                    this.borrow().f3
                }

                fn set(this: JsClass<Test>, new_value: u32) {
                    this.borrow_mut().f3 = new_value;
                }
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
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs();

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
