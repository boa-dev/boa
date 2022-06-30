//! `CodeBlock`
//!
//! This module is for the `CodeBlock` which implements a function representation in the VM

use crate::{
    builtins::{
        function::{
            arguments::Arguments, ClassFieldDefinition, ConstructorKind, Function, ThisMode,
        },
        generator::{Generator, GeneratorContext, GeneratorState},
    },
    context::intrinsics::StandardConstructors,
    environments::{BindingLocator, CompileTimeEnvironment},
    object::{
        internal_methods::get_prototype_from_constructor, JsObject, ObjectData, PrivateElement,
    },
    property::PropertyDescriptor,
    syntax::ast::node::FormalParameterList,
    vm::call_frame::GeneratorResumeKind,
    vm::{call_frame::FinallyReturn, CallFrame, Opcode},
    Context, JsResult, JsValue,
};
use boa_gc::{Cell, Finalize, Gc, Trace};
use boa_interner::{Interner, Sym, ToInternedString};
use boa_profiler::Profiler;
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
/// A `CodeBlock` is generated for each function compiled by the
/// [`ByteCompiler`](crate::bytecompiler::ByteCompiler). It stores the bytecode and the other
/// attributes of the function.
#[derive(Clone, Debug, Trace, Finalize)]
pub struct CodeBlock {
    /// Name of this function
    #[unsafe_ignore_trace]
    pub(crate) name: Sym,

    /// The number of arguments expected.
    pub(crate) length: u32,

    /// Is this function in strict mode.
    pub(crate) strict: bool,

    /// \[\[ThisMode\]\]
    pub(crate) this_mode: ThisMode,

    /// Parameters passed to this function.
    #[unsafe_ignore_trace]
    pub(crate) params: FormalParameterList,

    /// Bytecode
    pub(crate) code: Vec<u8>,

    /// Literals
    pub(crate) literals: Vec<JsValue>,

    /// Property field names.
    #[unsafe_ignore_trace]
    pub(crate) names: Vec<Sym>,

    /// Locators for all bindings in the codeblock.
    #[unsafe_ignore_trace]
    pub(crate) bindings: Vec<BindingLocator>,

    /// Number of binding for the function environment.
    pub(crate) num_bindings: usize,

    /// Functions inside this function
    pub(crate) functions: Vec<Gc<Self>>,

    /// The `arguments` binding location of the function, if set.
    #[unsafe_ignore_trace]
    pub(crate) arguments_binding: Option<BindingLocator>,

    /// Compile time environments in this function.
    pub(crate) compile_environments: Vec<Gc<Cell<CompileTimeEnvironment>>>,

    /// The `[[IsClassConstructor]]` internal slot.
    pub(crate) is_class_constructor: bool,
}

