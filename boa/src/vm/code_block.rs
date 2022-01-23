//! CodeBlock
//!
//! This module is for the CodeBlock which implements a function representation in the VM

use crate::{
    builtins::function::{
        arguments::Arguments, Captures, ClosureFunctionSignature, Function,
        NativeFunctionSignature, ThisMode,
    },
    context::StandardObjects,
    environment::{
        function_environment_record::{BindingStatus, FunctionEnvironmentRecord},
        lexical_environment::Environment,
    },
    gc::{Finalize, Gc, Trace},
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::PropertyDescriptor,
    syntax::ast::node::FormalParameter,
    vm::{call_frame::FinallyReturn, CallFrame, Opcode},
    Context, JsResult, JsString, JsValue,
};
use boa_interner::{Interner, Sym, ToInternedString};
use std::{convert::TryInto, mem::size_of};

/// This represents whether a value can be read from [`CodeBlock`] code.
///
/// # Safety
///
/// This trait is safe to implement as long as the type doesn't implement `Drop`.
/// At some point, if [negative impls][negative_impls] are stabilized, we might be able to remove
/// the unsafe bound.
///
/// [negative_impls]: https://doc.rust-lang.org/beta/unstable-book/language-features/negative-impls.html
pub(crate) unsafe trait Readable {}

unsafe impl Readable for u8 {}
unsafe impl Readable for i8 {}
unsafe impl Readable for u16 {}
unsafe impl Readable for i16 {}
unsafe impl Readable for u32 {}
unsafe impl Readable for i32 {}
unsafe impl Readable for u64 {}
unsafe impl Readable for i64 {}
unsafe impl Readable for f32 {}
unsafe impl Readable for f64 {}

/// The internal representation of a JavaScript function.
///
/// A CodeBlock is generated for each function compiled by the [ByteCompiler](crate::bytecompiler::ByteCompiler).
/// It stores the bytecode and the other attributes of the function.
#[derive(Clone, Debug, Trace, Finalize)]
pub struct CodeBlock {
    /// Name of this function
    pub(crate) name: Sym,

    /// The number of arguments expected.
    pub(crate) length: u32,

    /// Is this function in strict mode.
    pub(crate) strict: bool,

    /// Is this function a constructor.
    pub(crate) constructor: bool,

    /// [[ThisMode]]
    pub(crate) this_mode: ThisMode,

    /// Parameters passed to this function.
    pub(crate) params: Box<[FormalParameter]>,

    /// Bytecode
    pub(crate) code: Vec<u8>,

    /// Literals
    pub(crate) literals: Vec<JsValue>,

    /// Variables names
    pub(crate) variables: Vec<JsString>,

    /// Functions inside this function
    pub(crate) functions: Vec<Gc<CodeBlock>>,

    /// Indicates if the codeblock contains a lexical name `arguments`
    pub(crate) lexical_name_argument: bool,
}

impl CodeBlock {
    /// Constructs a new `CodeBlock`.
    pub fn new(name: Sym, length: u32, strict: bool, constructor: bool) -> Self {
        Self {
            code: Vec::new(),
            literals: Vec::new(),
            variables: Vec::new(),
            functions: Vec::new(),
            name,
            length,
            strict,
            constructor,
            this_mode: ThisMode::Global,
            params: Vec::new().into_boxed_slice(),
            lexical_name_argument: false,
        }
    }

    /// Read type T from code.
    ///
    /// # Safety
    ///
    /// Does not check if read happens out-of-bounds.
    pub(crate) unsafe fn read_unchecked<T>(&self, offset: usize) -> T
    where
        T: Readable,
    {
        // This has to be an unaligned read because we can't guarantee that
        // the types are aligned.
        self.code.as_ptr().add(offset).cast::<T>().read_unaligned()
    }

    /// Read type T from code.
    #[track_caller]
    pub(crate) fn read<T>(&self, offset: usize) -> T
    where
        T: Readable,
    {
        assert!(offset + size_of::<T>() - 1 < self.code.len());

        // Safety: We checked that it is not an out-of-bounds read,
        // so this is safe.
        unsafe { self.read_unchecked(offset) }
    }

