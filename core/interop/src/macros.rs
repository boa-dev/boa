//! Module declaring interop macro rules.

/// Declare a JavaScript class, in a simpler way.
///
/// This can make declaration of JavaScript classes easier by using an hybrid
/// declarative approach. The class itself follows a closer syntax to JavaScript
/// while the method arguments/results and bodies are written in Rust.
///
/// This only declares the Boa interop parts of the class. The actual type must
/// be declared separately as a Rust type, along with necessary derives and
/// traits.
///
/// Here's an example using the animal class declared in [`boa_engine::class`]:
/// # Example
/// ```
/// # use boa_engine::{
/// #    Context, JsResult, JsValue, JsString,
/// #    Source, js_str, js_string,
/// #    JsData,
/// # };
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

        $(#[$constructor_attr: meta])*
        constructor( $( $ctor_arg: ident: $ctor_arg_ty: ty ),* )
            $constructor_body: block

        $(
            $(#[$method_attr: meta])*
            fn $method_name: ident
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
                // Add all methods to the class.
                $(
                    fn $method_name ( $($fn_arg: $fn_arg_type),* ) -> $( $result_type )?
                        $method_body

                    let function = $crate::IntoJsFunctionCopied::into_js_function_copied(
                        $method_name,
                        class.context(),
                    );

                    class.method(
                        $crate::boa_engine::JsString::from(stringify!($method_name)),
                        $crate::__count!($( $fn_arg )*),
                        function,
                    );
                )*

                Ok(())
            }

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
                $(
                    fn $field_name ( $($field_arg: $field_arg_type),* ) -> $field_ty
                        $field_body

                    let function = $crate::IntoJsFunctionCopied::into_js_function_copied(
                        $field_name,
                        context,
                    );

                    instance.set(
                        $crate::boa_engine::JsString::from(stringify!($field_name)),
                        function.call(&JsValue::undefined(), args, context)?,
                        false,
                        context
                    );
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