impl CodeBlock {
    /// Constructs a new `CodeBlock`.
    pub fn new(name: Sym, length: u32, strict: bool) -> Self {
        Self {
            code: Vec::new(),
            literals: Vec::new(),
            names: Vec::new(),
            bindings: Vec::new(),
            num_bindings: 0,
            functions: Vec::new(),
            name,
            length,
            strict,
            this_mode: ThisMode::Global,
            params: FormalParameterList::default(),
            arguments_binding: None,
            compile_environments: Vec::new(),
            is_class_constructor: false,
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
    pub(crate) fn instruction_operands(&self, pc: &mut usize, interner: &Interner) -> String {
        let opcode: Opcode = self.code[*pc].try_into().expect("invalid opcode");
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
            | Opcode::CallEval
            | Opcode::CallEvalWithRest
            | Opcode::Call
            | Opcode::CallWithRest
            | Opcode::New
            | Opcode::NewWithRest
            | Opcode::SuperCall
            | Opcode::SuperCallWithRest
            | Opcode::ForInLoopInitIterator
            | Opcode::ForInLoopNext
            | Opcode::ConcatToString
            | Opcode::CopyDataProperties
            | Opcode::GeneratorNextDelegate => {
                let result = self.read::<u32>(*pc).to_string();
                *pc += size_of::<u32>();
                result
            }
            Opcode::TryStart
            | Opcode::PushDeclarativeEnvironment
            | Opcode::PushFunctionEnvironment => {
                let operand1 = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                let operand2 = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!("{operand1}, {operand2}")
            }
            Opcode::GetFunction | Opcode::GetGenerator => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!(
                    "{operand:04}: '{:?}' (length: {})",
                    interner.resolve_expect(self.functions[operand as usize].name),
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
            | Opcode::SetName => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!(
                    "{:04}: '{}'",
                    operand,
                    interner.resolve_expect(self.bindings[operand as usize].name()),
                )
            }
            Opcode::GetPropertyByName
            | Opcode::SetPropertyByName
            | Opcode::DefineOwnPropertyByName
            | Opcode::DefineClassMethodByName
            | Opcode::SetPropertyGetterByName
            | Opcode::DefineClassGetterByName
            | Opcode::SetPropertySetterByName
            | Opcode::DefineClassSetterByName
            | Opcode::AssignPrivateField
            | Opcode::SetPrivateField
            | Opcode::SetPrivateMethod
            | Opcode::SetPrivateSetter
            | Opcode::SetPrivateGetter
            | Opcode::GetPrivateField
            | Opcode::DeletePropertyByName
            | Opcode::PushClassFieldPrivate
            | Opcode::PushClassPrivateGetter
            | Opcode::PushClassPrivateSetter
            | Opcode::PushClassPrivateMethod => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!(
                    "{operand:04}: '{}'",
                    interner.resolve_expect(self.names[operand as usize]),
                )
            }
            Opcode::Pop
            | Opcode::PopIfThrown
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
            | Opcode::PushClassPrototype
            | Opcode::SetClassPrototype
            | Opcode::SetHomeObject
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
            | Opcode::IncPost
            | Opcode::Dec
            | Opcode::DecPost
            | Opcode::GetPropertyByValue
            | Opcode::SetPropertyByValue
            | Opcode::DefineOwnPropertyByValue
            | Opcode::DefineClassMethodByValue
            | Opcode::SetPropertyGetterByValue
            | Opcode::DefineClassGetterByValue
            | Opcode::SetPropertySetterByValue
            | Opcode::DefineClassSetterByValue
            | Opcode::DeletePropertyByValue
            | Opcode::ToPropertyKey
            | Opcode::ToBoolean
            | Opcode::Throw
            | Opcode::TryEnd
            | Opcode::CatchEnd
            | Opcode::CatchEnd2
            | Opcode::FinallyStart
            | Opcode::FinallyEnd
            | Opcode::This
            | Opcode::Super
            | Opcode::Return
            | Opcode::PopEnvironment
            | Opcode::LoopStart
            | Opcode::LoopContinue
            | Opcode::LoopEnd
            | Opcode::InitIterator
            | Opcode::IteratorNext
            | Opcode::IteratorClose
            | Opcode::IteratorToArray
            | Opcode::RequireObjectCoercible
            | Opcode::ValueNotNullOrUndefined
            | Opcode::RestParameterInit
            | Opcode::RestParameterPop
            | Opcode::PushValueToArray
            | Opcode::PushElisionToArray
            | Opcode::PushIteratorToArray
            | Opcode::PushNewArray
            | Opcode::PopOnReturnAdd
            | Opcode::PopOnReturnSub
            | Opcode::Yield
            | Opcode::GeneratorNext
            | Opcode::PushClassField
            | Opcode::SuperCallDerived
            | Opcode::Nop => String::new(),
        }
    }
}

impl ToInternedString for CodeBlock {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let name = interner.resolve_expect(self.name);
        let mut f = if self.name == Sym::MAIN {
            String::new()
        } else {
            "\n".to_owned()
        };