    /// Get the operands after the `Opcode` pointed to by `pc` as a `String`.
    /// Modifies the `pc` to point to the next instruction.
    ///
    /// Returns an empty `String` if no operands are present.
    pub(crate) fn instruction_operands(&self, pc: &mut usize) -> String {
        let opcode: Opcode = self.code[*pc].try_into().unwrap();
        *pc += size_of::<Opcode>();
        match opcode {
            Opcode::PushInt8 => {
                let result = self.read::<i8>(*pc).to_string();
                *pc += size_of::<i8>();
                result
            }
            Opcode::PushInt16 => {
                let result = self.read::<i16>(*pc).to_string();
                *pc += size_of::<i16>();
                result
            }
            Opcode::PushInt32 => {
                let result = self.read::<i32>(*pc).to_string();
                *pc += size_of::<i32>();
                result
            }
            Opcode::PushRational => {
                let operand = self.read::<f64>(*pc);
                *pc += size_of::<f64>();
                ryu_js::Buffer::new().format(operand).to_string()
            }
            Opcode::PushLiteral
            | Opcode::Jump
            | Opcode::JumpIfFalse
            | Opcode::JumpIfNotUndefined
            | Opcode::CatchStart
            | Opcode::FinallySetJump
            | Opcode::Case
            | Opcode::Default
            | Opcode::LogicalAnd
            | Opcode::LogicalOr
            | Opcode::Coalesce
            | Opcode::Call
            | Opcode::CallWithRest
            | Opcode::New
            | Opcode::NewWithRest
            | Opcode::ForInLoopInitIterator
            | Opcode::ForInLoopNext => {
                let result = self.read::<u32>(*pc).to_string();
                *pc += size_of::<u32>();
                result
            }
            Opcode::TryStart => {
                let operand1 = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                let operand2 = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!("{}, {}", operand1, operand2)
            }
            Opcode::GetFunction => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!(
                    "{:04}: '{:?}' (length: {})",
                    operand,
                    self.functions[operand as usize].name,
                    self.functions[operand as usize].length
                )
            }
            Opcode::DefInitArg
            | Opcode::DefVar
            | Opcode::DefInitVar
            | Opcode::DefLet
            | Opcode::DefInitLet
            | Opcode::DefInitConst
            | Opcode::GetName
            | Opcode::GetNameOrUndefined
            | Opcode::SetName
            | Opcode::GetPropertyByName
            | Opcode::SetPropertyByName
            | Opcode::DefineOwnPropertyByName
            | Opcode::SetPropertyGetterByName
            | Opcode::SetPropertySetterByName
            | Opcode::DeletePropertyByName
            | Opcode::ConcatToString
            | Opcode::CopyDataProperties => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!("{:04}: '{:?}'", operand, self.variables[operand as usize])
            }
            Opcode::Pop
            | Opcode::Dup
            | Opcode::Swap
            | Opcode::PushZero
            | Opcode::PushOne
            | Opcode::PushNaN
            | Opcode::PushPositiveInfinity
            | Opcode::PushNegativeInfinity
            | Opcode::PushNull
            | Opcode::PushTrue
            | Opcode::PushFalse
            | Opcode::PushUndefined
            | Opcode::PushEmptyObject
            | Opcode::Add
            | Opcode::Sub
            | Opcode::Div
            | Opcode::Mul
            | Opcode::Mod
            | Opcode::Pow
            | Opcode::ShiftRight
            | Opcode::ShiftLeft
            | Opcode::UnsignedShiftRight
            | Opcode::BitOr
            | Opcode::BitAnd
            | Opcode::BitXor
            | Opcode::BitNot
            | Opcode::In
            | Opcode::Eq
            | Opcode::StrictEq
            | Opcode::NotEq
            | Opcode::StrictNotEq
            | Opcode::GreaterThan
            | Opcode::GreaterThanOrEq
            | Opcode::LessThan
            | Opcode::LessThanOrEq
            | Opcode::InstanceOf
            | Opcode::TypeOf
            | Opcode::Void
            | Opcode::LogicalNot
            | Opcode::Pos
            | Opcode::Neg
            | Opcode::Inc
            | Opcode::Dec
            | Opcode::GetPropertyByValue
            | Opcode::SetPropertyByValue
            | Opcode::DefineOwnPropertyByValue
            | Opcode::SetPropertyGetterByValue
            | Opcode::SetPropertySetterByValue
            | Opcode::DeletePropertyByValue
            | Opcode::ToBoolean
            | Opcode::Throw
            | Opcode::TryEnd
            | Opcode::CatchEnd
            | Opcode::CatchEnd2
            | Opcode::FinallyStart
            | Opcode::FinallyEnd
            | Opcode::This
            | Opcode::Return
            | Opcode::PushDeclarativeEnvironment
            | Opcode::PushFunctionEnvironment
            | Opcode::PopEnvironment
            | Opcode::InitIterator
            | Opcode::IteratorNext
            | Opcode::IteratorNextFull
            | Opcode::IteratorClose
            | Opcode::IteratorToArray
            | Opcode::RequireObjectCoercible
            | Opcode::ValueNotNullOrUndefined
            | Opcode::RestParameterInit
            | Opcode::RestParameterPop
            | Opcode::PushValueToArray
            | Opcode::PushIteratorToArray
            | Opcode::PushNewArray
            | Opcode::PopOnReturnAdd
            | Opcode::PopOnReturnSub
            | Opcode::Nop => String::new(),
        }
    }
}

