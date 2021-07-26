use crate::{
    environment::{
        function_environment_record::{BindingStatus, FunctionEnvironmentRecord},
        lexical_environment::Environment,
    },
    gc::{Finalize, Trace},
    object::{
        function::{
            Captures, ClosureFunctionSignature, Function, NativeFunctionSignature, ThisMode,
        },
        JsObject, Object, PROTOTYPE,
    },
    property::PropertyDescriptor,
    syntax::ast::node::FormalParameter,
    vm::Opcode,
    Context, JsResult, JsString, JsValue,
};
use gc::Gc;

use std::{convert::TryInto, fmt::Write, mem::size_of};

use super::CallFrame;

/// This represents wether a value can be read from [`CodeBlock`] code.
pub unsafe trait Readable {}

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

#[derive(Debug, Trace, Finalize)]
pub struct CodeBlock {
    /// Name of this function
    pub(crate) name: JsString,

    // The length of this function.
    pub(crate) length: u32,

    /// Is this function in strict mode.
    pub(crate) strict: bool,

    /// Is this function constructable.
    pub(crate) constructable: bool,

    /// [[ThisMode]]
    pub(crate) this_mode: ThisMode,

    pub(crate) params: Box<[FormalParameter]>,

    /// Bytecode
    pub(crate) code: Vec<u8>,

    /// Literals
    pub(crate) literals: Vec<JsValue>,

    /// Variables names
    pub(crate) variables: Vec<JsString>,

    // Functions inside this function
    pub(crate) functions: Vec<Gc<CodeBlock>>,
}

impl CodeBlock {
    pub fn new(name: JsString, length: u32, strict: bool, constructable: bool) -> Self {
        Self {
            code: Vec::new(),
            literals: Vec::new(),
            variables: Vec::new(),
            functions: Vec::new(),
            name,
            length,
            strict,
            constructable,
            this_mode: ThisMode::Global,
            params: Vec::new().into_boxed_slice(),
        }
    }

    /// Read type T from code.
    ///
    /// # Safety
    ///
    /// Does not check if read happens out-of-bounds.
    pub unsafe fn read_unchecked<T: Readable>(&self, offset: usize) -> T {
        // This has to be an unaligned read because we can't gurantee that
        // the types are aligned.
        self.code.as_ptr().add(offset).cast::<T>().read_unaligned()
    }

    /// Read type T from code.
    #[track_caller]
    pub fn read<T: Readable>(&self, offset: usize) -> T {
        assert!(offset + size_of::<T>() - 1 < self.code.len());

        // Safety: We checked that it is not an out-of-bounds read,
        // so this is safe.
        unsafe { self.read_unchecked(offset) }
    }

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
            | Opcode::PushNewArray
            | Opcode::Jump
            | Opcode::JumpIfFalse
            | Opcode::JumpIfTrue
            | Opcode::Case
            | Opcode::Default
            | Opcode::LogicalAnd
            | Opcode::LogicalOr
            | Opcode::Coalesce
            | Opcode::Call => {
                let result = self.read::<u32>(*pc).to_string();
                *pc += size_of::<u32>();
                result
            }
            Opcode::GetFunction => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!(
                    "{:04}: '{}' (length: {})",
                    operand,
                    self.functions[operand as usize].name,
                    self.functions[operand as usize].length
                )
            }
            Opcode::DefVar
            | Opcode::DefLet
            | Opcode::DefConst
            | Opcode::InitLexical
            | Opcode::GetName
            | Opcode::SetName
            | Opcode::GetPropertyByName
            | Opcode::SetPropertyByName => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!("{:04}: '{}'", operand, self.variables[operand as usize])
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
            | Opcode::GetPropertyByValue
            | Opcode::SetPropertyByValue
            | Opcode::ToBoolean
            | Opcode::Throw
            | Opcode::This
            | Opcode::Return
            | Opcode::Nop => String::new(),
        }
    }
}