        f.push_str(&format!(
            "{:-^70}\nLocation  Count   Opcode                     Operands\n\n",
            format!("Compiled Output: '{name}'"),
        ));

        let mut pc = 0;
        let mut count = 0;
        while pc < self.code.len() {
            let opcode: Opcode = self.code[pc].try_into().expect("invalid opcode");
            let opcode = opcode.as_str();
            let previous_pc = pc;
            let operands = self.instruction_operands(&mut pc, interner);
            f.push_str(&format!(
                "{previous_pc:06}    {count:04}    {opcode:<27}{operands}\n",
            ));
            count += 1;
        }

        f.push_str("\nLiterals:\n");

        if self.literals.is_empty() {
            f.push_str("    <empty>\n");
        } else {
            for (i, value) in self.literals.iter().enumerate() {
                f.push_str(&format!(
                    "    {i:04}: <{}> {}\n",
                    value.type_of(),
                    value.display()
                ));
            }
        }

        f.push_str("\nBindings:\n");
        if self.bindings.is_empty() {
            f.push_str("    <empty>\n");
        } else {
            for (i, binding_locator) in self.bindings.iter().enumerate() {
                f.push_str(&format!(
                    "    {i:04}: {}\n",
                    interner.resolve_expect(binding_locator.name())
                ));
            }
        }

        f.push_str("\nFunctions:\n");
        if self.functions.is_empty() {
            f.push_str("    <empty>\n");
        } else {
            for (i, code) in self.functions.iter().enumerate() {
                f.push_str(&format!(
                    "    {i:04}: name: '{}' (length: {})\n",
                    interner.resolve_expect(code.name),
                    code.length
                ));
            }
        }

        f
    }
}

/// Creates a new function object.
pub(crate) fn create_function_object(code: Gc<CodeBlock>, context: &mut Context) -> JsObject {
    let _timer = Profiler::global().start_event("JsVmFunction::new", "vm");

    let function_prototype = context.intrinsics().constructors().function().prototype();

    let prototype = context.construct_object();

    let name_property = PropertyDescriptor::builder()
        .value(context.interner().resolve_expect(code.name))
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

    let function = Function::Ordinary {
        code,
        environments: context.realm.environments.clone(),
        constructor_kind: ConstructorKind::Base,
        home_object: None,
        fields: Vec::new(),
        private_methods: Vec::new(),
    };

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
        .expect("failed to define the constructor property of the function");

    let prototype_property = PropertyDescriptor::builder()
        .value(prototype)
        .writable(true)
        .enumerable(false)
        .configurable(false)
        .build();

    constructor
        .define_property_or_throw("prototype", prototype_property, context)
        .expect("failed to define the prototype property of the function");
    constructor
        .define_property_or_throw("name", name_property, context)
        .expect("failed to define the name property of the function");
    constructor
        .define_property_or_throw("length", length_property, context)
        .expect("failed to define the length property of the function");

    constructor
}

/// Creates a new generator function object.
pub(crate) fn create_generator_function_object(
    code: Gc<CodeBlock>,
    context: &mut Context,
) -> JsObject {
    let function_prototype = context
        .intrinsics()
        .constructors()
        .generator_function()
        .prototype();

    let name_property = PropertyDescriptor::builder()
        .value(context.interner().resolve_expect(code.name))
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

    let prototype = JsObject::from_proto_and_data(
        context.intrinsics().constructors().generator().prototype(),
        ObjectData::ordinary(),
    );

    let function = Function::Generator {
        code,
        environments: context.realm.environments.clone(),
    };

    let constructor =
        JsObject::from_proto_and_data(function_prototype, ObjectData::generator_function(function));

    let prototype_property = PropertyDescriptor::builder()
        .value(prototype)
        .writable(true)
        .enumerable(false)
        .configurable(false)
        .build();

    constructor
        .define_property_or_throw("prototype", prototype_property, context)
        .expect("failed to define the prototype property of the generator function");
    constructor
        .define_property_or_throw("name", name_property, context)
        .expect("failed to define the name property of the generator function");
    constructor
        .define_property_or_throw("length", length_property, context)
        .expect("failed to define the length property of the generator function");

    constructor
}

