//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//! A realm is represented in this implementation as a Realm struct with the fields specified from the spec.

use crate::{
    bytecompiler::EnvironmentStack,
    gc::{Finalize, Gc, Trace},
    object::{JsObject, ObjectData, PropertyMap},
    BoaProfiler, JsValue,
};
use gc::GcCell;
use std::borrow::Borrow;

/// Representation of a Realm.
///
/// In the specification these are called Realm Records.
#[derive(Debug)]
pub struct Realm {
    pub global_object: JsObject,
    pub(crate) global_bindings: PropertyMap,
    pub(crate) environments: DeclarativeEnvironmentStack,
    pub(crate) compile_env: EnvironmentStack,
}

impl Realm {
    #[allow(clippy::field_reassign_with_default)]
    pub fn create() -> Self {
        let _timer = BoaProfiler::global().start_event("Realm::create", "realm");
        // Create brand new global object
        // Global has no prototype to pass None to new_obj
        // Allow identification of the global object easily
        let global_object = JsObject::from_proto_and_data(None, ObjectData::global());

        Self {
            global_object,
            global_bindings: PropertyMap::default(),
            environments: DeclarativeEnvironmentStack::new(),
            compile_env: EnvironmentStack::new(),
        }
    }
}

#[derive(Clone, Debug, Trace, Finalize)]
pub struct DeclarativeEnvironmentStack {
    pub(crate) stack: Vec<Gc<DeclarativeEnvironment>>,
}

impl DeclarativeEnvironmentStack {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            stack: vec![Gc::new(DeclarativeEnvironment {
                bindings: GcCell::new(vec![None; 100]),
                function: None,
            })],
        }
    }

    #[inline]
    pub(crate) fn get_last_this(&self) -> Option<JsValue> {
        for env in self.stack.iter().rev() {
            if let Some(function_env) = env.function.borrow() {
                return Some(function_env.this_value.clone());
            }
        }
        None
    }

    #[inline]
    pub(crate) fn push_declarative(&mut self, num_bindings: usize) {
        self.stack.push(Gc::new(DeclarativeEnvironment {
            bindings: GcCell::new(vec![None; num_bindings]),
            function: None,
        }));
    }

    #[inline]
    pub(crate) fn push_function(
        &mut self,
        num_bindings: usize,
        this_value: JsValue,
        lexical: bool,
        function: JsObject,
        home_object: JsValue,
        new_target: JsValue,
    ) {
        self.stack.push(Gc::new(DeclarativeEnvironment {
            bindings: GcCell::new(vec![None; num_bindings]),
            function: Some(FunctionEnvironment {
                this_value,
                lexical,
                function,
                home_object,
                new_target,
            }),
        }));
    }

    #[inline]
    pub(crate) fn pop(&mut self) {
        self.stack.pop();
    }

    #[inline]
    pub(crate) fn current(&mut self) -> Gc<DeclarativeEnvironment> {
        self.stack.last().expect("environment must exist").clone()
    }

    #[inline]
    pub(crate) fn get_value_optional(
        &self,
        environment_index: usize,
        binding_index: usize,
    ) -> Option<JsValue> {
        self.stack
            .get(environment_index)
            .expect("environment_index out of range")
            .bindings
            .borrow()
            .get(binding_index)
            .expect("binding_index out of range")
            .clone()
    }

    #[inline]
    pub(crate) fn put_value(
        &mut self,
        environment_index: usize,
        binding_index: usize,
        value: JsValue,
    ) {
        let mut bindings = self
            .stack
            .get(environment_index)
            .expect("environment_index out of range")
            .bindings
            .borrow_mut();
        let l = bindings.len();
        let binding = bindings.get_mut(binding_index).unwrap_or_else(|| {
            panic!(
                "binding index out of range: env: {} binding: {}, b: {:?}",
                environment_index, binding_index, l
            )
        });
        *binding = Some(value);
    }

    #[inline]
    pub(crate) fn put_value_if_initialized(
        &mut self,
        environment_index: usize,
        binding_index: usize,
        value: JsValue,
    ) -> bool {
        let mut bindings = self
            .stack
            .get(environment_index)
            .expect("environment_index out of range")
            .bindings
            .borrow_mut();
        let l = bindings.len();
        let binding = bindings.get_mut(binding_index).unwrap_or_else(|| {
            panic!(
                "binding index out of range: env: {} binding: {}, b: {:?}",
                environment_index, binding_index, l
            )
        });
        if binding.is_none() {
            false
        } else {
            *binding = Some(value);
            true
        }
    }

    #[inline]
    pub(crate) fn put_value_if_uninitialized(
        &mut self,
        environment_index: usize,
        binding_index: usize,
        value: JsValue,
    ) {
        let mut bindings = self
            .stack
            .get(environment_index)
            .expect("environment_index out of range")
            .bindings
            .borrow_mut();
        let l = bindings.len();
        let binding = bindings.get_mut(binding_index).unwrap_or_else(|| {
            panic!(
                "binding index out of range: env: {} binding: {}, b: {:?}",
                environment_index, binding_index, l
            )
        });
        if binding.is_none() {
            *binding = Some(value);
        }
    }
}

#[derive(Debug, Trace, Finalize)]
pub(crate) struct DeclarativeEnvironment {
    bindings: GcCell<Vec<Option<JsValue>>>,
    function: Option<FunctionEnvironment>,
}

impl DeclarativeEnvironment {
    #[inline]
    pub(crate) fn get(&self, index: usize) -> JsValue {
        self.bindings
            .borrow()
            .get(index)
            .expect("binding index out of range")
            .clone()
            .expect("get called on uninitialized binding")
    }

    #[inline]
    pub(crate) fn set(&self, index: usize, value: JsValue) {
        let mut bindings = self.bindings.borrow_mut();
        let binding = bindings.get_mut(index).expect("binding index out of range");
        assert!(!binding.is_none(), "set called on uninitialized binding");
        *binding = Some(value);
    }
}

#[derive(Debug, Trace, Finalize)]
struct FunctionEnvironment {
    /// This is the this value used for this invocation of the function.
    this_value: JsValue,
    /// If the value is "lexical", this is an ArrowFunction and does not have a local this value.
    lexical: bool,
    /// The function object whose invocation caused this Environment Record to be created.
    function: JsObject,
    /// If the associated function has super property accesses and is not an ArrowFunction,
    /// `[[HomeObject]]` is the object that the function is bound to as a method.
    /// The default value for `[[HomeObject]]` is undefined.
    home_object: JsValue,
    /// If this Environment Record was created by the `[[Construct]]` internal method,
    /// `[[NewTarget]]` is the value of the `[[Construct]]` newTarget parameter.
    /// Otherwise, its value is undefined.
    new_target: JsValue,
}
