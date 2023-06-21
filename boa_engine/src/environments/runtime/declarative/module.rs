use std::cell::Cell;

use boa_ast::expression::Identifier;
use boa_gc::{Finalize, GcRefCell, Trace};

use crate::{module::Module, JsValue};

/// Type of accessor used to access an indirect binding.
#[derive(Debug, Clone, Copy)]
enum BindingAccessor {
    Identifier(Identifier),
    Index(u32),
}

impl crate::snapshot::Serialize for BindingAccessor {
    fn serialize(
        &self,
        s: &mut crate::snapshot::SnapshotSerializer,
    ) -> crate::snapshot::SnapshotResult<()> {
        match self {
            BindingAccessor::Identifier(i) => {
                s.write_u8(b'I')?;
                i.serialize(s)?;
            }
            BindingAccessor::Index(i) => {
                s.write_u8(b'X')?;
                i.serialize(s)?;
            }
        }
        Ok(())
    }
}

/// An indirect reference to a binding inside an environment.
#[derive(Clone, Debug, Trace, Finalize)]
struct IndirectBinding {
    module: Module,
    #[unsafe_ignore_trace]
    accessor: Cell<BindingAccessor>,
}

impl crate::snapshot::Serialize for IndirectBinding {
    fn serialize(
        &self,
        s: &mut crate::snapshot::SnapshotSerializer,
    ) -> crate::snapshot::SnapshotResult<()> {
        self.module.serialize(s)?;
        self.accessor.serialize(s)?;
        Ok(())
    }
}

/// The type of binding a [`ModuleEnvironment`] can contain.
#[derive(Clone, Debug, Trace, Finalize)]
enum BindingType {
    Direct(Option<JsValue>),
    Indirect(IndirectBinding),
}

impl crate::snapshot::Serialize for BindingType {
    fn serialize(
        &self,
        s: &mut crate::snapshot::SnapshotSerializer,
    ) -> crate::snapshot::SnapshotResult<()> {
        match self {
            BindingType::Direct(v) => {
                s.write_u8(b'D')?;
                v.serialize(s)?;
            }
            BindingType::Indirect(i) => {
                s.write_u8(b'I')?;
                i.serialize(s)?;
            }
        }
        Ok(())
    }
}

/// A [**Module Environment Record**][spec].
///
/// Module environments allow referencing bindings inside other environments, in addition
/// to the usual declarative environment functionality.
///
///
/// [spec]: https://tc39.es/ecma262/#sec-module-environment-records
#[derive(Debug, Trace, Finalize)]
pub(crate) struct ModuleEnvironment {
    bindings: GcRefCell<Vec<BindingType>>,
}

impl crate::snapshot::Serialize for ModuleEnvironment {
    fn serialize(
        &self,
        s: &mut crate::snapshot::SnapshotSerializer,
    ) -> crate::snapshot::SnapshotResult<()> {
        self.bindings.borrow().serialize(s)?;
        Ok(())
    }
}

impl ModuleEnvironment {
    /// Creates a new `LexicalEnvironment`.
    pub(crate) fn new(bindings: u32) -> Self {
        Self {
            bindings: GcRefCell::new(vec![BindingType::Direct(None); bindings as usize]),
        }
    }

    /// Get the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[track_caller]
    pub(crate) fn get(&self, index: u32) -> Option<JsValue> {
        let bindings = self.bindings.borrow();

        match &bindings[index as usize] {
            BindingType::Direct(v) => v.clone(),
            BindingType::Indirect(IndirectBinding { module, accessor }) => {
                let env = module.environment()?;

                match accessor.get() {
                    BindingAccessor::Identifier(name) => {
                        let index = env
                            .compile_env()
                            .borrow()
                            .get_binding(name)
                            .expect("linking must ensure the binding exists");

                        let value = env.get(index.binding_index)?;

                        accessor.set(BindingAccessor::Index(index.binding_index));

                        Some(value)
                    }
                    BindingAccessor::Index(index) => env.get(index),
                }
            }
        }
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: u32, value: JsValue) {
        let mut bindings = self.bindings.borrow_mut();

        match &mut bindings[index as usize] {
            BindingType::Direct(v) => *v = Some(value),
            BindingType::Indirect(_) => {
                panic!("cannot modify indirect references to other environments")
            }
        }
    }

    /// Creates an indirect binding reference to another environment binding.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set_indirect(
        &self,
        index: u32,
        target_module: Module,
        target_binding: Identifier,
    ) {
        let mut bindings = self.bindings.borrow_mut();

        bindings[index as usize] = BindingType::Indirect(IndirectBinding {
            module: target_module,
            accessor: Cell::new(BindingAccessor::Identifier(target_binding)),
        });
    }
}