impl JsObject {
    pub(crate) fn call_internal(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_function_object = self.clone();

        if !self.is_callable() {
            return context.throw_type_error("not a callable function");
        }

        let object = self.borrow();
        let function_object = object.as_function().expect("not a function");

        match function_object {
            Function::Native {
                function,
                constructor,
            } => {
                let function = *function;
                let constructor = *constructor;
                drop(object);

                if constructor.is_some() {
                    function(&JsValue::undefined(), args, context)
                } else {
                    function(this, args, context)
                }
            }
            Function::Closure {
                function, captures, ..
            } => {
                let function = function.clone();
                let captures = captures.clone();
                drop(object);

                (function)(this, args, captures, context)
            }
            Function::Ordinary {
                code, environments, ..
            } => {
                let code = code.clone();
                let mut environments = environments.clone();
                drop(object);

                if code.is_class_constructor {
                    return context
                        .throw_type_error("Class constructor cannot be invoked without 'new'");
                }

                std::mem::swap(&mut environments, &mut context.realm.environments);

                let lexical_this_mode = code.this_mode == ThisMode::Lexical;

                let this = if lexical_this_mode {
                    None
                } else if code.strict {
                    Some(this.clone())
                } else if this.is_null_or_undefined() {
                    Some(context.global_object().clone().into())
                } else {
                    Some(
                        this.to_object(context)
                            .expect("conversion cannot fail")
                            .into(),
                    )
                };

                if code.params.has_expressions() {
                    context.realm.environments.push_function(
                        code.num_bindings,
                        code.compile_environments[1].clone(),
                        this,
                        self.clone(),
                        None,
                        lexical_this_mode,
                    );
                } else {
                    context.realm.environments.push_function(
                        code.num_bindings,
                        code.compile_environments[0].clone(),
                        this,
                        self.clone(),
                        None,
                        lexical_this_mode,
                    );
                }

                if let Some(binding) = code.arguments_binding {
                    let arguments_obj = if code.strict || !code.params.is_simple() {
                        Arguments::create_unmapped_arguments_object(args, context)
                    } else {
                        let env = context.realm.environments.current();
                        Arguments::create_mapped_arguments_object(
                            &this_function_object,
                            &code.params,
                            args,
                            &env,
                            context,
                        )
                    };
                    context.realm.environments.put_value(
                        binding.environment_index(),
                        binding.binding_index(),
                        arguments_obj.into(),
                    );
                }

                let arg_count = args.len();

                // Push function arguments to the stack.
                let args = if code.params.parameters.len() > args.len() {
                    let mut v = args.to_vec();
                    v.extend(vec![
                        JsValue::Undefined;
                        code.params.parameters.len() - args.len()
                    ]);
                    v
                } else {
                    args.to_vec()
                };

                for arg in args.iter().rev() {
                    context.vm.push(arg);
                }

                let param_count = code.params.parameters.len();
                let has_expressions = code.params.has_expressions();

                context.vm.push_frame(CallFrame {
                    prev: None,
                    code,
                    pc: 0,
                    catch: Vec::new(),
                    finally_return: FinallyReturn::None,
                    finally_jump: Vec::new(),
                    pop_on_return: 0,
                    loop_env_stack: Vec::from([0]),
                    try_env_stack: Vec::from([crate::vm::TryStackEntry {
                        num_env: 0,
                        num_loop_stack_entries: 0,
                    }]),
                    param_count,
                    arg_count,
                    generator_resume_kind: GeneratorResumeKind::Normal,
                    thrown: false,
                });

                let result = context.run();
                context.vm.pop_frame().expect("must have frame");

                context.realm.environments.pop();
                if has_expressions {
                    context.realm.environments.pop();
                }

                std::mem::swap(&mut environments, &mut context.realm.environments);

                let (result, _) = result?;
                Ok(result)
            }
            Function::Generator { code, environments } => {
                let code = code.clone();
                let mut environments = environments.clone();
                drop(object);

                std::mem::swap(&mut environments, &mut context.realm.environments);

                let lexical_this_mode = code.this_mode == ThisMode::Lexical;

                let this = if lexical_this_mode {
                    None
                } else if code.strict {
                    Some(this.clone())
                } else if this.is_null_or_undefined() {
                    Some(context.global_object().clone().into())
                } else {
                    Some(
                        this.to_object(context)
                            .expect("conversion cannot fail")
                            .into(),
                    )
                };

                if code.params.has_expressions() {
                    context.realm.environments.push_function(
                        code.num_bindings,
                        code.compile_environments[1].clone(),
                        this,
                        self.clone(),
                        None,
                        lexical_this_mode,
                    );
                } else {
                    context.realm.environments.push_function(
                        code.num_bindings,
                        code.compile_environments[0].clone(),
                        this,
                        self.clone(),
                        None,
                        lexical_this_mode,
                    );
                }

                if let Some(binding) = code.arguments_binding {
                    let arguments_obj = if code.strict || !code.params.is_simple() {
                        Arguments::create_unmapped_arguments_object(args, context)
                    } else {
                        let env = context.realm.environments.current();
                        Arguments::create_mapped_arguments_object(
                            &this_function_object,
                            &code.params,
                            args,
                            &env,
                            context,
                        )
                    };
                    context.realm.environments.put_value(
                        binding.environment_index(),
                        binding.binding_index(),
                        arguments_obj.into(),
                    );
                }

                let arg_count = args.len();

                // Push function arguments to the stack.
                let mut args = if code.params.parameters.len() > args.len() {
                    let mut v = args.to_vec();
                    v.extend(vec![
                        JsValue::Undefined;
                        code.params.parameters.len() - args.len()
                    ]);
                    v
                } else {
                    args.to_vec()
                };
                args.reverse();

                let param_count = code.params.parameters.len();

                let call_frame = CallFrame {
                    prev: None,
                    code,
                    pc: 0,
                    catch: Vec::new(),
                    finally_return: FinallyReturn::None,
                    finally_jump: Vec::new(),
                    pop_on_return: 0,
                    loop_env_stack: Vec::from([0]),
                    try_env_stack: Vec::from([crate::vm::TryStackEntry {
                        num_env: 0,
                        num_loop_stack_entries: 0,
                    }]),
                    param_count,
                    arg_count,
                    generator_resume_kind: GeneratorResumeKind::Normal,
                    thrown: false,
                };
                let mut stack = args;

                std::mem::swap(&mut context.vm.stack, &mut stack);
                context.vm.push_frame(call_frame);

                let init_result = context.run();

                let call_frame = context.vm.pop_frame().expect("frame must exist");
                std::mem::swap(&mut environments, &mut context.realm.environments);
                std::mem::swap(&mut context.vm.stack, &mut stack);

                let prototype = if let Some(prototype) = this_function_object
                    .get("prototype", context)
                    .expect("GeneratorFunction must have a prototype property")
                    .as_object()
                {
                    prototype.clone()
                } else {
                    context.intrinsics().constructors().generator().prototype()
                };

                let generator = Self::from_proto_and_data(
                    prototype,
                    ObjectData::generator(Generator {
                        state: GeneratorState::SuspendedStart,
                        context: Some(Gc::new(Cell::new(GeneratorContext {
                            environments,
                            call_frame: *call_frame,
                            stack,
                        }))),
                    }),
                );

                init_result?;

                Ok(generator.into())
            }
        }
    }

