use crate::{
    environments::DeclarativeEnvironment,
    object::{JsObject, ObjectData},
    property::PropertyDescriptor,
    symbol::{self, WellKnownSymbols},
    Context, JsValue,
};
use boa_ast::{function::FormalParameterList, operations::bound_names};
use boa_gc::{Finalize, Gc, Trace};
use rustc_hash::FxHashMap;

/// `ParameterMap` represents the `[[ParameterMap]]` internal slot on a Arguments exotic object.
///
/// This struct stores all the data to access mapped function parameters in their environment.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct ParameterMap {
    binding_indices: Vec<Option<usize>>,
    environment: Gc<DeclarativeEnvironment>,
}

impl ParameterMap {
    /// Deletes the binding with the given index from the parameter map.
    pub(crate) fn delete(&mut self, index: usize) {
        if let Some(binding) = self.binding_indices.get_mut(index) {
            *binding = None;
        }
    }

    /// Get the value of the binding at the given index from the function environment.
    ///
    /// Note: This function is the abstract getter closure described in 10.4.4.7.1 `MakeArgGetter ( name, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-makearggetter
    pub(crate) fn get(&self, index: usize) -> Option<JsValue> {
        if let Some(Some(binding_index)) = self.binding_indices.get(index) {
            return Some(self.environment.get(*binding_index));
        }
        None
    }

