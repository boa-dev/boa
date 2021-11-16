use crate::{
    object::{
        internal_methods::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS},
        JsObject,
    },
    Context, JsResult, JsValue,
};

#[cfg(not(feature = "vm"))]
use crate::{
    builtins::function::{
        arguments::Arguments, Captures, ClosureFunctionSignature, Function, NativeFunctionSignature,
    },
    context::StandardObjects,
    environment::{
        function_environment_record::{BindingStatus, FunctionEnvironmentRecord},
        lexical_environment::Environment,
    },
    exec::{Executable, InterpreterState},
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    syntax::ast::node::RcStatementList,
};

/// Definitions of the internal object methods for function objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
pub(crate) static FUNCTION_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __call__: Some(function_call),
    __construct__: None,
    ..ORDINARY_INTERNAL_METHODS
};

pub(crate) static CONSTRUCTOR_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __call__: Some(function_call),
    __construct__: Some(function_construct),
    ..ORDINARY_INTERNAL_METHODS
};

/// Call this object.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-prepareforordinarycall>
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
#[track_caller]
#[inline]
fn function_call(
    obj: &JsObject,
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    #[cfg(not(feature = "vm"))]
    return call_construct(obj, this, args, context, false);
    #[cfg(feature = "vm")]
    return obj.call_internal(this, args, context);
}

/// Construct an instance of this object with the specified arguments.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
#[track_caller]
#[inline]
fn function_construct(
    obj: &JsObject,
    args: &[JsValue],
    new_target: &JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    #[cfg(not(feature = "vm"))]
    return call_construct(obj, new_target, args, context, true);
    #[cfg(feature = "vm")]
    return obj.construct_internal(args, new_target, context);
}

