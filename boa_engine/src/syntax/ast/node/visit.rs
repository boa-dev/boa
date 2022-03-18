use crate::syntax::ast::node::declaration::{
    BindingPatternTypeArray, BindingPatternTypeObject, DeclarationPatternArray,
    DeclarationPatternObject,
};
use crate::syntax::ast::node::iteration::IterableLoopInitializer;
use crate::syntax::ast::node::object::{MethodDefinition, PropertyDefinition, PropertyName};
use crate::syntax::ast::node::operator::assign::AssignTarget;
use crate::syntax::ast::node::template::TemplateElement;
use crate::syntax::ast::node::{
    ArrayDecl, ArrowFunctionDecl, Assign, AsyncFunctionDecl, AsyncFunctionExpr, AsyncGeneratorDecl,
    AsyncGeneratorExpr, AwaitExpr, BinOp, Block, Break, Call, Case, Catch, ConditionalOp, Continue,
    Declaration, DeclarationList, DeclarationPattern, DoWhileLoop, Finally, ForInLoop, ForLoop,
    ForOfLoop, FormalParameter, FormalParameterList, FormalParameterListFlags, FunctionDecl,
    FunctionExpr, GeneratorDecl, GeneratorExpr, GetConstField, GetField, Identifier, If, New,
    Object, Return, Spread, StatementList, Switch, TaggedTemplate, TemplateLit, Throw, Try,
    UnaryOp, WhileLoop, Yield,
};
use crate::syntax::ast::{op, Const, Node};
use boa_interner::Sym;