impl std::fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "----------------- name '{}' (length: {}) ------------------",
            self.name, self.length
        )?;

        writeln!(f, "    Location  Count   Opcode              Operands")?;
        let mut pc = 0;
        let mut count = 0;
        while pc < self.code.len() {
            let opcode: Opcode = self.code[pc].try_into().unwrap();
            write!(
                f,
                "    {:06}    {:04}    {:<20}",
                pc,
                count,
                opcode.as_str()
            )?;
            writeln!(f, "{}", self.instruction_operands(&mut pc))?;
            count += 1;
        }

        f.write_char('\n')?;

        f.write_str("Literals:\n")?;
        if !self.literals.is_empty() {
            for (i, value) in self.literals.iter().enumerate() {
                writeln!(f, "    {:04}: <{}> {}", i, value.type_of(), value.display())?;
            }
        } else {
            writeln!(f, "    <empty>")?;
        }

        f.write_char('\n')?;

        f.write_str("Names:\n")?;
        if !self.variables.is_empty() {
            for (i, value) in self.variables.iter().enumerate() {
                writeln!(f, "    {:04}: {}", i, value)?;
            }
        } else {
            writeln!(f, "    <empty>")?;
        }

        f.write_char('\n')?;

        f.write_str("Functions:\n")?;
        if !self.functions.is_empty() {
            for (i, code) in self.functions.iter().enumerate() {
                writeln!(
                    f,
                    "    {:04}: name: '{}' (length: {})",
                    i, code.name, code.length
                )?;
            }
        } else {
            writeln!(f, "    <empty>")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct JsVmFunction {
    inner: (),
}

impl JsVmFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(code: Gc<CodeBlock>, environment: Environment, context: &mut Context) -> JsObject {
        let function_prototype = context.standard_objects().function_object().prototype();

        let prototype = context.construct_object();

        let name_property = PropertyDescriptor::builder()
            .value(code.name.clone())
            .writable(true)
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

        let constructor = JsObject::new(Object::function(function, function_prototype.into()));

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
            .writable(false)
            .enumerable(false)
            .configurable(true)
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

impl JsObject {
    pub(crate) fn call_internal(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
        exit_on_return: bool,
    ) -> JsResult<JsValue> {
        let this_function_object = self.clone();
        // let mut has_parameter_expressions = false;

        if !self.is_callable() {
            return context.throw_type_error("not a callable function");
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
                Function::Ordinary { .. } => unreachable!(),
            }
        };

        match body {
            FunctionBody::Native { function } => function(this, args, context),
            FunctionBody::Closure { function, captures } => {
                (function)(this, args, captures, context)
            }
            FunctionBody::Ordinary { code, environment } => {
                let lexical_this_mode = code.this_mode == ThisMode::Lexical;

                // Create a new Function environment whose parent is set to the scope of the function declaration (self.environment)
                // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                let local_env = FunctionEnvironmentRecord::new(
                    this_function_object,
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

                // Add argument bindings to the function environment
                for (i, param) in code.params.iter().enumerate() {
                    // Rest Parameters
                    if param.is_rest_param() {
                        todo!("Rest parameter");
                    }

                    let value = match args.get(i).cloned() {
                        None => JsValue::undefined(),
                        Some(value) => value,
                    };

                    Function::add_arguments_to_environment(param, value, &local_env, context);
                }

                context.vm.push_frame(CallFrame {
                    prev: None,
                    code,
                    this: this.clone(),
                    pc: 0,
                    fp: context.vm.stack.len(),
                    exit_on_return,
                    environment: local_env,
                });

                let result = context.run();

                context.pop_environment();

                result
            }
        }
    }

    pub fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        self.call_internal(this, args, context, true)
    }

    pub(crate) fn construct_internal(
        &self,
        args: &[JsValue],
        this_target: &JsValue,
        context: &mut Context,
        exit_on_return: bool,
    ) -> JsResult<JsValue> {
        let this_function_object = self.clone();
        // let mut has_parameter_expressions = false;

        if !self.is_constructable() {
            return context.throw_type_error("not a constructable function");
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
                Function::Ordinary { .. } => unreachable!(),
            }
        };

        match body {
            FunctionBody::Native { function, .. } => function(this_target, args, context),
            FunctionBody::Closure { function, captures } => {
                (function)(this_target, args, captures, context)
            }
            FunctionBody::Ordinary { code, environment } => {
                let this = {
                    // If the prototype of the constructor is not an object, then use the default object
                    // prototype as prototype for the new object
                    // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
                    // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
                    let proto = this_target.as_object().unwrap().__get__(
                        &PROTOTYPE.into(),
                        this_target.clone(),
                        context,
                    )?;
                    let proto = if proto.is_object() {
                        proto
                    } else {
                        context
                            .standard_objects()
                            .object_object()
                            .prototype()
                            .into()
                    };
                    JsValue::from(Object::create(proto))
                };
                let lexical_this_mode = code.this_mode == ThisMode::Lexical;

                // Create a new Function environment whose parent is set to the scope of the function declaration (self.environment)
                // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                let local_env = FunctionEnvironmentRecord::new(
                    this_function_object,
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

                // Add argument bindings to the function environment
                for (i, param) in code.params.iter().enumerate() {
                    // Rest Parameters
                    if param.is_rest_param() {
                        todo!("Rest parameter");
                    }

                    let value = match args.get(i).cloned() {
                        None => JsValue::undefined(),
                        Some(value) => value,
                    };

                    Function::add_arguments_to_environment(param, value, &local_env, context);
                }

                context.vm.push_frame(CallFrame {
                    prev: None,
                    code,
                    this,
                    pc: 0,
                    fp: context.vm.stack.len(),
                    exit_on_return,
                    environment: local_env,
                });

                let _result = context.run();

                context.pop_environment();

                context.get_this_binding()
            }
        }
    }

    pub fn construct(
        &self,
        args: &[JsValue],
        this_target: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        self.construct_internal(args, this_target, context, true)
    }
}