    pub(crate) fn construct_internal(
        &self,
        args: &[JsValue],
        this_target: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let this_function_object = self.clone();

        let create_this = |context| {
            let prototype =
                get_prototype_from_constructor(this_target, StandardConstructors::object, context)?;
            Ok(Self::from_proto_and_data(prototype, ObjectData::ordinary()))
        };

        if !self.is_constructor() {
            return context.throw_type_error("not a constructor function");
        }

        let object = self.borrow();
        let function_object = object.as_function().expect("not a function");

        match function_object {
            Function::Native {
                function,
                constructor,
                ..
            } => {
                let function = *function;
                let constructor = *constructor;
                drop(object);

                match function(this_target, args, context)? {
                    JsValue::Object(ref o) => Ok(o.clone()),
                    val => {
                        if constructor.expect("hmm").is_base() || val.is_undefined() {
                            create_this(context)
                        } else {
                            context.throw_type_error(
                                "Derived constructor can only return an Object or undefined",
                            )
                        }
                    }
                }
            }
            Function::Closure {
                function,
                captures,
                constructor,
                ..
            } => {
                let function = function.clone();
                let captures = captures.clone();
                let constructor = *constructor;
                drop(object);

                match (function)(this_target, args, captures, context)? {
                    JsValue::Object(ref o) => Ok(o.clone()),
                    val => {
                        if constructor.expect("hmma").is_base() || val.is_undefined() {
                            create_this(context)
                        } else {
                            context.throw_type_error(
                                "Derived constructor can only return an Object or undefined",
                            )
                        }
                    }
                }
            }
            Function::Ordinary {
                code,
                environments,
                constructor_kind,
                ..
            } => {
                let code = code.clone();
                let mut environments = environments.clone();
                let constructor_kind = *constructor_kind;
                drop(object);

                std::mem::swap(&mut environments, &mut context.realm.environments);

                let this = if constructor_kind.is_base() {
                    // If the prototype of the constructor is not an object, then use the default object
                    // prototype as prototype for the new object
                    // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
                    // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
                    let prototype = get_prototype_from_constructor(
                        this_target,
                        StandardConstructors::object,
                        context,
                    )?;
                    let this = Self::from_proto_and_data(prototype, ObjectData::ordinary());

                    initialize_instance_elements(&this, self, context)?;

                    Some(this)
                } else {
                    None
                };

                let new_target = this_target.as_object().expect("must be object");

                if code.params.has_expressions() {
                    context.realm.environments.push_function(
                        code.num_bindings,
                        code.compile_environments[1].clone(),
                        this.clone().map(Into::into),
                        self.clone(),
                        Some(new_target.clone()),
                        false,
                    );
                } else {
                    context.realm.environments.push_function(
                        code.num_bindings,
                        code.compile_environments[0].clone(),
                        this.clone().map(Into::into),
                        self.clone(),
                        Some(new_target.clone()),
                        false,
                    );
                }

                let mut arguments_in_parameter_names = false;
                let mut is_simple_parameter_list = true;
                let mut has_parameter_expressions = false;

                for param in code.params.parameters.iter() {
                    has_parameter_expressions = has_parameter_expressions || param.init().is_some();
                    arguments_in_parameter_names =
                        arguments_in_parameter_names || param.names().contains(&Sym::ARGUMENTS);
                    is_simple_parameter_list = is_simple_parameter_list
                        && !param.is_rest_param()
                        && param.is_identifier()
                        && param.init().is_none();
                }

                if let Some(binding) = code.arguments_binding {
                    let arguments_obj = if code.strict || !code.params.is_simple() {
                        Arguments::create_unmapped_arguments_object(args, context)
                    } else {
                        let env = context.realm.environments.current();
                        Arguments::create_mapped_arguments_object(
                            &this_function_object,
                            &code.params,
                            args,
                            &env,
                            context,
                        )
                    };
                    context.realm.environments.put_value(
                        binding.environment_index(),
                        binding.binding_index(),
                        arguments_obj.into(),
                    );
                }

                let arg_count = args.len();

                // Push function arguments to the stack.
                let args = if code.params.parameters.len() > args.len() {
                    let mut v = args.to_vec();
                    v.extend(vec![
                        JsValue::Undefined;
                        code.params.parameters.len() - args.len()
                    ]);
                    v
                } else {
                    args.to_vec()
                };

                for arg in args.iter().rev() {
                    context.vm.push(arg);
                }

                let param_count = code.params.parameters.len();

                context.vm.push_frame(CallFrame {
                    prev: None,
                    code,
                    pc: 0,
                    catch: Vec::new(),
                    finally_return: FinallyReturn::None,
                    finally_jump: Vec::new(),
                    pop_on_return: 0,
                    loop_env_stack: Vec::from([0]),
                    try_env_stack: Vec::from([crate::vm::TryStackEntry {
                        num_env: 0,
                        num_loop_stack_entries: 0,
                    }]),
                    param_count,
                    arg_count,
                    generator_resume_kind: GeneratorResumeKind::Normal,
                    thrown: false,
                });

                let result = context.run();

                context.vm.pop_frame();

                let mut environment = context.realm.environments.pop();
                if has_parameter_expressions {
                    environment = context.realm.environments.pop();
                }

                std::mem::swap(&mut environments, &mut context.realm.environments);

                let (result, _) = result?;

                if let Some(result) = result.as_object() {
                    Ok(result.clone())
                } else if let Some(this) = this {
                    Ok(this)
                } else if !result.is_undefined() {
                    context.throw_type_error("Function constructor must not return non-object")
                } else {
                    let function_env = environment
                        .slots()
                        .expect("must be function environment")
                        .as_function_slots()
                        .expect("must be function environment");
                    if let Some(this_binding) = function_env.borrow().get_this_binding() {
                        Ok(this_binding
                            .as_object()
                            .expect("this binding must be object")
                            .clone())
                    } else {
                        //context.throw_type_error("Function constructor must not return non-object")
                        context.throw_reference_error("Must call super constructor in derived class before accessing 'this' or returning from derived constructor")
                    }
                }
            }
            Function::Generator { .. } => {
                unreachable!("generator function cannot be a constructor")
            }
        }
    }
}