pub trait Visitor<'ast> {
    fn visit_node(&mut self, n: &'ast Node) {
        match n {
            Node::ArrayDecl(n) => self.visit_array_decl(n),
            Node::ArrowFunctionDecl(n) => self.visit_arrow_function_decl(n),
            Node::Assign(n) => self.visit_assign(n),
            Node::AsyncFunctionDecl(n) => self.visit_async_function_decl(n),
            Node::AsyncFunctionExpr(n) => self.visit_async_function_expr(n),
            Node::AsyncGeneratorExpr(n) => self.visit_async_generator_expr(n),
            Node::AsyncGeneratorDecl(n) => self.visit_async_generator_decl(n),
            Node::AwaitExpr(n) => self.visit_await_expr(n),
            Node::BinOp(n) => self.visit_bin_op(n),
            Node::Block(n) => self.visit_block(n),
            Node::Break(n) => self.visit_break(n),
            Node::Call(n) => self.visit_call(n),
            Node::ConditionalOp(n) => self.visit_conditional_op(n),
            Node::Const(n) => self.visit_const(n),
            Node::ConstDeclList(n) => self.visit_declaration_list(n),
            Node::Continue(n) => self.visit_continue(n),
            Node::DoWhileLoop(n) => self.visit_do_while_loop(n),
            Node::FunctionDecl(n) => self.visit_function_decl(n),
            Node::FunctionExpr(n) => self.visit_function_expr(n),
            Node::GetConstField(n) => self.visit_get_const_field(n),
            Node::GetField(n) => self.visit_get_field(n),
            Node::ForLoop(n) => self.visit_for_loop(n),
            Node::ForInLoop(n) => self.visit_for_in_loop(n),
            Node::ForOfLoop(n) => self.visit_for_of_loop(n),
            Node::If(n) => self.visit_if(n),
            Node::LetDeclList(n) => self.visit_declaration_list(n),
            Node::Identifier(n) => self.visit_identifier(n),
            Node::New(n) => self.visit_new(n),
            Node::Object(n) => self.visit_object(n),
            Node::Return(n) => self.visit_return(n),
            Node::Switch(n) => self.visit_switch(n),
            Node::Spread(n) => self.visit_spread(n),
            Node::TaggedTemplate(n) => self.visit_tagged_template(n),
            Node::TemplateLit(n) => self.visit_template_lit(n),
            Node::Throw(n) => self.visit_throw(n),
            Node::Try(n) => self.visit_try(n),
            Node::UnaryOp(n) => self.visit_unary_op(n),
            Node::VarDeclList(n) => self.visit_declaration_list(n),
            Node::WhileLoop(n) => self.visit_while_loop(n),
            Node::Yield(n) => self.visit_yield(n),
            Node::GeneratorDecl(n) => self.visit_generator_decl(n),
            Node::GeneratorExpr(n) => self.visit_generator_expr(n),
            Node::Empty | Node::This => { /* do nothing */ }
        }
    }

    fn visit_array_decl(&mut self, n: &'ast ArrayDecl) {
        for inner in n.arr.iter() {
            self.visit_node(inner);
        }
    }

    fn visit_arrow_function_decl(&mut self, n: &'ast ArrowFunctionDecl) {
        if let Some(name) = &n.name {
            self.visit_sym(name);
        }
        self.visit_formal_parameter_list(&n.params);
        self.visit_statement_list(&n.body);
    }

    fn visit_assign(&mut self, n: &'ast Assign) {
        self.visit_assign_target(&n.lhs);
        self.visit_node(&n.rhs);
    }

    fn visit_async_function_expr(&mut self, n: &'ast AsyncFunctionExpr) {
        if let Some(name) = &n.name {
            self.visit_sym(name);
        }
        self.visit_formal_parameter_list(&n.parameters);
        self.visit_statement_list(&n.body);
    }

    fn visit_async_function_decl(&mut self, n: &'ast AsyncFunctionDecl) {
        self.visit_sym(&n.name);
        self.visit_formal_parameter_list(&n.parameters);
        self.visit_statement_list(&n.body);
    }

    fn visit_async_generator_expr(&mut self, n: &'ast AsyncGeneratorExpr) {
        if let Some(name) = &n.name {
            self.visit_sym(name);
        }
        self.visit_formal_parameter_list(&n.parameters);
        self.visit_statement_list(&n.body);
    }

    fn visit_async_generator_decl(&mut self, n: &'ast AsyncGeneratorDecl) {
        self.visit_sym(&n.name);
        self.visit_formal_parameter_list(&n.parameters);
        self.visit_statement_list(&n.body);
    }

    fn visit_await_expr(&mut self, n: &'ast AwaitExpr) {
        self.visit_node(&n.expr);
    }

    fn visit_bin_op(&mut self, n: &'ast BinOp) {
        self.visit_raw_binop(&n.op);
        self.visit_node(&n.lhs);
        self.visit_node(&n.rhs);
    }

    fn visit_block(&mut self, n: &'ast Block) {
        self.visit_statement_list(&n.statements);
    }

    fn visit_break(&mut self, n: &'ast Break) {
        if let Some(name) = &n.label {
            self.visit_sym(name);
        }
    }

    fn visit_call(&mut self, n: &'ast Call) {
        self.visit_node(&n.expr);
        for inner in n.args.iter() {
            self.visit_node(inner);
        }
    }

    fn visit_conditional_op(&mut self, n: &'ast ConditionalOp) {
        self.visit_node(&n.condition);
        self.visit_node(&n.if_true);
        self.visit_node(&n.if_false);
    }

    fn visit_const(&mut self, n: &'ast Const) {
        if let Const::String(s) = n {
            self.visit_sym(s);
        }
    }

    fn visit_continue(&mut self, n: &'ast Continue) {
        if let Some(s) = &n.label {
            self.visit_sym(s);
        }
    }

    fn visit_do_while_loop(&mut self, n: &'ast DoWhileLoop) {
        self.visit_node(&n.body);
        self.visit_node(&n.cond);
        if let Some(name) = &n.label {
            self.visit_sym(name);
        }
    }

    fn visit_function_decl(&mut self, _n: &'ast FunctionDecl) {
        todo!()
    }

    fn visit_function_expr(&mut self, _n: &'ast FunctionExpr) {
        todo!()
    }

    fn visit_get_const_field(&mut self, _n: &'ast GetConstField) {
        todo!()
    }

    fn visit_get_field(&mut self, _n: &'ast GetField) {
        todo!()
    }

    fn visit_for_loop(&mut self, _n: &'ast ForLoop) {
        todo!()
    }

    fn visit_for_in_loop(&mut self, _n: &'ast ForInLoop) {
        todo!()
    }

    fn visit_for_of_loop(&mut self, _n: &'ast ForOfLoop) {
        todo!()
    }

    fn visit_if(&mut self, _n: &'ast If) {
        todo!()
    }

    fn visit_identifier(&mut self, _n: &'ast Identifier) {
        todo!()
    }

    fn visit_new(&mut self, _n: &'ast New) {
        todo!()
    }

    fn visit_object(&mut self, _n: &'ast Object) {
        todo!()
    }

    fn visit_return(&mut self, _n: &'ast Return) {
        todo!()
    }

    fn visit_switch(&mut self, _n: &'ast Switch) {
        todo!()
    }

    fn visit_spread(&mut self, _n: &'ast Spread) {
        todo!()
    }

    fn visit_tagged_template(&mut self, _n: &'ast TaggedTemplate) {
        todo!()
    }

    fn visit_template_lit(&mut self, _n: &'ast TemplateLit) {
        todo!()
    }

    fn visit_throw(&mut self, _n: &'ast Throw) {
        todo!()
    }

    fn visit_try(&mut self, _n: &'ast Try) {
        todo!()
    }

    fn visit_unary_op(&mut self, _n: &'ast UnaryOp) {
        todo!()
    }

    fn visit_declaration_list(&mut self, _n: &'ast DeclarationList) {
        todo!()
    }

    fn visit_while_loop(&mut self, _n: &'ast WhileLoop) {
        todo!()
    }

    fn visit_yield(&mut self, _n: &'ast Yield) {
        todo!()
    }

    fn visit_generator_decl(&mut self, _n: &'ast GeneratorDecl) {
        todo!()
    }

    fn visit_generator_expr(&mut self, _n: &'ast GeneratorExpr) {
        todo!()
    }

    fn visit_sym(&mut self, _n: &'ast Sym) {
        todo!()
    }

    fn visit_formal_parameter_list(&mut self, _n: &'ast FormalParameterList) {
        todo!()
    }

    fn visit_statement_list(&mut self, _n: &'ast StatementList) {
        todo!()
    }

    fn visit_assign_target(&mut self, _n: &'ast AssignTarget) {
        todo!()
    }

    fn visit_raw_binop(&mut self, _n: &'ast op::BinOp) {
        todo!()
    }

    fn visit_declaration(&mut self, _n: &'ast Declaration) {
        todo!()
    }

    fn visit_iterable_loop_initializer(&mut self, _n: &'ast IterableLoopInitializer) {
        todo!()
    }

    fn visit_property_definition(&mut self, _n: &'ast PropertyDefinition) {
        todo!()
    }

    fn visit_case(&mut self, _n: &'ast Case) {
        todo!()
    }

    fn visit_template_element(&mut self, _n: &'ast TemplateElement) {
        todo!()
    }

    fn visit_catch(&mut self, _n: &'ast Catch) {
        todo!()
    }

    fn visit_finally(&mut self, _n: &'ast Finally) {
        todo!()
    }

    fn visit_raw_unary_op(&mut self, _n: &'ast op::UnaryOp) {
        todo!()
    }

    fn visit_formal_parameter(&mut self, _n: &'ast FormalParameter) {
        todo!()
    }

    fn visit_formal_parameter_list_flags(&mut self, _n: &'ast FormalParameterListFlags) {
        todo!()
    }

    fn visit_declaration_pattern(&mut self, _n: &'ast DeclarationPattern) {
        todo!()
    }

    fn visit_raw_num_op(&mut self, _n: &'ast op::NumOp) {
        todo!()
    }

    fn visit_raw_bit_op(&mut self, _n: &'ast op::BitOp) {
        todo!()
    }

    fn visit_raw_comp_op(&mut self, _n: &'ast op::CompOp) {
        todo!()
    }

    fn visit_raw_log_op(&mut self, _n: &'ast op::LogOp) {
        todo!()
    }

    fn visit_raw_assign_op(&mut self, _n: &'ast op::AssignOp) {
        todo!()
    }

    fn visit_property_name(&mut self, _n: &'ast PropertyName) {
        todo!()
    }

    fn visit_method_definition(&mut self, _n: &'ast MethodDefinition) {
        todo!()
    }

    fn visit_declaration_pattern_object(&mut self, _n: &'ast DeclarationPatternObject) {
        todo!()
    }

    fn visit_declaration_pattern_array(&mut self, _n: &'ast DeclarationPatternArray) {
        todo!()
    }

    fn visit_binding_pattern_type_object(&mut self, _n: &'ast BindingPatternTypeObject) {
        todo!()
    }

    fn visit_binding_pattern_type_array(&mut self, _n: &'ast BindingPatternTypeArray) {
        todo!()
    }
}
