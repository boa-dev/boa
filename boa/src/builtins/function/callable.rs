use crate::{Context, Result, Value};

pub trait Callable {
    fn call(this: &Value, args: &[Value], context: &mut Context) -> Result<Value>;
}