impl ToInternedString for CodeBlock {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let name = interner.resolve(self.name).expect("string disappeared");
        let mut f = if self.name == Sym::MAIN {
            String::new()
        } else {
            "\n".to_owned()
        };

        f.push_str(&format!(
            "{:-^width$}\n    Location  Count   Opcode                     Operands\n\n",
            format!("Compiled Output: '{}'", name),
            width = 70
        ));

        let mut pc = 0;
        let mut count = 0;
        while pc < self.code.len() {
            let opcode: Opcode = self.code[pc].try_into().unwrap();
            let operands = self.instruction_operands(&mut pc);
            f.push_str(&format!(
                "    {:06}    {:04}    {:<27}\n{}",
                pc,
                count,
                opcode.as_str(),
                operands
            ));
            count += 1;
        }

        f.push_str("\nLiterals:\n");

        if !self.literals.is_empty() {
            for (i, value) in self.literals.iter().enumerate() {
                f.push_str(&format!(
                    "    {:04}: <{}> {}\n",
                    i,
                    value.type_of(),
                    value.display()
                ));
            }
        } else {
            f.push_str("    <empty>");
        }

        f.push_str("\nNames:\n");
        if !self.variables.is_empty() {
            for (i, value) in self.variables.iter().enumerate() {
                f.push_str(&format!("    {:04}: {}\n", i, value));
            }
        } else {
            f.push_str("    <empty>");
        }

        f.push_str("\nFunctions:\n");
        if !self.functions.is_empty() {
            for (i, code) in self.functions.iter().enumerate() {
                f.push_str(&format!(
                    "    {:04}: name: '{}' (length: {})\n",
                    i,
                    interner.resolve(code.name).expect("string disappeared"),
                    code.length
                ));
            }
        } else {
            f.push_str("    <empty>");
        }

        f
    }
}

#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct JsVmFunction {}

impl JsVmFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(code: Gc<CodeBlock>, environment: Environment, context: &mut Context) -> JsObject {
        let function_prototype = context.standard_objects().function_object().prototype();

        let prototype = context.construct_object();

        let name_property = PropertyDescriptor::builder()
            .value(
                context
                    .interner()
                    .resolve(code.name)
                    .expect("string disappeared"),
            )
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();

        let length_property = PropertyDescriptor::builder()
            .value(code.length)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();

        let function = Function::VmOrdinary { code, environment };

        let constructor =
            JsObject::from_proto_and_data(function_prototype, ObjectData::function(function));

        let constructor_property = PropertyDescriptor::builder()
            .value(constructor.clone())
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build();

        prototype
            .define_property_or_throw("constructor", constructor_property, context)
            .unwrap();

        let prototype_property = PropertyDescriptor::builder()
            .value(prototype)
            .writable(true)
            .enumerable(false)
            .configurable(false)
            .build();

        constructor
            .define_property_or_throw("prototype", prototype_property, context)
            .unwrap();
        constructor
            .define_property_or_throw("name", name_property, context)
            .unwrap();
        constructor
            .define_property_or_throw("length", length_property, context)
            .unwrap();

        constructor
    }
}

pub(crate) enum FunctionBody {
    Ordinary {
        code: Gc<CodeBlock>,
        environment: Environment,
    },
    Native {
        function: NativeFunctionSignature,
    },
    Closure {
        function: Box<dyn ClosureFunctionSignature>,
        captures: Captures,
    },
}

