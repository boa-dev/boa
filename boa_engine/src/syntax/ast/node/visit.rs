use crate::syntax::ast::node::declaration::{
    BindingPatternTypeArray, BindingPatternTypeObject, DeclarationPatternArray,
    DeclarationPatternObject,
};
use crate::syntax::ast::node::iteration::IterableLoopInitializer;
use crate::syntax::ast::node::object::{MethodDefinition, PropertyDefinition, PropertyName};
use crate::syntax::ast::node::operator::assign::AssignTarget;
use crate::syntax::ast::node::template::TemplateElement;
use crate::syntax::ast::node::{
    ArrayDecl, ArrowFunctionDecl, Assign, AsyncFunctionDecl, AsyncGeneratorDecl,
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
    fn visit_node(&mut self, _n: &'ast mut Node) {
        todo!()
    }

    fn visit_array_decl(&mut self, _n: &'ast mut ArrayDecl) {
        todo!()
    }

    fn visit_arrow_function_decl(&mut self, _n: &'ast mut ArrowFunctionDecl) {
        todo!()
    }

    fn visit_assign(&mut self, _n: &'ast mut Assign) {
        todo!()
    }

    fn visit_async_function_decl(&mut self, _n: &'ast mut AsyncFunctionDecl) {
        todo!()
    }

    fn visit_async_generator_expr(&mut self, _n: &'ast mut AsyncGeneratorExpr) {
        todo!()
    }

    fn visit_async_generator_decl(&mut self, _n: &'ast mut AsyncGeneratorDecl) {
        todo!()
    }

    fn visit_await_expr(&mut self, _n: &'ast mut AwaitExpr) {
        todo!()
    }

    fn visit_bin_op(&mut self, _n: &'ast mut BinOp) {
        todo!()
    }

    fn visit_block(&mut self, _n: &'ast mut Block) {
        todo!()
    }

    fn visit_break(&mut self, _n: &'ast mut Break) {
        todo!()
    }

    fn visit_call(&mut self, _n: &'ast mut Call) {
        todo!()
    }

    fn visit_conditional_op(&mut self, _n: &'ast mut ConditionalOp) {
        todo!()
    }

    fn visit_const(&mut self, _n: &'ast mut Const) {
        todo!()
    }

    fn visit_const_decl_list(&mut self, _n: &'ast mut DeclarationList) {
        todo!()
    }

    fn visit_continue(&mut self, _n: &'ast mut Continue) {
        todo!()
    }

    fn visit_do_while_loop(&mut self, _n: &'ast mut DoWhileLoop) {
        todo!()
    }

    fn visit_function_decl(&mut self, _n: &'ast mut FunctionDecl) {
        todo!()
    }

    fn visit_function_expr(&mut self, _n: &'ast mut FunctionExpr) {
        todo!()
    }

    fn visit_get_const_field(&mut self, _n: &'ast mut GetConstField) {
        todo!()
    }

    fn visit_get_field(&mut self, _n: &'ast mut GetField) {
        todo!()
    }

    fn visit_for_loop(&mut self, _n: &'ast mut ForLoop) {
        todo!()
    }

    fn visit_for_in_loop(&mut self, _n: &'ast mut ForInLoop) {
        todo!()
    }

    fn visit_for_of_loop(&mut self, _n: &'ast mut ForOfLoop) {
        todo!()
    }

    fn visit_if(&mut self, _n: &'ast mut If) {
        todo!()
    }

    fn visit_let_decl_list(&mut self, _n: &'ast mut DeclarationList) {
        todo!()
    }

    fn visit_identifier(&mut self, _n: &'ast mut Identifier) {
        todo!()
    }

    fn visit_new(&mut self, _n: &'ast mut New) {
        todo!()
    }

    fn visit_object(&mut self, _n: &'ast mut Object) {
        todo!()
    }

    fn visit_return(&mut self, _n: &'ast mut Return) {
        todo!()
    }

    fn visit_switch(&mut self, _n: &'ast mut Switch) {
        todo!()
    }

    fn visit_spread(&mut self, _n: &'ast mut Spread) {
        todo!()
    }

    fn visit_tagged_template(&mut self, _n: &'ast mut TaggedTemplate) {
        todo!()
    }

    fn visit_template_lit(&mut self, _n: &'ast mut TemplateLit) {
        todo!()
    }

    fn visit_throw(&mut self, _n: &'ast mut Throw) {
        todo!()
    }

    fn visit_try(&mut self, _n: &'ast mut Try) {
        todo!()
    }

    fn visit_unary_op(&mut self, _n: &'ast mut UnaryOp) {
        todo!()
    }

    fn visit_var_decl_list(&mut self, _n: &'ast mut DeclarationList) {
        todo!()
    }

    fn visit_while_loop(&mut self, _n: &'ast mut WhileLoop) {
        todo!()
    }

    fn visit_yield(&mut self, _n: &'ast mut Yield) {
        todo!()
    }

    fn visit_generator_decl(&mut self, _n: &'ast mut GeneratorDecl) {
        todo!()
    }

    fn visit_generator_expr(&mut self, _n: &'ast mut GeneratorExpr) {
        todo!()
    }

    fn visit_sym(&mut self, _n: &'ast mut Sym) {
        todo!()
    }

    fn visit_formal_parameter_list(&mut self, _n: &'ast mut FormalParameterList) {
        todo!()
    }

    fn visit_statement_list(&mut self, _n: &'ast mut StatementList) {
        todo!()
    }

    fn visit_assign_target(&mut self, _n: &'ast mut AssignTarget) {
        todo!()
    }

    fn visit_raw_binop(&mut self, _n: &'ast mut op::BinOp) {
        todo!()
    }

    fn visit_declaration(&mut self, _n: &'ast mut Declaration) {
        todo!()
    }

    fn visit_iterable_loop_initializer(&mut self, _n: &'ast mut IterableLoopInitializer) {
        todo!()
    }

    fn visit_property_definition(&mut self, _n: &'ast mut PropertyDefinition) {
        todo!()
    }

    fn visit_case(&mut self, _n: &'ast mut Case) {
        todo!()
    }

    fn visit_template_element(&mut self, _n: &'ast mut TemplateElement) {
        todo!()
    }

    fn visit_catch(&mut self, _n: &'ast mut Catch) {
        todo!()
    }

    fn visit_finally(&mut self, _n: &'ast mut Finally) {
        todo!()
    }

    fn visit_raw_unary_op(&mut self, _n: &'ast mut op::UnaryOp) {
        todo!()
    }

    fn visit_formal_parameter(&mut self, _n: &'ast mut FormalParameter) {
        todo!()
    }

    fn visit_formal_parameter_list_flags(&mut self, _n: &'ast mut FormalParameterListFlags) {
        todo!()
    }

    fn visit_declaration_pattern(&mut self, _n: &'ast mut DeclarationPattern) {
        todo!()
    }

    fn visit_raw_num_op(&mut self, _n: &'ast mut op::NumOp) {
        todo!()
    }

    fn visit_raw_bit_op(&mut self, _n: &'ast mut op::BitOp) {
        todo!()
    }

    fn visit_raw_comp_op(&mut self, _n: &'ast mut op::CompOp) {
        todo!()
    }

    fn visit_raw_log_op(&mut self, _n: &'ast mut op::LogOp) {
        todo!()
    }

    fn visit_raw_assign_op(&mut self, _n: &'ast mut op::AssignOp) {
        todo!()
    }

    fn visit_property_name(&mut self, _n: &'ast mut PropertyName) {
        todo!()
    }

    fn visit_method_definition(&mut self, _n: &'ast mut MethodDefinition) {
        todo!()
    }

    fn visit_declaration_pattern_object(&mut self, _n: &'ast mut DeclarationPatternObject) {
        todo!()
    }

    fn visit_declaration_pattern_array(&mut self, _n: &'ast mut DeclarationPatternArray) {
        todo!()
    }

    fn visit_binding_pattern_type_object(&mut self, _n: &'ast mut BindingPatternTypeObject) {
        todo!()
    }

    fn visit_binding_pattern_type_array(&mut self, _n: &'ast mut BindingPatternTypeArray) {
        todo!()
    }
}
