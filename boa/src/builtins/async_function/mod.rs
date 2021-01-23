use crate::{
    environment::lexical_environment::Environment,
    gc::{empty_trace, Finalize, Trace},
    syntax::ast::node::{FormalParameter, RcStatementList},
};

#[derive(Clone, Finalize, Debug)]
pub struct AsyncFunction {
    body: RcStatementList,
    params: Box<[FormalParameter]>,
    environment: Environment,
}

unsafe impl Trace for AsyncFunction {
    empty_trace!();
}
