use crate::vm::Instruction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// An address is a bytecode offset, displayed as hexadecimal.
pub struct Address(pub(crate) u32);

impl Address {
    /// Create a new [`Address`] from a u32 value.
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the inner `u32` value.
    pub(crate) const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<Address> for u32 {
    fn from(addr: Address) -> Self {
        addr.0
    }
}

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl std::ops::Add<u32> for Address {
    type Output = Self;

    fn add(self, rhs: u32) -> Self {
        Self::new(self.0 + rhs)
    }
}

impl std::fmt::Display for Address {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:06x}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A register operand is a register index used in bytecode instructions.
pub struct RegisterOperand(pub(crate) u32);

impl RegisterOperand {
    /// Create a new [`RegisterOperand`] from a u32 value.
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }
}

impl From<RegisterOperand> for u32 {
    fn from(value: RegisterOperand) -> Self {
        value.0
    }
}

impl From<RegisterOperand> for usize {
    fn from(value: RegisterOperand) -> Self {
        value.0 as usize
    }
}

impl From<u8> for RegisterOperand {
    fn from(value: u8) -> Self {
        Self::new(value.into())
    }
}

impl From<u16> for RegisterOperand {
    fn from(value: u16) -> Self {
        Self::new(value.into())
    }
}

impl From<u32> for RegisterOperand {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for RegisterOperand {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{:02}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A index operand is e.g. an index into the constant pool
pub struct IndexOperand(pub(crate) u32);

impl IndexOperand {
    /// Create a new [`IndexOperand`] from a u32 value.
    pub(crate) fn new(value: u32) -> Self {
        Self(value)
    }
}

impl From<IndexOperand> for u32 {
    fn from(value: IndexOperand) -> Self {
        value.0
    }
}

impl From<IndexOperand> for usize {
    fn from(value: IndexOperand) -> Self {
        value.0 as usize
    }
}

impl From<bool> for IndexOperand {
    fn from(value: bool) -> Self {
        Self::new(value.into())
    }
}

impl From<u8> for IndexOperand {
    fn from(value: u8) -> Self {
        Self::new(value.into())
    }
}

impl From<u16> for IndexOperand {
    fn from(value: u16) -> Self {
        Self::new(value.into())
    }
}

impl From<u32> for IndexOperand {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for IndexOperand {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Available Operands types that Boa's VM uses
#[expect(missing_docs)]
#[derive(Clone, Debug, PartialEq)]
pub enum OperandsShape {
    None,
    Dst {
        dst: RegisterOperand,
    },
    LhsRhsDst {
        lhs: RegisterOperand,
        rhs: RegisterOperand,
        dst: RegisterOperand,
    },
    RhsIndexDst {
        rhs: RegisterOperand,
        index: IndexOperand,
        dst: RegisterOperand,
    },
    SrcDst {
        src: RegisterOperand,
        dst: RegisterOperand,
    },
    SetFunctionName {
        function: RegisterOperand,
        name: RegisterOperand,
        prefix: u8,
    },
    ValueDstI8 {
        value: i8,
        dst: RegisterOperand,
    },
    ValueDstI16 {
        value: i16,
        dst: RegisterOperand,
    },
    ValueDstI32 {
        value: i32,
        dst: RegisterOperand,
    },
    ValueDstF32 {
        value: f32,
        dst: RegisterOperand,
    },
    ValueDstF64 {
        value: f64,
        dst: RegisterOperand,
    },
    IndexDst {
        index: IndexOperand,
        dst: RegisterOperand,
    },
    Message {
        message: IndexOperand,
    },
    Regexp {
        pattern_index: IndexOperand,
        flags_index: IndexOperand,
        dst: RegisterOperand,
    },
    Address {
        address: Address,
    },
    AddressValue {
        address: Address,
        value: RegisterOperand,
    },
    AddressLhsRhs {
        address: Address,
        lhs: RegisterOperand,
        rhs: RegisterOperand,
    },
    Case {
        address: Address,
        value: RegisterOperand,
        condition: RegisterOperand,
    },
    CallEval {
        argument_count: IndexOperand,
        scope_index: IndexOperand,
    },
    ScopeIndex {
        scope_index: IndexOperand,
    },
    ArgumentCount {
        argument_count: IndexOperand,
    },
    BindingIndex {
        binding_index: IndexOperand,
    },
    SrcBindingIndex {
        src: RegisterOperand,
        binding_index: IndexOperand,
    },
    DstBindingIndex {
        dst: RegisterOperand,
        binding_index: IndexOperand,
    },
    GetNameGlobal {
        dst: RegisterOperand,
        binding_index: IndexOperand,
        ic_index: IndexOperand,
    },
    ObjectValueName {
        object: RegisterOperand,
        value: RegisterOperand,
        name_index: IndexOperand,
    },
    DstObjectName {
        dst: RegisterOperand,
        object: RegisterOperand,
        name_index: IndexOperand,
    },
    ObjectProtoValueName {
        object: RegisterOperand,
        proto: RegisterOperand,
        value: RegisterOperand,
        name_index: IndexOperand,
    },
    Index {
        index: IndexOperand,
    },
    ObjectName {
        object: RegisterOperand,
        name_index: IndexOperand,
    },
    DstValueIc {
        dst: RegisterOperand,
        value: RegisterOperand,
        ic_index: IndexOperand,
    },
    DstReceiverValueIc {
        dst: RegisterOperand,
        receiver: RegisterOperand,
        value: RegisterOperand,
        ic_index: IndexOperand,
    },
    ObjectValueIc {
        object: RegisterOperand,
        value: RegisterOperand,
        ic_index: IndexOperand,
    },
    ObjectReceiverValueIc {
        object: RegisterOperand,
        receiver: RegisterOperand,
        value: RegisterOperand,
        ic_index: IndexOperand,
    },
    DstKeyReceiverObject {
        dst: RegisterOperand,
        key: RegisterOperand,
        receiver: RegisterOperand,
        object: RegisterOperand,
    },
    ObjectReceiverKeyValue {
        object: RegisterOperand,
        receiver: RegisterOperand,
        key: RegisterOperand,
        value: RegisterOperand,
    },
    ObjectKeyValue {
        object: RegisterOperand,
        key: RegisterOperand,
        value: RegisterOperand,
    },
    ObjectKey {
        object: RegisterOperand,
        key: RegisterOperand,
    },
    ValueDone {
        value: RegisterOperand,
        done: IndexOperand,
    },
    DstClassSuperclass {
        dst: RegisterOperand,
        class: RegisterOperand,
        superclass: RegisterOperand,
    },
    DstPrototypeClass {
        dst: RegisterOperand,
        prototype: RegisterOperand,
        class: RegisterOperand,
    },
    FunctionHome {
        function: RegisterOperand,
        home: RegisterOperand,
    },
    Function {
        function: RegisterOperand,
    },
    ObjectPrototype {
        object: RegisterOperand,
        prototype: RegisterOperand,
    },
    Object {
        object: RegisterOperand,
    },
    ValueArray {
        value: RegisterOperand,
        array: RegisterOperand,
    },
    Array {
        array: RegisterOperand,
    },
    Value {
        value: RegisterOperand,
    },
    SpecifierOptions {
        specifier: RegisterOperand,
        options: RegisterOperand,
        phase: IndexOperand,
    },
    ClassField {
        object: RegisterOperand,
        name: RegisterOperand,
        value: RegisterOperand,
        is_anonymous_function: IndexOperand,
    },
    MaybeException {
        has_exception: RegisterOperand,
        exception: RegisterOperand,
    },
    Src {
        src: RegisterOperand,
    },
    IteratorNextReg {
        iterator: RegisterOperand,
        next: RegisterOperand,
    },
    Result {
        result: RegisterOperand,
    },
    ResumeKindValue {
        resume_kind: RegisterOperand,
        value: RegisterOperand,
    },
    ValueCalled {
        value: RegisterOperand,
        called: RegisterOperand,
    },
    SrcConfigurableName {
        src: RegisterOperand,
        configurable: RegisterOperand,
        name_index: IndexOperand,
    },
    ConfigurableName {
        configurable: bool,
        name_index: IndexOperand,
    },
    ClassNames {
        class: RegisterOperand,
        name_indices: Box<[u32]>,
    },
    AddressSiteDst {
        address: Address,
        site: u64,
        dst: RegisterOperand,
    },
    JumpTable {
        index: u32,
        addresses: Box<[Address]>,
    },
    DstValues {
        dst: RegisterOperand,
        values: Box<[RegisterOperand]>,
    },
    ObjectSourceExcluded {
        object: RegisterOperand,
        source: RegisterOperand,
        excluded_keys: Box<[RegisterOperand]>,
    },
    SiteDstValues {
        site: u64,
        dst: RegisterOperand,
        values: Box<[u32]>,
    },
    FunctionObject {
        function_object: RegisterOperand,
    },
}

impl OperandsShape {
    pub(crate) fn from_instruction(instruction: &Instruction) -> Self {
        match instruction {
            Instruction::Pop
            | Instruction::DeleteSuperThrow
            | Instruction::ReThrow
            | Instruction::CheckReturn
            | Instruction::Return
            | Instruction::AsyncGeneratorClose
            | Instruction::CreatePromiseCapability
            | Instruction::PopEnvironment
            | Instruction::IncrementLoopIteration
            | Instruction::IteratorNext
            | Instruction::SuperCallDerived
            | Instruction::CallSpread
            | Instruction::NewSpread
            | Instruction::SuperCallSpread
            | Instruction::PopPrivateEnvironment
            | Instruction::Generator
            | Instruction::AsyncGenerator => OperandsShape::None,

            Instruction::Add { lhs, rhs, dst }
            | Instruction::Sub { lhs, rhs, dst }
            | Instruction::Div { lhs, rhs, dst }
            | Instruction::Mul { lhs, rhs, dst }
            | Instruction::Mod { lhs, rhs, dst }
            | Instruction::Pow { lhs, rhs, dst }
            | Instruction::ShiftRight { lhs, rhs, dst }
            | Instruction::ShiftLeft { lhs, rhs, dst }
            | Instruction::UnsignedShiftRight { lhs, rhs, dst }
            | Instruction::BitOr { lhs, rhs, dst }
            | Instruction::BitAnd { lhs, rhs, dst }
            | Instruction::BitXor { lhs, rhs, dst }
            | Instruction::In { lhs, rhs, dst }
            | Instruction::Eq { lhs, rhs, dst }
            | Instruction::StrictEq { lhs, rhs, dst }
            | Instruction::NotEq { lhs, rhs, dst }
            | Instruction::StrictNotEq { lhs, rhs, dst }
            | Instruction::GreaterThan { lhs, rhs, dst }
            | Instruction::GreaterThanOrEq { lhs, rhs, dst }
            | Instruction::LessThan { lhs, rhs, dst }
            | Instruction::LessThanOrEq { lhs, rhs, dst }
            | Instruction::InstanceOf { lhs, rhs, dst } => OperandsShape::LhsRhsDst {
                lhs: *lhs,
                rhs: *rhs,
                dst: *dst,
            },

            Instruction::InPrivate { dst, index, rhs } => OperandsShape::RhsIndexDst {
                rhs: *rhs,
                index: *index,
                dst: *dst,
            },

            Instruction::Inc { src, dst }
            | Instruction::Dec { src, dst }
            | Instruction::Move { src, dst }
            | Instruction::ToInt32 { src, dst }
            | Instruction::ToPropertyKey { src, dst } => OperandsShape::SrcDst {
                src: *src,
                dst: *dst,
            },

            Instruction::SetFunctionName {
                function,
                name,
                prefix,
            } => OperandsShape::SetFunctionName {
                function: *function,
                name: *name,
                prefix: u32::from(*prefix) as u8,
            },
            Instruction::ThisForObjectEnvironmentName { index, dst }
            | Instruction::GetFunction { index, dst }
            | Instruction::StoreLiteral { index, dst }
            | Instruction::GetArgument { index, dst } => OperandsShape::IndexDst {
                index: *index,
                dst: *dst,
            },

            Instruction::ThrowNewTypeError { message }
            | Instruction::ThrowNewReferenceError { message } => {
                OperandsShape::Message { message: *message }
            }

            Instruction::Jump { address } => OperandsShape::Address { address: *address },

            Instruction::StoreInt8 { value, dst } => OperandsShape::ValueDstI8 {
                value: *value,
                dst: *dst,
            },
            Instruction::StoreInt16 { value, dst } => OperandsShape::ValueDstI16 {
                value: *value,
                dst: *dst,
            },
            Instruction::StoreInt32 { value, dst } => OperandsShape::ValueDstI32 {
                value: *value,
                dst: *dst,
            },
            Instruction::StoreFloat { value, dst } => OperandsShape::ValueDstF32 {
                value: *value,
                dst: *dst,
            },
            Instruction::StoreDouble { value, dst } => OperandsShape::ValueDstF64 {
                value: *value,
                dst: *dst,
            },

            Instruction::StoreClassPrototype {
                dst,
                class,
                superclass,
            } => OperandsShape::DstClassSuperclass {
                dst: *dst,
                class: *class,
                superclass: *superclass,
            },

            Instruction::JumpIfTrue { address, value }
            | Instruction::JumpIfFalse { address, value }
            | Instruction::JumpIfNotUndefined { address, value }
            | Instruction::JumpIfNullOrUndefined { address, value }
            | Instruction::LogicalAnd { address, value }
            | Instruction::LogicalOr { address, value }
            | Instruction::Coalesce { address, value } => OperandsShape::AddressValue {
                address: *address,
                value: *value,
            },

            Instruction::JumpIfNotLessThan { address, lhs, rhs }
            | Instruction::JumpIfNotLessThanOrEqual { address, lhs, rhs }
            | Instruction::JumpIfNotGreaterThan { address, lhs, rhs }
            | Instruction::JumpIfNotGreaterThanOrEqual { address, lhs, rhs }
            | Instruction::JumpIfNotEqual { address, lhs, rhs } => OperandsShape::AddressLhsRhs {
                address: *address,
                lhs: *lhs,
                rhs: *rhs,
            },

            Instruction::Case {
                address,
                value,
                condition,
            } => OperandsShape::Case {
                address: *address,
                value: *value,
                condition: *condition,
            },

            Instruction::CallEval {
                argument_count,
                scope_index,
            } => OperandsShape::CallEval {
                argument_count: *argument_count,
                scope_index: *scope_index,
            },

            Instruction::CallEvalSpread { scope_index }
            | Instruction::PushScope { scope_index } => OperandsShape::ScopeIndex {
                scope_index: *scope_index,
            },

            Instruction::Call { argument_count }
            | Instruction::New { argument_count }
            | Instruction::SuperCall { argument_count } => OperandsShape::ArgumentCount {
                argument_count: *argument_count,
            },

            Instruction::DefVar { binding_index }
            | Instruction::DefEvalVar { binding_index }
            | Instruction::GetLocator { binding_index } => OperandsShape::BindingIndex {
                binding_index: *binding_index,
            },

            Instruction::DefInitVar { src, binding_index }
            | Instruction::PutLexicalValue { src, binding_index }
            | Instruction::SetName { src, binding_index } => OperandsShape::SrcBindingIndex {
                src: *src,
                binding_index: *binding_index,
            },

            Instruction::GetName { dst, binding_index }
            | Instruction::GetNameAndLocator { dst, binding_index }
            | Instruction::GetNameOrUndefined { dst, binding_index }
            | Instruction::DeleteName { dst, binding_index } => OperandsShape::DstBindingIndex {
                dst: *dst,
                binding_index: *binding_index,
            },

            Instruction::GetNameGlobal {
                dst,
                binding_index,
                ic_index,
            } => OperandsShape::GetNameGlobal {
                dst: *dst,
                binding_index: *binding_index,
                ic_index: *ic_index,
            },

            Instruction::DefineOwnPropertyByName {
                object,
                value,
                name_index,
            }
            | Instruction::SetPropertyGetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::SetPropertySetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefinePrivateField {
                object,
                value,
                name_index,
            }
            | Instruction::SetPrivateMethod {
                object,
                value,
                name_index,
            }
            | Instruction::SetPrivateSetter {
                object,
                value,
                name_index,
            }
            | Instruction::SetPrivateGetter {
                object,
                value,
                name_index,
            }
            | Instruction::PushClassPrivateGetter {
                object,
                value,
                name_index,
            }
            | Instruction::PushClassPrivateSetter {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassStaticMethodByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassMethodByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassStaticGetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassGetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassStaticSetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassSetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::SetPrivateField {
                object,
                value,
                name_index,
            }
            | Instruction::PushClassFieldPrivate {
                object,
                value,
                name_index,
            } => OperandsShape::ObjectValueName {
                object: *object,
                value: *value,
                name_index: *name_index,
            },
            Instruction::GetPrivateField {
                dst,
                object,
                name_index,
            } => OperandsShape::DstObjectName {
                dst: *dst,
                object: *object,
                name_index: *name_index,
            },
            Instruction::PushClassPrivateMethod {
                object,
                proto,
                value,
                name_index,
            } => OperandsShape::ObjectProtoValueName {
                object: *object,
                proto: *proto,
                value: *value,
                name_index: *name_index,
            },
            Instruction::ThrowMutateImmutable { index } => OperandsShape::Index { index: *index },
            Instruction::DeletePropertyByName { object, name_index }
            | Instruction::GetMethod { object, name_index } => OperandsShape::ObjectName {
                object: *object,
                name_index: *name_index,
            },
            Instruction::GetLengthProperty {
                dst,
                value,
                ic_index,
            }
            | Instruction::GetPropertyByName {
                dst,
                value,
                ic_index,
            } => OperandsShape::DstValueIc {
                dst: *dst,
                value: *value,
                ic_index: *ic_index,
            },
            Instruction::GetPropertyByNameWithThis {
                dst,
                receiver,
                value,
                ic_index,
            } => OperandsShape::DstReceiverValueIc {
                dst: *dst,
                receiver: *receiver,
                value: *value,
                ic_index: *ic_index,
            },
            Instruction::SetPropertyByName {
                value,
                object,
                ic_index,
            } => OperandsShape::ObjectValueIc {
                object: *object,
                value: *value,
                ic_index: *ic_index,
            },
            Instruction::SetPropertyByNameWithThis {
                value,
                receiver,
                object,
                ic_index,
            } => OperandsShape::ObjectReceiverValueIc {
                object: *object,
                receiver: *receiver,
                value: *value,
                ic_index: *ic_index,
            },
            Instruction::GetPropertyByValue {
                dst,
                key,
                receiver,
                object,
            }
            | Instruction::GetPropertyByValuePush {
                dst,
                key,
                receiver,
                object,
            } => OperandsShape::DstKeyReceiverObject {
                dst: *dst,
                key: *key,
                receiver: *receiver,
                object: *object,
            },
            Instruction::SetPropertyByValue {
                value,
                key,
                receiver,
                object,
            } => OperandsShape::ObjectReceiverKeyValue {
                object: *object,
                receiver: *receiver,
                key: *key,
                value: *value,
            },
            Instruction::DefineOwnPropertyByValue { value, key, object }
            | Instruction::DefineClassStaticMethodByValue { value, key, object }
            | Instruction::DefineClassMethodByValue { value, key, object }
            | Instruction::SetPropertyGetterByValue { value, key, object }
            | Instruction::DefineClassStaticGetterByValue { value, key, object }
            | Instruction::DefineClassGetterByValue { value, key, object }
            | Instruction::SetPropertySetterByValue { value, key, object }
            | Instruction::DefineClassStaticSetterByValue { value, key, object }
            | Instruction::DefineClassSetterByValue { value, key, object } => {
                OperandsShape::ObjectKeyValue {
                    object: *object,
                    key: *key,
                    value: *value,
                }
            }
            Instruction::DeletePropertyByValue { key, object } => OperandsShape::ObjectKey {
                object: *object,
                key: *key,
            },
            Instruction::CreateIteratorResult { value, done } => OperandsShape::ValueDone {
                value: *value,
                done: *done,
            },
            Instruction::SetClassPrototype {
                dst,
                prototype,
                class,
            } => OperandsShape::DstPrototypeClass {
                dst: *dst,
                prototype: *prototype,
                class: *class,
            },
            Instruction::SetHomeObject { function, home } => OperandsShape::FunctionHome {
                function: *function,
                home: *home,
            },
            Instruction::GetHomeObject { function } => OperandsShape::Function {
                function: *function,
            },
            Instruction::SetPrototype { object, prototype } => OperandsShape::ObjectPrototype {
                object: *object,
                prototype: *prototype,
            },
            Instruction::GetPrototype { object } => OperandsShape::Object { object: *object },
            Instruction::PushValueToArray { value, array } => OperandsShape::ValueArray {
                value: *value,
                array: *array,
            },
            Instruction::PushElisionToArray { array }
            | Instruction::PushIteratorToArray { array } => OperandsShape::Array { array: *array },
            Instruction::TypeOf { value }
            | Instruction::LogicalNot { value }
            | Instruction::Pos { value }
            | Instruction::Neg { value }
            | Instruction::IsObject { value }
            | Instruction::BindThisValue { value }
            | Instruction::BitNot { value } => OperandsShape::Value { value: *value },
            Instruction::ImportCall {
                specifier,
                options,
                phase,
            } => OperandsShape::SpecifierOptions {
                specifier: *specifier,
                options: *options,
                phase: *phase,
            },
            Instruction::PushClassField {
                object,
                name,
                value,
                is_anonymous_function,
            } => OperandsShape::ClassField {
                object: *object,
                name: *name,
                value: *value,
                is_anonymous_function: *is_anonymous_function,
            },
            Instruction::MaybeException {
                has_exception,
                exception,
            } => OperandsShape::MaybeException {
                has_exception: *has_exception,
                exception: *exception,
            },
            Instruction::SetAccumulator { src }
            | Instruction::PushFromRegister { src }
            | Instruction::Throw { src }
            | Instruction::SetNameByLocator { src }
            | Instruction::PushObjectEnvironment { src }
            | Instruction::CreateForInIterator { src }
            | Instruction::GetIterator { src }
            | Instruction::GetAsyncIterator { src }
            | Instruction::ValueNotNullOrUndefined { src }
            | Instruction::GeneratorYield { src }
            | Instruction::AsyncGeneratorYield { src }
            | Instruction::Await { src } => OperandsShape::Src { src: *src },
            Instruction::IteratorPush { iterator, next }
            | Instruction::IteratorPop { iterator, next } => OperandsShape::IteratorNextReg {
                iterator: *iterator,
                next: *next,
            },
            Instruction::IteratorUpdateResult { result } => {
                OperandsShape::Result { result: *result }
            }
            Instruction::SetRegisterFromAccumulator { dst }
            | Instruction::PopIntoRegister { dst }
            | Instruction::StoreZero { dst }
            | Instruction::StoreOne { dst }
            | Instruction::StoreNan { dst }
            | Instruction::StorePositiveInfinity { dst }
            | Instruction::StoreNegativeInfinity { dst }
            | Instruction::StoreNull { dst }
            | Instruction::StoreTrue { dst }
            | Instruction::StoreFalse { dst }
            | Instruction::StoreUndefined { dst }
            | Instruction::Exception { dst }
            | Instruction::This { dst }
            | Instruction::NewTarget { dst }
            | Instruction::ImportMeta { dst }
            | Instruction::CreateMappedArgumentsObject { dst }
            | Instruction::CreateUnmappedArgumentsObject { dst }
            | Instruction::RestParameterInit { dst }
            | Instruction::StoreEmptyObject { dst }
            | Instruction::IteratorDone { dst }
            | Instruction::IteratorResult { dst }
            | Instruction::IteratorStackEmpty { dst }
            | Instruction::IteratorValue { dst }
            | Instruction::StoreNewArray { dst } => OperandsShape::Dst { dst: *dst },
            Instruction::PushPrivateEnvironment {
                class,
                name_indices,
            } => OperandsShape::ClassNames {
                class: *class,
                name_indices: name_indices
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            Instruction::TemplateLookup { address, site, dst } => OperandsShape::AddressSiteDst {
                address: *address,
                site: *site,
                dst: *dst,
            },
            Instruction::JumpTable { index, addresses } => OperandsShape::JumpTable {
                index: *index,
                addresses: addresses
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            Instruction::ConcatToString { dst, values } => OperandsShape::DstValues {
                dst: *dst,
                values: values
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            Instruction::CopyDataProperties {
                object,
                source,
                excluded_keys,
            } => OperandsShape::ObjectSourceExcluded {
                object: *object,
                source: *source,
                excluded_keys: excluded_keys
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            Instruction::TemplateCreate { site, dst, values } => OperandsShape::SiteDstValues {
                site: *site,
                dst: *dst,
                values: values
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            Instruction::GetFunctionObject { function_object } => OperandsShape::FunctionObject {
                function_object: *function_object,
            },
            Instruction::StoreRegexp {
                dst,
                pattern_index,
                flags_index,
            } => OperandsShape::Regexp {
                pattern_index: *pattern_index,
                flags_index: *flags_index,
                dst: *dst,
            },

            Instruction::Reserved1
            | Instruction::Reserved2
            | Instruction::Reserved3
            | Instruction::Reserved4
            | Instruction::Reserved5
            | Instruction::Reserved6
            | Instruction::Reserved7
            | Instruction::Reserved8
            | Instruction::Reserved9
            | Instruction::Reserved10
            | Instruction::Reserved11
            | Instruction::Reserved12
            | Instruction::Reserved13
            | Instruction::Reserved14
            | Instruction::Reserved15
            | Instruction::Reserved16
            | Instruction::Reserved17
            | Instruction::Reserved18
            | Instruction::Reserved19
            | Instruction::Reserved20
            | Instruction::Reserved21
            | Instruction::Reserved22
            | Instruction::Reserved23
            | Instruction::Reserved24
            | Instruction::Reserved25
            | Instruction::Reserved26
            | Instruction::Reserved27
            | Instruction::Reserved28
            | Instruction::Reserved29
            | Instruction::Reserved30
            | Instruction::Reserved31
            | Instruction::Reserved32
            | Instruction::Reserved33
            | Instruction::Reserved34
            | Instruction::Reserved35
            | Instruction::Reserved36
            | Instruction::Reserved37
            | Instruction::Reserved38
            | Instruction::Reserved39
            | Instruction::Reserved40
            | Instruction::Reserved41
            | Instruction::Reserved42
            | Instruction::Reserved43
            | Instruction::Reserved44
            | Instruction::Reserved45
            | Instruction::Reserved46
            | Instruction::Reserved47
            | Instruction::Reserved48
            | Instruction::Reserved49
            | Instruction::Reserved50
            | Instruction::Reserved51
            | Instruction::Reserved52
            | Instruction::Reserved53
            | Instruction::Reserved54
            | Instruction::Reserved55
            | Instruction::Reserved56
            | Instruction::Reserved57
            | Instruction::Reserved58
            | Instruction::Reserved59
            | Instruction::Reserved60
            | Instruction::Reserved61 => unreachable!("Reserved opcodes are unreachable"),
        }
    }
}

impl std::fmt::Display for OperandsShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Dst { dst } => write!(f, "dst:{dst}"),
            Self::LhsRhsDst { lhs, rhs, dst } => write!(f, "lhs:{lhs}, rhs:{rhs}, dst:{dst}"),
            Self::RhsIndexDst { rhs, index, dst } => {
                write!(f, "rhs:{rhs}, index:{index}, dst:{dst}")
            }
            Self::SrcDst { src, dst } => write!(f, "src:{src}, dst:{dst}"),
            Self::SetFunctionName {
                function,
                name,
                prefix,
            } => {
                let prefix_str = match prefix {
                    1 => "prefix: get",
                    2 => "prefix: set",
                    _ => "prefix:",
                };
                write!(f, "function:{function}, name:{name}, {prefix_str}")
            }
            Self::ValueDstI8 { value, dst } => write!(f, "value:{value}, dst:{dst}"),
            Self::ValueDstI16 { value, dst } => write!(f, "value:{value}, dst:{dst}"),
            Self::ValueDstI32 { value, dst } => write!(f, "value:{value}, dst:{dst}"),
            Self::ValueDstF32 { value, dst } => write!(f, "value:{value}, dst:{dst}"),
            Self::ValueDstF64 { value, dst } => write!(f, "value:{value}, dst:{dst}"),
            Self::IndexDst { index, dst } => write!(f, "index:{index}, dst:{dst}"),
            Self::Message { message } => write!(f, "message:{message}"),
            Self::Regexp {
                pattern_index,
                flags_index,
                dst,
            } => {
                write!(f, "pattern:{pattern_index}, flags:{flags_index}, dst:{dst}")
            }
            Self::Address { address } => write!(f, "address:{address}"),
            Self::AddressValue { address, value } => write!(f, "value:{value}, address:{address}"),
            Self::AddressLhsRhs { address, lhs, rhs } => {
                write!(f, "lhs:{lhs}, rhs:{rhs}, address:{address}")
            }
            Self::Case {
                address,
                value,
                condition,
            } => {
                write!(f, "value:{value}, condition:{condition}, address:{address}")
            }
            Self::CallEval {
                argument_count,
                scope_index,
            } => {
                write!(
                    f,
                    "argument_count:{argument_count}, scope_index:{scope_index}"
                )
            }
            Self::ScopeIndex { scope_index } => write!(f, "scope_index:{scope_index}"),
            Self::ArgumentCount { argument_count } => write!(f, "argument_count:{argument_count}"),
            Self::BindingIndex { binding_index } => write!(f, "binding_index:{binding_index}"),
            Self::SrcBindingIndex { src, binding_index } => {
                write!(f, "src:{src}, binding_index:{binding_index}")
            }
            Self::DstBindingIndex { dst, binding_index } => {
                write!(f, "dst:{dst}, binding_index:{binding_index}")
            }
            Self::GetNameGlobal {
                dst,
                binding_index,
                ic_index,
            } => {
                write!(
                    f,
                    "dst:{dst}, binding_index:{binding_index}, ic_index:{ic_index}"
                )
            }
            Self::ObjectValueName {
                object,
                value,
                name_index,
            } => {
                write!(f, "object:{object}, value:{value}, name_index:{name_index}")
            }
            Self::DstObjectName {
                dst,
                object,
                name_index,
            } => {
                write!(f, "dst:{dst}, object:{object}, name_index:{name_index}")
            }
            Self::ObjectProtoValueName {
                object,
                proto,
                value,
                name_index,
            } => {
                write!(
                    f,
                    "object:{object}, proto:{proto}, value:{value}, name_index:{name_index}"
                )
            }
            Self::Index { index } => write!(f, "index:{index}"),
            Self::ObjectName { object, name_index } => {
                write!(f, "object:{object}, name_index:{name_index}")
            }
            Self::DstValueIc {
                dst,
                value,
                ic_index,
            } => write!(f, "dst:{dst}, value:{value}, ic:{ic_index}"),
            Self::DstReceiverValueIc {
                dst,
                receiver,
                value,
                ic_index,
            } => {
                write!(
                    f,
                    "dst:{dst}, receiver:{receiver}, value:{value}, ic:{ic_index}"
                )
            }
            Self::ObjectValueIc {
                object,
                value,
                ic_index,
            } => write!(f, "object:{object}, value:{value}, ic:{ic_index}"),
            Self::ObjectReceiverValueIc {
                object,
                receiver,
                value,
                ic_index,
            } => {
                write!(
                    f,
                    "object:{object}, receiver:{receiver}, value:{value}, ic:{ic_index}"
                )
            }
            Self::DstKeyReceiverObject {
                dst,
                key,
                receiver,
                object,
            } => {
                write!(
                    f,
                    "dst:{dst}, object:{object}, receiver:{receiver}, key:{key}"
                )
            }
            Self::ObjectReceiverKeyValue {
                object,
                receiver,
                key,
                value,
            } => {
                write!(
                    f,
                    "object:{object}, receiver:{receiver}, key:{key}, value:{value}"
                )
            }
            Self::ObjectKeyValue { object, key, value } => {
                write!(f, "object:{object}, key:{key}, value:{value}")
            }
            Self::ObjectKey { object, key } => write!(f, "object:{object}, key:{key}"),
            Self::ValueDone { value, done } => write!(f, "value:{value}, done:{done}"),
            Self::DstClassSuperclass {
                dst,
                class,
                superclass,
            } => {
                write!(f, "dst:{dst}, class:{class}, superclass:{superclass}")
            }
            Self::DstPrototypeClass {
                dst,
                prototype,
                class,
            } => {
                write!(f, "dst:{dst}, prototype:{prototype}, class:{class}")
            }
            Self::FunctionHome { function, home } => write!(f, "function:{function}, home:{home}"),
            Self::Function { function } => write!(f, "function:{function}"),
            Self::ObjectPrototype { object, prototype } => {
                write!(f, "object:{object}, prototype:{prototype}")
            }
            Self::Object { object } => write!(f, "object:{object}"),
            Self::ValueArray { value, array } => write!(f, "value:{value}, array:{array}"),
            Self::Array { array } => write!(f, "array:{array}"),
            Self::Value { value } => write!(f, "value:{value}"),
            Self::SpecifierOptions {
                specifier,
                options,
                phase,
            } => {
                write!(
                    f,
                    "specifier:{specifier}, options:{options}, options:{phase}"
                )
            }
            Self::ClassField {
                object,
                name,
                value,
                is_anonymous_function,
            } => {
                write!(
                    f,
                    "object:{object}, value:{value}, name:{name}, is_anonymous_function:{is_anonymous_function}"
                )
            }
            Self::MaybeException {
                has_exception,
                exception,
            } => {
                write!(f, "has_exception:{has_exception}, exception:{exception}")
            }
            Self::Src { src } => write!(f, "src:{src}"),
            Self::IteratorNextReg { iterator, next } => {
                write!(f, "iterator:{iterator}, next:{next}")
            }
            Self::Result { result } => write!(f, "result:{result}"),
            Self::ResumeKindValue { resume_kind, value } => {
                write!(f, "resume_kind:{resume_kind}, value:{value}")
            }
            Self::ValueCalled { value, called } => write!(f, "value:{value}, called:{called}"),
            Self::SrcConfigurableName {
                src,
                configurable,
                name_index,
            } => {
                write!(
                    f,
                    "src:{src}, configurable:{configurable}, name_index:{name_index}"
                )
            }
            Self::ConfigurableName {
                configurable,
                name_index,
            } => {
                write!(f, "configurable:{configurable}, name_index:{name_index}")
            }
            Self::ClassNames {
                class,
                name_indices,
            } => write!(f, "class:{class}, names:{name_indices:?}"),
            Self::AddressSiteDst { address, site, dst } => {
                write!(f, "address:{address}, site:{site}, dst:{dst}")
            }
            Self::JumpTable { index, addresses } => {
                use itertools::Itertools;
                write!(
                    f,
                    "index:{index}, jump_table:({})",
                    addresses.iter().format(", ")
                )
            }
            Self::DstValues { dst, values } => write!(f, "dst:{dst}, values:{values:?}"),
            Self::ObjectSourceExcluded {
                object,
                source,
                excluded_keys,
            } => {
                write!(
                    f,
                    "object:{object}, source:{source}, excluded_keys:{excluded_keys:?}"
                )
            }
            Self::SiteDstValues { site, dst, values } => {
                write!(f, "site:{site}, dst:{dst}, values:{values:?}")
            }
            Self::FunctionObject { function_object } => {
                write!(f, "function_object:{function_object}")
            }
        }
    }
}
