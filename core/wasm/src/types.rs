//! `WebAssembly` type definitions.

/// A value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValType {
    /// 32-bit integer.
    I32,
    /// 64-bit integer.
    I64,
    /// 32-bit float.
    F32,
    /// 64-bit float.
    F64,
    /// 128-bit SIMD vector.
    V128,
    /// Function reference.
    FuncRef,
    /// External reference.
    ExternRef,
}

/// A reference type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefType {
    /// Function reference.
    FuncRef,
    /// External reference.
    ExternRef,
}

/// A function type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    /// Parameter types.
    pub params: Vec<ValType>,
    /// Result types.
    pub results: Vec<ValType>,
}

/// Size limits for tables and memories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Minimum size.
    pub min: u32,
    /// Maximum size.
    pub max: Option<u32>,
}

/// A table type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableType {
    /// Element reference type.
    pub element: RefType,
    /// Size limits.
    pub limits: Limits,
}

/// A memory type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemType {
    /// Size limits.
    pub limits: Limits,
}

/// Mutability of a global.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mut {
    /// Immutable.
    Const,
    /// Mutable.
    Var,
}

/// A global type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalType {
    /// Value type.
    pub val_type: ValType,
    /// Mutability.
    pub mutability: Mut,
}

/// An import descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportDesc {
    /// Function import (type index).
    Func(u32),
    /// Table import.
    Table(TableType),
    /// Memory import.
    Memory(MemType),
    /// Global import.
    Global(GlobalType),
}

/// An import entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    /// Module name.
    pub module: String,
    /// Field name.
    pub name: String,
    /// Descriptor.
    pub desc: ImportDesc,
}

/// An export descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportDesc {
    /// Function export (index).
    Func(u32),
    /// Table export (index).
    Table(u32),
    /// Memory export (index).
    Memory(u32),
    /// Global export (index).
    Global(u32),
}

/// An export entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    /// Exported name.
    pub name: String,
    /// Descriptor.
    pub desc: ExportDesc,
}

/// A global definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Global {
    /// Global type.
    pub global_type: GlobalType,
    /// Init expression bytes.
    pub init_expr: Vec<u8>,
}

/// A local variable declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Local {
    /// Count.
    pub count: u32,
    /// Type.
    pub val_type: ValType,
}

/// A function body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncBody {
    /// Locals.
    pub locals: Vec<Local>,
    /// Raw instruction bytes.
    pub code: Vec<u8>,
}

/// Element segment mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElemMode {
    /// Passive.
    Passive,
    /// Active with table index and offset.
    Active {
        /// Table index.
        table_idx: u32,
        /// Offset expression bytes.
        offset_expr: Vec<u8>,
    },
    /// Declarative.
    Declarative,
}

/// Element initialization data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElemInit {
    /// Function indices.
    FuncIndices(Vec<u32>),
    /// Init expression bytes.
    Expressions(Vec<Vec<u8>>),
}

/// An element segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// Reference type.
    pub ref_type: RefType,
    /// Mode.
    pub mode: ElemMode,
    /// Init data.
    pub init: ElemInit,
}

/// Data segment mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataMode {
    /// Passive.
    Passive,
    /// Active with memory index and offset.
    Active {
        /// Memory index.
        mem_idx: u32,
        /// Offset expression bytes.
        offset_expr: Vec<u8>,
    },
}

/// A data segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data {
    /// Mode.
    pub mode: DataMode,
    /// Raw bytes.
    pub data: Vec<u8>,
}

/// A custom section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomSection {
    /// Section name.
    pub name: String,
    /// Payload bytes.
    pub data: Vec<u8>,
}
