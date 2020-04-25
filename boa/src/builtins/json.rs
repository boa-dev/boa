use crate::builtins::function::NativeFunctionData;
//! The JSON Object
//!
//! <https://tc39.es/ecma262/#sec-json-object>

use crate::builtins::value::{to_value, ResultValue, Value, ValueData};
use crate::exec::Interpreter;
use serde_json::{self, Value as JSONValue};

/// Parse a JSON string into a Javascript object
/// <https://tc39.es/ecma262/#sec-json.parse>
pub fn parse(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    match serde_json::from_str::<JSONValue>(
        &args
            .get(0)
            .expect("cannot get argument for JSON.parse")
            .clone()
            .to_string(),
    ) {
        Ok(json) => Ok(to_value(json)),
        Err(err) => Err(to_value(err.to_string())),
    }
}
/// Process a Javascript object into a JSON string
pub fn stringify(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("cannot get argument for JSON.stringify");
    let json = obj.to_json().to_string();
    Ok(to_value(json))
}

/// Create a new `JSON` object
pub fn create_constructor(global: &Value) -> Value {
    let json = ValueData::new_obj(Some(global));

    make_builtin_fn!(parse, named "parse", with length 2, of json);
    make_builtin_fn!(stringify, named "stringify", with length 3, of json);

    to_value(json)
}

#[cfg(test)]
mod tests {
    use crate::{
    	exec::Executor,
    	forward,
    	realm::Realm,
    };

    #[test]
    fn json_sanity() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        assert_eq!(
            forward(&mut engine, r#"JSON.parse('{"aaa":"bbb"}').aaa == 'bbb'"#),
            "true"
        );
        assert_eq!(
            forward(
                &mut engine,
                r#"JSON.stringify({aaa: 'bbb'}) == '{"aaa":"bbb"}'"#
            ),
            "true"
        );
    }
}
