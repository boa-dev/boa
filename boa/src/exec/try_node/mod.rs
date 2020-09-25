//! Try..catch node execution.

use super::{Context, Executable};
use crate::{
    environment::lexical_environment::{new_declarative_environment, VariableScope},
    syntax::ast::node::Try,
    BoaProfiler, Result, Value,
};

#[cfg(test)]
mod tests;

impl Executable for Try {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("Try", "exec");
        let res = self.block().run(interpreter).map_or_else(
            |err| {
                if let Some(catch) = self.catch() {
                    {
                        let env = &mut interpreter.realm_mut().environment;
                        env.push(new_declarative_environment(Some(
                            env.get_current_environment_ref().clone(),
                        )));

                        if let Some(param) = catch.parameter() {
                            env.create_mutable_binding(
                                param.to_owned(),
                                false,
                                VariableScope::Block,
                            );

                            env.initialize_binding(param, err);
                        }
                    }

                    let res = catch.block().run(interpreter);

                    // pop the block env
                    let _ = interpreter.realm_mut().environment.pop();

                    res
                } else {
                    Err(err)
                }
            },
            Ok,
        );

        if let Some(finally) = self.finally() {
            finally.run(interpreter)?;
        }

        res
    }
}
