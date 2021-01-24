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

impl AsyncFunction {
    pub fn new<B, P, E>(body: B, params: P, environment: E) -> Self
    where
        B: Into<RcStatementList>,
        P: Into<Box<[FormalParameter]>>,
        E: Into<Environment>,
    {
        Self {
            body: body.into(),
            params: params.into(),
            environment: environment.into(),
        }
    }
}

unsafe impl Trace for AsyncFunction {
    empty_trace!();
}
