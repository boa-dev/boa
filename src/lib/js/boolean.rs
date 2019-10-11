use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        object::{internal_methods_trait::ObjectInternalMethods, Object, ObjectKind, PROTOTYPE},
        value::{to_value, ResultValue, Value, ValueData},
    },
};
use std::{borrow::Borrow, ops::Deref};

/// Create a new boolean object - [[Construct]]
pub fn construct_boolean(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.set_kind(ObjectKind::Boolean);

    // Get the argument, if any
    match args.get(0) {
        Some(ref value) => {
            this.set_internal_slot("BooleanData", to_boolean(value));
        }
        None => {
            this.set_internal_slot("BooleanData", to_boolean(&to_value(false)));
        }
    }

    // no need to return `this` as its passed by reference
    Ok(this.clone())
}

/// Return a boolean literal [[Call]]
pub fn call_boolean(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // Get the argument, if any
    match args.get(0) {
        Some(ref value) => Ok(to_boolean(value)),
        None => Ok(to_boolean(&to_value(false))),
    }
}

/// https://tc39.es/ecma262/#sec-boolean.prototype.tostring
pub fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let b = this_boolean_value(this);
    Ok(to_value(b.to_string()))
}

/// https://tc39.es/ecma262/#sec-boolean.prototype.valueof
pub fn value_of(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(this_boolean_value(this))
}

/// Create a new `Boolean` object
pub fn create_constructor(global: &Value) -> Value {
    let mut boolean = Object::default();
    boolean.kind = ObjectKind::Function;
    boolean.set_internal_method("construct", construct_boolean);
    boolean.set_internal_method("call", call_boolean);
    // Create Prototype
    // https://tc39.es/ecma262/#sec-properties-of-the-boolean-prototype-object
    let boolean_prototype = ValueData::new_obj(Some(global));
    boolean_prototype.set_internal_slot("BooleanData", to_boolean(&to_value(false)));
    boolean_prototype.set_field_slice("toString", to_value(to_string as NativeFunctionData));
    boolean_prototype.set_field_slice("valueOf", to_value(value_of as NativeFunctionData));

    let boolean_value = to_value(boolean);
    boolean_prototype.set_field_slice("constructor", to_value(boolean_value.clone()));
    boolean_value.set_field_slice(PROTOTYPE, boolean_prototype);
    boolean_value
}

// === Utility Functions ===
/// [toBoolean](https://tc39.github.io/ecma262/#sec-toboolean)
/// Creates a new boolean value from the input
pub fn to_boolean(value: &Value) -> Value {
    match *value.deref().borrow() {
        ValueData::Object(_) => to_value(true),
        ValueData::String(ref s) if !s.is_empty() => to_value(true),
        ValueData::Number(n) if n != 0.0 && !n.is_nan() => to_value(true),
        ValueData::Integer(n) if n != 0 => to_value(true),
        ValueData::Boolean(v) => to_value(v),
        _ => to_value(false),
    }
}

pub fn this_boolean_value(value: &Value) -> Value {
    match *value.deref().borrow() {
        ValueData::Boolean(v) => to_value(v),
        ValueData::Object(ref v) => (v).deref().borrow().get_internal_slot("BooleanData"),
        _ => to_value(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::Executor;
    use crate::realm::Realm;
    use crate::{forward, forward_val, js::value::same_value};

    #[test]
    fn check_boolean_constructor_is_function() {
        let global = ValueData::new_obj(None);
        let boolean_constructor = create_constructor(&global);
        assert_eq!(boolean_constructor.is_function(), true);
    }

    #[allow(clippy::result_unwrap_used)]
    #[test]
    /// Test the correct type is returned from call and construct
    fn construct_and_call() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const one = new Boolean(1);
        const zero = Boolean(0);
        "#;
        forward(&mut engine, init);
        let one = forward_val(&mut engine, "one").unwrap();
        let zero = forward_val(&mut engine, "zero").unwrap();

        assert_eq!(one.is_object(), true);
        assert_eq!(zero.is_boolean(), true);
    }

    #[test]
    fn constructor_gives_true_instance() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const trueVal = new Boolean(true);
        const trueNum = new Boolean(1);
        const trueString = new Boolean("true");
        const trueBool = new Boolean(trueVal);
        "#;

        forward(&mut engine, init);
        let true_val = forward_val(&mut engine, "trueVal").expect("value expected");
        let true_num = forward_val(&mut engine, "trueNum").expect("value expected");
        let true_string = forward_val(&mut engine, "trueString").expect("value expected");
        let true_bool = forward_val(&mut engine, "trueBool").expect("value expected");

        // Values should all be objects
        assert_eq!(true_val.is_object(), true);
        assert_eq!(true_num.is_object(), true);
        assert_eq!(true_string.is_object(), true);
        assert_eq!(true_bool.is_object(), true);

        // Values should all be truthy
        assert_eq!(true_val.is_true(), true);
        assert_eq!(true_num.is_true(), true);
        assert_eq!(true_string.is_true(), true);
        assert_eq!(true_bool.is_true(), true);
    }

    #[test]
    fn instances_have_correct_proto_set() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const boolInstance = new Boolean(true);
        const boolProto = Boolean.prototype;
        "#;

        forward(&mut engine, init);
        let bool_instance = forward_val(&mut engine, "boolInstance").expect("value expected");
        let bool_prototype = forward_val(&mut engine, "boolProto").expect("value expected");

        assert!(same_value(
            &bool_instance.get_internal_slot("__proto__"),
            &bool_prototype,
            true
        ));
    }
}