/// `InitializeInstanceElements ( O, constructor )`
///
/// Add private methods and fields from a class constructor to an object.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-initializeinstanceelements
pub(crate) fn initialize_instance_elements(
    target: &JsObject,
    constructor: &JsObject,
    context: &mut Context,
) -> JsResult<()> {
    let constructor_borrow = constructor.borrow();
    let constructor_function = constructor_borrow
        .as_function()
        .expect("class constructor must be function object");

    for (name, private_method) in constructor_function.get_private_methods() {
        match private_method {
            PrivateElement::Method(_) => {
                target
                    .borrow_mut()
                    .set_private_element(*name, private_method.clone());
            }
            PrivateElement::Accessor { getter, setter } => {
                if let Some(getter) = getter {
                    target
                        .borrow_mut()
                        .set_private_element_getter(*name, getter.clone());
                }
                if let Some(setter) = setter {
                    target
                        .borrow_mut()
                        .set_private_element_setter(*name, setter.clone());
                }
            }
            PrivateElement::Field(_) => unreachable!(),
        }
    }

    for field in constructor_function.get_fields() {
        match field {
            ClassFieldDefinition::Public(name, function) => {
                let value = function.call(&target.clone().into(), &[], context)?;
                target.__define_own_property__(
                    name.clone(),
                    PropertyDescriptor::builder()
                        .value(value)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true)
                        .build(),
                    context,
                )?;
            }
            ClassFieldDefinition::Private(name, function) => {
                let value = function.call(&target.clone().into(), &[], context)?;
                target
                    .borrow_mut()
                    .set_private_element(*name, PrivateElement::Field(value));
            }
        }
    }

    Ok(())
}
