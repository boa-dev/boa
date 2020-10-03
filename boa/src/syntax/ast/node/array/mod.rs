//! Array declaration node.

use super::{join_nodes, Node};
use crate::{
    builtins::{iterable, Array},
    exec::Executable,
    gc::{Finalize, Trace},
    BoaProfiler, Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An array is an ordered collection of data (either primitive or object depending upon the
/// language).
///
/// Arrays are used to store multiple values in a single variable.
/// This is compared to a variable that can store only one value.
///
/// Each item in an array has a number attached to it, called a numeric index, that allows you
/// to access it. In JavaScript, arrays start at index zero and can be manipulated with various
/// methods.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ArrayLiteral
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ArrayDecl {
    #[cfg_attr(feature = "serde", serde(flatten))]
    arr: Box<[Node]>,
}

impl Executable for ArrayDecl {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("ArrayDecl", "exec");
        let array = Array::new_array(interpreter)?;
        let mut elements = Vec::new();
        for elem in self.as_ref() {
            if let Node::Spread(ref x) = elem {
                let val = x.run(interpreter)?;
                let iterator_record = iterable::get_iterator(interpreter, val)?;
                // TODO after proper internal Array representation as per https://github.com/boa-dev/boa/pull/811#discussion_r502460858
                // next_index variable should be utilized here as per https://tc39.es/ecma262/#sec-runtime-semantics-arrayaccumulation
                // let mut next_index = 0;
                loop {
                    let next = iterator_record.next(interpreter)?;
                    if next.is_done() {
                        break;
                    }
                    let next_value = next.value();
                    //next_index += 1;
                    elements.push(next_value.clone());
                }
            } else {
                elements.push(elem.run(interpreter)?);
            }
        }

        Array::add_to_array_object(&array, &elements)?;
        Ok(array)
    }
}

impl AsRef<[Node]> for ArrayDecl {
    fn as_ref(&self) -> &[Node] {
        &self.arr
    }
}

impl<T> From<T> for ArrayDecl
where
    T: Into<Box<[Node]>>,
{
    fn from(decl: T) -> Self {
        Self { arr: decl.into() }
    }
}

impl fmt::Display for ArrayDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        join_nodes(f, &self.arr)?;
        f.write_str("]")
    }
}

impl From<ArrayDecl> for Node {
    fn from(arr: ArrayDecl) -> Self {
        Self::ArrayDecl(arr)
    }
}