    /// Set the value of the binding at the given index in the function environment.
    ///
    /// Note: This function is the abstract setter closure described in 10.4.4.7.2 `MakeArgSetter ( name, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-makeargsetter
    pub(crate) fn set(&self, index: usize, value: &JsValue) {
        if let Some(Some(binding_index)) = self.binding_indices.get(index) {
            self.environment.set(*binding_index, value.clone());
        }
    }
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Arguments {
    Unmapped,
    Mapped(ParameterMap),
}

impl Arguments {
    /// Creates a new unmapped Arguments ordinary object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createunmappedargumentsobject
    pub(crate) fn create_unmapped_arguments_object(
        arguments_list: &[JsValue],
        context: &mut Context,
    ) -> JsObject {
        // 1. Let len be the number of elements in argumentsList.
        let len = arguments_list.len();

        // 2. Let obj be ! OrdinaryObjectCreate(%Object.prototype%, « [[ParameterMap]] »).
        let obj = context.construct_object();

        // 3. Set obj.[[ParameterMap]] to undefined.
        // skipped because the `Arguments` enum ensures ordinary argument objects don't have a `[[ParameterMap]]`
        obj.borrow_mut().data = ObjectData::arguments(Self::Unmapped);

        // 4. Perform DefinePropertyOrThrow(obj, "length", PropertyDescriptor { [[Value]]: 𝔽(len),
        // [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        obj.define_property_or_throw(
            "length",
            PropertyDescriptor::builder()
                .value(len)
                .writable(true)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("Defining new own properties for a new ordinary object cannot fail");

        // 5. Let index be 0.
        // 6. Repeat, while index < len,
        for (index, value) in arguments_list.iter().cloned().enumerate() {
            // a. Let val be argumentsList[index].
            // b. Perform ! CreateDataPropertyOrThrow(obj, ! ToString(𝔽(index)), val).
            obj.create_data_property_or_throw(index, value, context)
                .expect("Defining new own properties for a new ordinary object cannot fail");

            // c. Set index to index + 1.
        }

        // 7. Perform ! DefinePropertyOrThrow(obj, @@iterator, PropertyDescriptor {
        // [[Value]]: %Array.prototype.values%, [[Writable]]: true, [[Enumerable]]: false,
        // [[Configurable]]: true }).
        let values_function = context.intrinsics().objects().array_prototype_values();
        obj.define_property_or_throw(
            symbol::WellKnownSymbols::iterator(),
            PropertyDescriptor::builder()
                .value(values_function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("Defining new own properties for a new ordinary object cannot fail");

        let throw_type_error = context.intrinsics().objects().throw_type_error();

        // 8. Perform ! DefinePropertyOrThrow(obj, "callee", PropertyDescriptor {
        // [[Get]]: %ThrowTypeError%, [[Set]]: %ThrowTypeError%, [[Enumerable]]: false,
        // [[Configurable]]: false }).
        obj.define_property_or_throw(
            "callee",
            PropertyDescriptor::builder()
                .get(throw_type_error.clone())
                .set(throw_type_error)
                .enumerable(false)
                .configurable(false),
            context,
        )
        .expect("Defining new own properties for a new ordinary object cannot fail");

        // 9. Return obj.
        obj
    }

    /// Creates a new mapped Arguments exotic object.
    ///
    /// <https://tc39.es/ecma262/#sec-createmappedargumentsobject>
    pub(crate) fn create_mapped_arguments_object(
        func: &JsObject,
        formals: &FormalParameterList,
        arguments_list: &[JsValue],
        env: &Gc<DeclarativeEnvironment>,
        context: &mut Context,
    ) -> JsObject {
        // 1. Assert: formals does not contain a rest parameter, any binding patterns, or any initializers.
        // It may contain duplicate identifiers.
        // 2. Let len be the number of elements in argumentsList.
        let len = arguments_list.len();

        // 3. Let obj be ! MakeBasicObject(« [[Prototype]], [[Extensible]], [[ParameterMap]] »).
        // 4. Set obj.[[GetOwnProperty]] as specified in 10.4.4.1.
        // 5. Set obj.[[DefineOwnProperty]] as specified in 10.4.4.2.
        // 6. Set obj.[[Get]] as specified in 10.4.4.3.
        // 7. Set obj.[[Set]] as specified in 10.4.4.4.
        // 8. Set obj.[[Delete]] as specified in 10.4.4.5.
        // 9. Set obj.[[Prototype]] to %Object.prototype%.

        // Section 17-19 are done first, for easier object creation in 11.
        //
        // The section 17-19 differs from the spec, due to the way the runtime environments work.
        //
        // This section creates getters and setters for all mapped arguments.
        // Getting and setting values on the `arguments` object will actually access the bindings in the environment:
        // ```
        // function f(a) {console.log(a); arguments[0] = 1; console.log(a)};
        // f(0) // 0, 1
        // ```
        //
        // The spec assumes, that identifiers are used at runtime to reference bindings in the environment.
        // We use indices to access environment bindings at runtime.
        // To map to function parameters to binding indices, we use the fact, that bindings in a
        // function environment start with all of the arguments in order:
        // `function f (a,b,c)`
        // | binding index | `arguments` property key | identifier |
        // | 0             | 0                        | a          |
        // | 1             | 1                        | b          |
        // | 2             | 2                        | c          |
        //
        // Notice that the binding index does not correspond to the argument index:
        // `function f (a,a,b)` => binding indices 0 (a), 1 (b), 2 (c)
        // | binding index | `arguments` property key | identifier |
        // | -             | 0                        | -          |
        // | 0             | 1                        | a          |
        // | 1             | 2                        | b          |
        // While the `arguments` object contains all arguments, they must not be all bound.
        // In the case of duplicate parameter names, the last one is bound as the environment binding.
        //
        // The following logic implements the steps 17-19 adjusted for our environment structure.

        let mut bindings = FxHashMap::default();
        let mut property_index = 0;
        for name in bound_names(formals) {
            if property_index >= len {
                break;
            }
            let binding_index = bindings.len() + 1;
            let entry = bindings
                .entry(name)
                .or_insert((binding_index, property_index));
            entry.1 = property_index;
            property_index += 1;
        }

        let mut map = ParameterMap {
            binding_indices: vec![None; property_index],
            environment: env.clone(),
        };

        for (binding_index, property_index) in bindings.values() {
            map.binding_indices[*property_index] = Some(*binding_index);
        }

        // 11. Set obj.[[ParameterMap]] to map.
        let obj = JsObject::from_proto_and_data(
            context.intrinsics().constructors().object().prototype(),
            ObjectData::arguments(Self::Mapped(map)),
        );

        // 14. Let index be 0.
        // 15. Repeat, while index < len,
        for (index, val) in arguments_list.iter().cloned().enumerate() {
            // a. Let val be argumentsList[index].
            // b. Perform ! CreateDataPropertyOrThrow(obj, ! ToString(𝔽(index)), val).
            // Note: Insert is used here because `CreateDataPropertyOrThrow` would cause a panic while executing
            // exotic argument object set methods before the variables in the environment are initialized.
            obj.insert(
                index,
                PropertyDescriptor::builder()
                    .value(val)
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
            );
            // c. Set index to index + 1.
        }

        // 16. Perform ! DefinePropertyOrThrow(obj, "length", PropertyDescriptor { [[Value]]: 𝔽(len),
        // [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        obj.define_property_or_throw(
            "length",
            PropertyDescriptor::builder()
                .value(len)
                .writable(true)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("Defining new own properties for a new ordinary object cannot fail");

        // 20. Perform ! DefinePropertyOrThrow(obj, @@iterator, PropertyDescriptor {
        // [[Value]]: %Array.prototype.values%, [[Writable]]: true, [[Enumerable]]: false,
        // [[Configurable]]: true }).
        let values_function = context.intrinsics().objects().array_prototype_values();
        obj.define_property_or_throw(
            WellKnownSymbols::iterator(),
            PropertyDescriptor::builder()
                .value(values_function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("Defining new own properties for a new ordinary object cannot fail");

        // 21. Perform ! DefinePropertyOrThrow(obj, "callee", PropertyDescriptor {
        // [[Value]]: func, [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        obj.define_property_or_throw(
            "callee",
            PropertyDescriptor::builder()
                .value(func.clone())
                .writable(true)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("Defining new own properties for a new ordinary object cannot fail");

        // 22. Return obj.
        obj
    }
}