// TODO: this should be modified to not take `exit_on_return` and then moved to `internal_methods`
impl JsObject {
    pub(crate) fn call_internal(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_function_object = self.clone();
        // let mut has_parameter_expressions = false;

        if !self.is_callable() {
            return context.throw_type_error("not a callable function");
        }

        let mut construct = false;

        let body = {
            let object = self.borrow();
            let function = object.as_function().unwrap();

            match function {
                Function::Native {
                    function,
                    constructor,
                } => {
                    if *constructor {
                        construct = true;
                    }

                    FunctionBody::Native {
                        function: *function,
                    }
                }
                Function::Closure {
                    function, captures, ..
                } => FunctionBody::Closure {
                    function: function.clone(),
                    captures: captures.clone(),
                },
                Function::VmOrdinary { code, environment } => FunctionBody::Ordinary {
                    code: code.clone(),
                    environment: environment.clone(),
                },
            }
        };

        match body {
            FunctionBody::Native { function } if construct => {
                function(&JsValue::undefined(), args, context)
            }
            FunctionBody::Native { function } => function(this, args, context),
            FunctionBody::Closure { function, captures } => {
                (function)(this, args, captures, context)
            }
            FunctionBody::Ordinary { code, environment } => {
                let lexical_this_mode = code.this_mode == ThisMode::Lexical;

                // Create a new Function environment whose parent is set to the scope of the function declaration (self.environment)
                // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                let local_env = FunctionEnvironmentRecord::new(
                    this_function_object.clone(),
                    if !lexical_this_mode {
                        Some(this.clone())
                    } else {
                        None
                    },
                    Some(environment.clone()),
                    // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                    if lexical_this_mode {
                        BindingStatus::Lexical
                    } else {
                        BindingStatus::Uninitialized
                    },
                    JsValue::undefined(),
                    context,
                )?;

                // Turn local_env into Environment so it can be cloned
                let local_env: Environment = local_env.into();

                // Push the environment first so that it will be used by default parameters
                context.push_environment(local_env.clone());

                let mut arguments_in_parameter_names = false;
                let mut is_simple_parameter_list = true;
                let mut has_parameter_expressions = false;

                let arguments = Sym::ARGUMENTS;
                for param in code.params.iter() {
                    has_parameter_expressions = has_parameter_expressions || param.init().is_some();
                    arguments_in_parameter_names =
                        arguments_in_parameter_names || param.names().contains(&arguments);
                    is_simple_parameter_list = is_simple_parameter_list
                        && !param.is_rest_param()
                        && param.is_identifier()
                        && param.init().is_none()
                }

                // An arguments object is added when all of the following conditions are met
                // - If not in an arrow function (10.2.11.16)
                // - If the parameter list does not contain `arguments` (10.2.11.17)
                // - If there are default parameters or if lexical names and function names do not contain `arguments` (10.2.11.18)
                //
                // https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
                if !lexical_this_mode
                    && !arguments_in_parameter_names
                    && (has_parameter_expressions || !code.lexical_name_argument)
                {
                    // Add arguments object
                    let arguments_obj =
                        if context.strict() || code.strict || !is_simple_parameter_list {
                            Arguments::create_unmapped_arguments_object(args, context)
                        } else {
                            Arguments::create_mapped_arguments_object(
                                &this_function_object,
                                &code.params,
                                args,
                                &local_env,
                                context,
                            )
                        };
                    local_env.create_mutable_binding(arguments, false, true, context)?;
                    local_env.initialize_binding(arguments, arguments_obj.into(), context)?;
                }

                let arg_count = args.len();

                // Push function arguments to the stack.
                let args = if code.params.len() > args.len() {
                    let mut v = args.to_vec();
                    v.extend(vec![JsValue::Undefined; code.params.len() - args.len()]);
                    v
                } else {
                    args.to_vec()
                };

                for arg in args.iter().rev() {
                    context.vm.push(arg)
                }

                let param_count = code.params.len();

                let this = if this.is_null_or_undefined() {
                    context
                        .get_global_this_binding()
                        .expect("global env must have this binding")
                } else {
                    this.to_object(context)
                        .expect("conversion to object cannot fail here")
                        .into()
                };

                context.vm.push_frame(CallFrame {
                    prev: None,
                    code,
                    this,
                    pc: 0,
                    catch: Vec::new(),
                    finally_return: FinallyReturn::None,
                    finally_jump: Vec::new(),
                    pop_on_return: 0,
                    pop_env_on_return: 0,
                    param_count,
                    arg_count,
                });

                let result = context.run();

                context.pop_environment();
                if has_parameter_expressions {
                    context.pop_environment();
                }

                result
            }
        }
    }

    pub(crate) fn construct_internal(
        &self,
        args: &[JsValue],
        this_target: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_function_object = self.clone();
        // let mut has_parameter_expressions = false;

        if !self.is_constructor() {
            return context.throw_type_error("not a constructor function");
        }

        let body = {
            let object = self.borrow();
            let function = object.as_function().unwrap();

            match function {
                Function::Native { function, .. } => FunctionBody::Native {
                    function: *function,
                },
                Function::Closure {
                    function, captures, ..
                } => FunctionBody::Closure {
                    function: function.clone(),
                    captures: captures.clone(),
                },
                Function::VmOrdinary { code, environment } => FunctionBody::Ordinary {
                    code: code.clone(),
                    environment: environment.clone(),
                },
            }
        };

        match body {
            FunctionBody::Native { function, .. } => function(this_target, args, context),
            FunctionBody::Closure { function, captures } => {
                (function)(this_target, args, captures, context)
            }
            FunctionBody::Ordinary { code, environment } => {
                let this: JsValue = {
                    // If the prototype of the constructor is not an object, then use the default object
                    // prototype as prototype for the new object
                    // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
                    // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
                    let prototype = get_prototype_from_constructor(
                        this_target,
                        StandardObjects::object_object,
                        context,
                    )?;
                    JsObject::from_proto_and_data(prototype, ObjectData::ordinary()).into()
                };
                let lexical_this_mode = code.this_mode == ThisMode::Lexical;

                // Create a new Function environment whose parent is set to the scope of the function declaration (self.environment)
                // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                let local_env = FunctionEnvironmentRecord::new(
                    this_function_object.clone(),
                    Some(this.clone()),
                    Some(environment),
                    // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                    if lexical_this_mode {
                        BindingStatus::Lexical
                    } else {
                        BindingStatus::Uninitialized
                    },
                    JsValue::undefined(),
                    context,
                )?;

                // Turn local_env into Environment so it can be cloned
                let local_env: Environment = local_env.into();

                // Push the environment first so that it will be used by default parameters
                context.push_environment(local_env.clone());

                let mut arguments_in_parameter_names = false;
                let mut is_simple_parameter_list = true;
                let mut has_parameter_expressions = false;

                for param in code.params.iter() {
                    has_parameter_expressions = has_parameter_expressions || param.init().is_some();
                    arguments_in_parameter_names =
                        arguments_in_parameter_names || param.names().contains(&Sym::ARGUMENTS);
                    is_simple_parameter_list = is_simple_parameter_list
                        && !param.is_rest_param()
                        && param.is_identifier()
                        && param.init().is_none()
                }

                // An arguments object is added when all of the following conditions are met
                // - If not in an arrow function (10.2.11.16)
                // - If the parameter list does not contain `arguments` (10.2.11.17)
                // - If there are default parameters or if lexical names and function names do not contain `arguments` (10.2.11.18)
                //
                // https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
                if !lexical_this_mode
                    && !arguments_in_parameter_names
                    && (has_parameter_expressions || !code.lexical_name_argument)
                {
                    // Add arguments object
                    let arguments_obj =
                        if context.strict() || code.strict || !is_simple_parameter_list {
                            Arguments::create_unmapped_arguments_object(args, context)
                        } else {
                            Arguments::create_mapped_arguments_object(
                                &this_function_object,
                                &code.params,
                                args,
                                &local_env,
                                context,
                            )
                        };
                    local_env.create_mutable_binding(Sym::ARGUMENTS, false, true, context)?;
                    local_env.initialize_binding(Sym::ARGUMENTS, arguments_obj.into(), context)?;
                }

                let arg_count = args.len();

                // Push function arguments to the stack.
                let args = if code.params.len() > args.len() {
                    let mut v = args.to_vec();
                    v.extend(vec![JsValue::Undefined; code.params.len() - args.len()]);
                    v
                } else {
                    args.to_vec()
                };

                for arg in args.iter().rev() {
                    context.vm.push(arg)
                }

                let param_count = code.params.len();

                let this = if this.is_null_or_undefined() {
                    context
                        .get_global_this_binding()
                        .expect("global env must have this binding")
                } else {
                    this.to_object(context)
                        .expect("conversion to object cannot fail here")
                        .into()
                };

                context.vm.push_frame(CallFrame {
                    prev: None,
                    code,
                    this,
                    pc: 0,
                    catch: Vec::new(),
                    finally_return: FinallyReturn::None,
                    finally_jump: Vec::new(),
                    pop_on_return: 0,
                    pop_env_on_return: 0,
                    param_count,
                    arg_count,
                });

                let result = context.run()?;

                let this = context.get_this_binding();

                context.pop_environment();
                if has_parameter_expressions {
                    context.pop_environment();
                }

                if result.is_object() {
                    Ok(result)
                } else {
                    this
                }
            }
        }
    }
}