/// Internal implementation of [`call`](#method.call) and [`construct`](#method.construct).
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
///
/// <https://tc39.es/ecma262/#sec-prepareforordinarycall>
/// <https://tc39.es/ecma262/#sec-ordinarycallbindthis>
/// <https://tc39.es/ecma262/#sec-runtime-semantics-evaluatebody>
/// <https://tc39.es/ecma262/#sec-ordinarycallevaluatebody>
#[track_caller]
#[cfg(not(feature = "vm"))]
pub(super) fn call_construct(
    obj: &JsObject,
    this_target: &JsValue,
    args: &[JsValue],
    context: &mut Context,
    construct: bool,
) -> JsResult<JsValue> {
    /// The body of a JavaScript function.
    ///
    /// This is needed for the call method since we cannot mutate the function itself since we
    /// already borrow it so we get the function body clone it then drop the borrow and run the body
    enum FunctionBody {
        BuiltInFunction(NativeFunctionSignature),
        BuiltInConstructor(NativeFunctionSignature),
        Closure {
            function: Box<dyn ClosureFunctionSignature>,
            captures: Captures,
        },
        Ordinary(RcStatementList),
    }

    let this_function_object = obj.clone();
    let mut has_parameter_expressions = false;

    let body = if let Some(function) = obj.borrow().as_function() {
        if construct && !function.is_constructor() {
            let name = obj.get("name", context)?.display().to_string();
            return context.throw_type_error(format!("{} is not a constructor", name));
        } else {
            match function {
                Function::Native {
                    function,
                    constructor,
                } => {
                    if *constructor || construct {
                        FunctionBody::BuiltInConstructor(*function)
                    } else {
                        FunctionBody::BuiltInFunction(*function)
                    }
                }
                Function::Closure {
                    function, captures, ..
                } => FunctionBody::Closure {
                    function: function.clone(),
                    captures: captures.clone(),
                },
                Function::Ordinary {
                    constructor: _,
                    this_mode,
                    body,
                    params,
                    environment,
                } => {
                    let this = if construct {
                        // If the prototype of the constructor is not an object, then use the default object
                        // prototype as prototype for the new object
                        // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
                        // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
                        let proto = get_prototype_from_constructor(
                            this_target,
                            StandardObjects::object_object,
                            context,
                        )?;
                        JsObject::from_proto_and_data(Some(proto), ObjectData::ordinary()).into()
                    } else {
                        this_target.clone()
                    };

                    // Create a new Function environment whose parent is set to the scope of the function declaration (obj.environment)
                    // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                    let local_env = FunctionEnvironmentRecord::new(
                        this_function_object.clone(),
                        if construct || !this_mode.is_lexical() {
                            Some(this.clone())
                        } else {
                            None
                        },
                        Some(environment.clone()),
                        // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                        if this_mode.is_lexical() {
                            BindingStatus::Lexical
                        } else {
                            BindingStatus::Uninitialized
                        },
                        JsValue::undefined(),
                        context,
                    )?;

                    let mut arguments_in_parameter_names = false;
                    let mut is_simple_parameter_list = true;

                    for param in params.iter() {
                        has_parameter_expressions =
                            has_parameter_expressions || param.init().is_some();

                        for param_name in param.names() {
                            arguments_in_parameter_names =
                                arguments_in_parameter_names || param_name == "arguments";
                        }

                        is_simple_parameter_list = is_simple_parameter_list
                            && !param.is_rest_param()
                            && param.is_identifier()
                            && param.init().is_none()
                    }

                    // Turn local_env into Environment so it can be cloned
                    let local_env: Environment = local_env.into();

                    // An arguments object is added when all of the following conditions are met
                    // - If not in an arrow function (10.2.11.16)
                    // - If the parameter list does not contain `arguments` (10.2.11.17)
                    // - If there are default parameters or if lexical names and function names do not contain `arguments` (10.2.11.18)
                    //
                    // https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
                    if !this_mode.is_lexical()
                        && !arguments_in_parameter_names
                        && (has_parameter_expressions
                            || (!body.lexically_declared_names().contains("arguments")
                                && !body.function_declared_names().contains("arguments")))
                    {
                        // Add arguments object
                        let arguments_obj =
                            if context.strict() || body.strict() || !is_simple_parameter_list {
                                Arguments::create_unmapped_arguments_object(args, context)
                            } else {
                                Arguments::create_mapped_arguments_object(
                                    obj, params, args, &local_env, context,
                                )
                            };
                        local_env.create_mutable_binding("arguments", false, true, context)?;
                        local_env.initialize_binding("arguments", arguments_obj.into(), context)?;
                    }

                    // Push the environment first so that it will be used by default parameters
                    context.push_environment(local_env.clone());

                    // Add argument bindings to the function environment
                    for (i, param) in params.iter().enumerate() {
                        // Rest Parameters
                        if param.is_rest_param() {
                            Function::add_rest_param(param, i, args, context, &local_env);
                            break;
                        }

                        let value = match args.get(i).cloned() {
                            None | Some(JsValue::Undefined) => param
                                .init()
                                .map(|init| init.run(context).ok())
                                .flatten()
                                .unwrap_or_default(),
                            Some(value) => value,
                        };

                        Function::add_arguments_to_environment(param, value, &local_env, context);
                    }

                    if has_parameter_expressions {
                        // Create a second environment when default parameter expressions are used
                        // This prevents variables declared in the function body from being
                        // used in default parameter initializers.
                        // https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
                        let second_env = FunctionEnvironmentRecord::new(
                            this_function_object,
                            if construct || !this_mode.is_lexical() {
                                Some(this)
                            } else {
                                None
                            },
                            Some(local_env),
                            // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                            if this_mode.is_lexical() {
                                BindingStatus::Lexical
                            } else {
                                BindingStatus::Uninitialized
                            },
                            JsValue::undefined(),
                            context,
                        )?;
                        context.push_environment(second_env);
                    }

                    FunctionBody::Ordinary(body.clone())
                }
                #[cfg(feature = "vm")]
                Function::VmOrdinary { .. } => {
                    todo!("vm call")
                }
            }
        }
    } else {
        return context.throw_type_error("not a function");
    };

    match body {
        FunctionBody::BuiltInConstructor(function) if construct => {
            function(this_target, args, context)
        }
        FunctionBody::BuiltInConstructor(function) => {
            function(&JsValue::undefined(), args, context)
        }
        FunctionBody::BuiltInFunction(function) => function(this_target, args, context),
        FunctionBody::Closure { function, captures } => {
            (function)(this_target, args, captures, context)
        }
        FunctionBody::Ordinary(body) => {
            let result = body.run(context);
            let this = context.get_this_binding();

            if has_parameter_expressions {
                context.pop_environment();
            }
            context.pop_environment();

            if construct {
                // https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget
                // 12. If result.[[Type]] is return, then
                if context.executor().get_current_state() == &InterpreterState::Return {
                    // a. If Type(result.[[Value]]) is Object, return NormalCompletion(result.[[Value]]).
                    if let Ok(v) = &result {
                        if v.is_object() {
                            return result;
                        }
                    }
                }

                // 13. Else, ReturnIfAbrupt(result).
                result?;

                // 14. Return ? constructorEnv.GetThisBinding().
                this
            } else if context.executor().get_current_state() == &InterpreterState::Return {
                result
            } else {
                result?;
                Ok(JsValue::undefined())
            }
        }
    }
}
