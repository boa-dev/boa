//! Declaration execution.

use super::{Executable, Interpreter};
use crate::{
    builtins::function::FunctionFlags,
    environment::lexical_environment::VariableScope,
    syntax::ast::node::{
        ArrowFunctionDecl, ConstDeclList, FunctionDecl, FunctionExpr, LetDeclList, VarDeclList,
    },
    BoaProfiler, Result, Value,
};

impl Executable for FunctionDecl {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("FunctionDecl", "exec");
        let val = interpreter.create_function(
            self.parameters().to_vec(),
            self.body().to_vec(),
            FunctionFlags::CALLABLE | FunctionFlags::CONSTRUCTABLE,
        );

        // Set the name and assign it in the current environment
        val.set_field("name", self.name());
        interpreter.realm_mut().environment.create_mutable_binding(
            self.name().to_owned(),
            false,
            VariableScope::Function,
        );

        interpreter
            .realm_mut()
            .environment
            .initialize_binding(self.name(), val);

        Ok(Value::undefined())
    }
}

impl Executable for FunctionExpr {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let val = interpreter.create_function(
            self.parameters().to_vec(),
            self.body().to_vec(),
            FunctionFlags::CALLABLE | FunctionFlags::CONSTRUCTABLE,
        );

        if let Some(name) = self.name() {
            val.set_field("name", Value::from(name));
        }

        Ok(val)
    }
}

impl Executable for VarDeclList {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        for var in self.as_ref() {
            let val = match var.init() {
                Some(v) => v.run(interpreter)?,
                None => Value::undefined(),
            };
            let environment = &mut interpreter.realm_mut().environment;

            if environment.has_binding(var.name()) {
                if var.init().is_some() {
                    environment.set_mutable_binding(var.name(), val, true);
                }
            } else {
                environment.create_mutable_binding(
                    var.name().to_owned(),
                    false,
                    VariableScope::Function,
                );
                environment.initialize_binding(var.name(), val);
            }
        }
        Ok(Value::undefined())
    }
}

impl Executable for ConstDeclList {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        for decl in self.as_ref() {
            let val = decl.init().run(interpreter)?;

            interpreter
                .realm_mut()
                .environment
                .create_immutable_binding(decl.name().to_owned(), false, VariableScope::Block);

            interpreter
                .realm_mut()
                .environment
                .initialize_binding(decl.name(), val);
        }
        Ok(Value::undefined())
    }
}

impl Executable for LetDeclList {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        for var in self.as_ref() {
            let val = match var.init() {
                Some(v) => v.run(interpreter)?,
                None => Value::undefined(),
            };
            interpreter.realm_mut().environment.create_mutable_binding(
                var.name().to_owned(),
                false,
                VariableScope::Block,
            );
            interpreter
                .realm_mut()
                .environment
                .initialize_binding(var.name(), val);
        }
        Ok(Value::undefined())
    }
}

impl Executable for ArrowFunctionDecl {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        Ok(interpreter.create_function(
            self.params().to_vec(),
            self.body().to_vec(),
            FunctionFlags::CALLABLE
                | FunctionFlags::CONSTRUCTABLE
                | FunctionFlags::LEXICAL_THIS_MODE,
        ))
    }
}
