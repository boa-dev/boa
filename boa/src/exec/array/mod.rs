//! Array declaration execution.

use super::{Context, Executable};
use crate::{
    builtins::Array,
    syntax::ast::node::{ArrayDecl, Node},
    BoaProfiler, Result, Value,
};

impl Executable for ArrayDecl {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("ArrayDecl", "exec");
        let array = Array::new_array(interpreter)?;
        let mut elements = Vec::new();
        for elem in self.as_ref() {
            if let Node::Spread(ref x) = elem {
                let val = x.run(interpreter)?;
                let mut vals = interpreter.extract_array_properties(&val).unwrap();
                elements.append(&mut vals);
                continue; // Don't push array after spread
            }
            elements.push(elem.run(interpreter)?);
        }
        Array::add_to_array_object(&array, &elements)?;

        Ok(array)
    }
}
