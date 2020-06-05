//! Array declaration execution.

use super::{Executable, Interpreter};
use crate::{
    builtins::{Array, ResultValue},
    syntax::ast::node::{ArrayDecl, Node},
    BoaProfiler,
};

impl Executable for ArrayDecl {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
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
