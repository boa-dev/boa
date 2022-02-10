use crate::{
    builtins::Array,
    environments::DeclarativeEnvironment,
    gc::{Finalize, Trace},
    object::{FunctionBuilder, JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyKey},
    symbol::{self, WellKnownSymbols},
    syntax::ast::node::FormalParameter,
    Context, JsValue,
};
use gc::Gc;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Trace, Finalize)]
pub struct MappedArguments(JsObject);

impl MappedArguments {
    pub(crate) fn parameter_map(&self) -> JsObject {
        self.0.clone()
    }
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Arguments {
    Unmapped,
    Mapped(MappedArguments),
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
        .expect("DefinePropertyOrThrow must not fail per the spec");

        // 5. Let index be 0.
        // 6. Repeat, while index < len,
        for (index, value) in arguments_list.iter().cloned().enumerate() {
            // a. Let val be argumentsList[index].
            // b. Perform ! CreateDataPropertyOrThrow(obj, ! ToString(𝔽(index)), val).
            obj.create_data_property_or_throw(index, value, context)
                .expect("CreateDataPropertyOrThrow must not fail per the spec");

            // c. Set index to index + 1.
        }

        // 7. Perform ! DefinePropertyOrThrow(obj, @@iterator, PropertyDescriptor {
        // [[Value]]: %Array.prototype.values%, [[Writable]]: true, [[Enumerable]]: false,
        // [[Configurable]]: true }).
        obj.define_property_or_throw(
            symbol::WellKnownSymbols::iterator(),
            PropertyDescriptor::builder()
                .value(Array::values_intrinsic(context))
                .writable(true)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("DefinePropertyOrThrow must not fail per the spec");

        let throw_type_error = context.intrinsics().throw_type_error();

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
        .expect("DefinePropertyOrThrow must not fail per the spec");

        // 9. Return obj.
        obj
    }

    /// Creates a new mapped Arguments exotic object.
    ///
    /// <https://tc39.es/ecma262/#sec-createmappedargumentsobject>
    pub(crate) fn create_mapped_arguments_object(
        func: &JsObject,
        formals: &[FormalParameter],
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

        // 10. Let map be ! OrdinaryObjectCreate(null).
        let map = JsObject::empty();

        // 11. Set obj.[[ParameterMap]] to map.
        let obj = JsObject::from_proto_and_data(
            context.standard_objects().object_object().prototype(),
            ObjectData::arguments(Self::Mapped(MappedArguments(map.clone()))),
        );

        // 14. Let index be 0.
        // 15. Repeat, while index < len,
        for (index, val) in arguments_list.iter().cloned().enumerate() {
            // a. Let val be argumentsList[index].
            // b. Perform ! CreateDataPropertyOrThrow(obj, ! ToString(𝔽(index)), val).
            obj.create_data_property_or_throw(index, val, context)
                .expect("CreateDataPropertyOrThrow must not fail per the spec");
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
        .expect("DefinePropertyOrThrow must not fail per the spec");

        // 17. Let mappedNames be a new empty List.
        // 12. Let parameterNames be the BoundNames of formals.
        // 13. Let numberOfParameters be the number of elements in parameterNames.
        // 18. Set index to numberOfParameters - 1.
        // 19. Repeat, while index ≥ 0,
        // a. Let name be parameterNames[index].

        // Get the mapping of the environment binding index to the index of the object property.
        // Parameters can be defined multiple times, so the mapping is not one to one.
        let mut bindings = FxHashMap::default();
        let mut property_index = 0;
        'outer: for formal in formals {
            for name in formal.names() {
                if property_index >= len {
                    break 'outer;
                }
                let binding_index = bindings.len() + 1;
                let entry = bindings
                    .entry(name)
                    .or_insert((binding_index, property_index));
                entry.1 = property_index;
                property_index += 1;
            }
        }
        for (binding_index, property_index) in bindings.values() {
            // 1. Let g be MakeArgGetter(name, env).
            // https://tc39.es/ecma262/#sec-makearggetter
            let g = {
                // 2. Let getter be ! CreateBuiltinFunction(getterClosure, 0, "", « »).
                // 3. NOTE: getter is never directly accessible to ECMAScript code.
                // 4. Return getter.
                FunctionBuilder::closure_with_captures(
                    context,
                    // 1. Let getterClosure be a new Abstract Closure with no parameters that captures
                    // name and env and performs the following steps when called:
                    |_, _, captures, _| Ok(captures.0.get(captures.1)),
                    (env.clone(), *binding_index),
                )
                .length(0)
                .build()
            };
            // 2. Let p be MakeArgSetter(name, env).
            // https://tc39.es/ecma262/#sec-makeargsetter
            let p = {
                // 2. Let setter be ! CreateBuiltinFunction(setterClosure, 1, "", « »).
                // 3. NOTE: setter is never directly accessible to ECMAScript code.
                // 4. Return setter.
                FunctionBuilder::closure_with_captures(
                    context,
                    // 1. Let setterClosure be a new Abstract Closure with parameters (value) that captures
                    // name and env and performs the following steps when called:
                    |_, args, captures, _| {
                        let value = args.get(0).cloned().unwrap_or_default();
                        captures.0.set(captures.1, value);
                        Ok(JsValue::undefined())
                    },
                    (env.clone(), *binding_index),
                )
                .length(1)
                .build()
            };

            // 3. Perform map.[[DefineOwnProperty]](! ToString(𝔽(index)), PropertyDescriptor {
            // [[Set]]: p, [[Get]]: g, [[Enumerable]]: false, [[Configurable]]: true }).
            map.__define_own_property__(
                PropertyKey::from(*property_index),
                PropertyDescriptor::builder()
                    .set(p)
                    .get(g)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            )
            .expect("[[DefineOwnProperty]] must not fail per the spec");
        }

        // 20. Perform ! DefinePropertyOrThrow(obj, @@iterator, PropertyDescriptor {
        // [[Value]]: %Array.prototype.values%, [[Writable]]: true, [[Enumerable]]: false,
        // [[Configurable]]: true }).
        obj.define_property_or_throw(
            WellKnownSymbols::iterator(),
            PropertyDescriptor::builder()
                .value(Array::values_intrinsic(context))
                .writable(true)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("DefinePropertyOrThrow must not fail per the spec");

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
        .expect("DefinePropertyOrThrow must not fail per the spec");

        // 22. Return obj.
        obj
    }
}
