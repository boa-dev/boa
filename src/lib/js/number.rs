use crate::{
    exec::Interpreter,
    js::value::{ResultValue, Value},
};

/// Create a new number [[Construct]]
pub fn make_number(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number-constructor-number-value
pub fn call_number(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.toexponential
pub fn to_expotential(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.tofixed
pub fn to_fixed(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.tolocalestring
pub fn to_locale_string(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.toprecision
pub fn to_precision(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.tostring
pub fn to_string(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.valueof
pub fn value_of(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// Create a new `Number` object
pub fn create_constructor(global: &Value) -> Value {
    unimplemented!()
}

/// Iniitalize the `Number` object on the global object
pub fn init(global: &Value) {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn make_number() {
        unimplemented!()
    }

    #[test]
    pub fn call_number() {
        unimplemented!()
    }

    #[test]
    pub fn to_expotential() {
        unimplemented!()
    }

    #[test]
    pub fn to_fixed() {
        unimplemented!()
    }

    #[test]
    pub fn to_locale_string() {
        unimplemented!()
    }

    #[test]
    pub fn to_precision() {
        unimplemented!()
    }

    #[test]
    pub fn to_string() {
        unimplemented!()
    }

    #[test]
    pub fn value_of() {
        unimplemented!()
    }
}
