use crate::syntax::ast::node::declaration::class_decl::ClassElement;
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
    AsyncGeneratorExpr, AwaitExpr, BinOp, Block, Break, Call, Case, Catch, Class, ConditionalOp,
    Continue, Declaration, DeclarationList, DeclarationPattern, DoWhileLoop, Finally, ForInLoop,
    ForLoop, ForOfLoop, FormalParameter, FormalParameterList, FormalParameterListFlags,
    FunctionDecl, FunctionExpr, GeneratorDecl, GeneratorExpr, GetConstField, GetField,
    GetPrivateField, GetSuperField, Identifier, If, New, Object, Return, Spread, StatementList,
    SuperCall, Switch, TaggedTemplate, TemplateLit, Throw, Try, UnaryOp, WhileLoop, Yield,
};
use crate::syntax::ast::{op, Const, Node};
use boa_interner::Sym;

/// `Visitor`s "walk" the AST at certain nodes. Useful for when you need to modify source code, but at
/// the API level.
///
/// `Visitor`s have default implementations which are "reasonable", which is to say that users merely
/// need to implement the visitor for the specific type they wish to visit. For example, let's say
/// we want to replace all variables named "y" with a variable named "z".
///
/// To do so, we implement `Visitor` over a `NameReplacer` struct, then use `NameReplacer` to, well,
/// replace the names in a parsed source.
///
/// ```
/// use boa_engine::syntax::ast::node::visit::Visitor;
/// use boa_engine::Context;
/// use boa_interner::Sym;
///
/// struct NameReplacer {
///     find: Sym,
///     replace: Sym,
/// }
/// impl<'ast> Visitor<'ast> for NameReplacer {
///     type Output = ();
///     type Error = ();
///
///     fn visit_sym_mut(&mut self, n: &'ast mut Sym) -> Result<Self::Output, Self::Error> {
///         if *n == self.find {
///             *n = self.replace;
///         }
///         Ok(())
///     }
///
///     fn get_default_ok() -> Result<Self::Output, Self::Error> {
///         Ok(())
///     }
/// }
///
/// // -- snip -- //
/// # fn main() -> Result<(), ()>{
/// let source = r#"
/// let x = 5;
/// let y = 6;
/// x + y;
/// "#;
/// let mut ctx = Context::default();
/// let mut parsed = ctx.parse(source).unwrap();
/// let find = ctx.interner_mut().get_or_intern_static("y");
/// let replace = ctx.interner_mut().get_or_intern_static("z");
/// let mut replacer = NameReplacer { find, replace };
/// replacer.visit_statement_list_mut(&mut parsed)?;
/// let replaced = parsed.to_indented_string(ctx.interner(), 0);
/// assert_eq!(
///     replaced,
///     r#"
/// let x = 5;
/// let z = 6;
/// x + z;
/// "#
///     .trim_start()
/// );
/// # Ok(())
/// # }
/// ```
pub trait Visitor<'ast> {
    type Output;
    type Error;

    fn visit_node(&mut self, n: &'ast Node) -> Result<Self::Output, Self::Error> {
        self.walk_node(n)
    }
    fn walk_node(&mut self, n: &'ast Node) -> Result<Self::Output, Self::Error> {
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
            Node::ConstDeclList(n) | Node::LetDeclList(n) | Node::VarDeclList(n) => {
                self.visit_declaration_list(n)
            }
            Node::Continue(n) => self.visit_continue(n),
            Node::DoWhileLoop(n) => self.visit_do_while_loop(n),
            Node::FunctionDecl(n) => self.visit_function_decl(n),
            Node::FunctionExpr(n) => self.visit_function_expr(n),
            Node::GetConstField(n) => self.visit_get_const_field(n),
            Node::GetPrivateField(n) => self.visit_get_private_field(n),
            Node::GetField(n) => self.visit_get_field(n),
            Node::GetSuperField(n) => self.visit_get_super_field(n),
            Node::ForLoop(n) => self.visit_for_loop(n),
            Node::ForInLoop(n) => self.visit_for_in_loop(n),
            Node::ForOfLoop(n) => self.visit_for_of_loop(n),
            Node::If(n) => self.visit_if(n),
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
            Node::WhileLoop(n) => self.visit_while_loop(n),
            Node::Yield(n) => self.visit_yield(n),
            Node::GeneratorDecl(n) => self.visit_generator_decl(n),
            Node::GeneratorExpr(n) => self.visit_generator_expr(n),
            Node::ClassDecl(n) => self.visit_class_decl(n),
            Node::ClassExpr(n) => self.visit_class_expr(n),
            Node::SuperCall(n) => self.visit_super_call(n),
            Node::Empty | Node::This => Self::get_default_ok(),
        }
    }

    fn visit_array_decl(&mut self, n: &'ast ArrayDecl) -> Result<Self::Output, Self::Error> {
        self.walk_array_decl(n)
    }
    fn walk_array_decl(&mut self, n: &'ast ArrayDecl) -> Result<Self::Output, Self::Error> {
        for inner in n.arr.iter() {
            self.visit_node(inner)?;
        }
        Self::get_default_ok()
    }

    fn visit_arrow_function_decl(
        &mut self,
        n: &'ast ArrowFunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_arrow_function_decl(n)
    }
    fn walk_arrow_function_decl(
        &mut self,
        n: &'ast ArrowFunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &n.name {
            self.visit_sym(name)?;
        }
        self.visit_formal_parameter_list(&n.params)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_assign(&mut self, n: &'ast Assign) -> Result<Self::Output, Self::Error> {
        self.walk_assign(n)
    }
    fn walk_assign(&mut self, n: &'ast Assign) -> Result<Self::Output, Self::Error> {
        self.visit_assign_target(&n.lhs)?;
        self.visit_node(&n.rhs)?;
        Self::get_default_ok()
    }

    fn visit_async_function_expr(
        &mut self,
        n: &'ast AsyncFunctionExpr,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_async_function_expr(n)
    }
    fn walk_async_function_expr(
        &mut self,
        n: &'ast AsyncFunctionExpr,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &n.name {
            self.visit_sym(name)?;
        }
        self.visit_formal_parameter_list(&n.parameters)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_async_function_decl(
        &mut self,
        n: &'ast AsyncFunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_async_function_decl(n)
    }
    fn walk_async_function_decl(
        &mut self,
        n: &'ast AsyncFunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_sym(&n.name)?;
        self.visit_formal_parameter_list(&n.parameters)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_async_generator_expr(
        &mut self,
        n: &'ast AsyncGeneratorExpr,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_async_generator_expr(n)
    }
    fn walk_async_generator_expr(
        &mut self,
        n: &'ast AsyncGeneratorExpr,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &n.name {
            self.visit_sym(name)?;
        }
        self.visit_formal_parameter_list(&n.parameters)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_async_generator_decl(
        &mut self,
        n: &'ast AsyncGeneratorDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_async_generator_decl(n)
    }
    fn walk_async_generator_decl(
        &mut self,
        n: &'ast AsyncGeneratorDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_sym(&n.name)?;
        self.visit_formal_parameter_list(&n.parameters)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_await_expr(&mut self, n: &'ast AwaitExpr) -> Result<Self::Output, Self::Error> {
        self.walk_await_expr(n)
    }
    fn walk_await_expr(&mut self, n: &'ast AwaitExpr) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.expr)?;
        Self::get_default_ok()
    }

    fn visit_bin_op(&mut self, n: &'ast BinOp) -> Result<Self::Output, Self::Error> {
        self.walk_bin_op(n)
    }
    fn walk_bin_op(&mut self, n: &'ast BinOp) -> Result<Self::Output, Self::Error> {
        self.visit_raw_binop(&n.op)?;
        self.visit_node(&n.lhs)?;
        self.visit_node(&n.rhs)?;
        Self::get_default_ok()
    }

    fn visit_block(&mut self, n: &'ast Block) -> Result<Self::Output, Self::Error> {
        self.walk_block(n)
    }
    fn walk_block(&mut self, n: &'ast Block) -> Result<Self::Output, Self::Error> {
        self.visit_statement_list(&n.statements)?;
        Self::get_default_ok()
    }

    fn visit_break(&mut self, n: &'ast Break) -> Result<Self::Output, Self::Error> {
        self.walk_break(n)
    }
    fn walk_break(&mut self, n: &'ast Break) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &n.label {
            self.visit_sym(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_call(&mut self, n: &'ast Call) -> Result<Self::Output, Self::Error> {
        self.walk_call(n)
    }
    fn walk_call(&mut self, n: &'ast Call) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.expr)?;
        for inner in n.args.iter() {
            self.visit_node(inner)?;
        }
        Self::get_default_ok()
    }

    fn visit_conditional_op(
        &mut self,
        n: &'ast ConditionalOp,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_conditional_op(n)
    }
    fn walk_conditional_op(&mut self, n: &'ast ConditionalOp) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.condition)?;
        self.visit_node(&n.if_true)?;
        self.visit_node(&n.if_false)?;
        Self::get_default_ok()
    }

    fn visit_const(&mut self, n: &'ast Const) -> Result<Self::Output, Self::Error> {
        self.walk_const(n)
    }
    fn walk_const(&mut self, n: &'ast Const) -> Result<Self::Output, Self::Error> {
        if let Const::String(s) = n {
            self.visit_sym(s)?;
        }
        Self::get_default_ok()
    }

    fn visit_continue(&mut self, n: &'ast Continue) -> Result<Self::Output, Self::Error> {
        self.walk_continue(n)
    }
    fn walk_continue(&mut self, n: &'ast Continue) -> Result<Self::Output, Self::Error> {
        if let Some(s) = &n.label {
            self.visit_sym(s)?;
        }
        Self::get_default_ok()
    }

    fn visit_do_while_loop(&mut self, n: &'ast DoWhileLoop) -> Result<Self::Output, Self::Error> {
        self.walk_do_while_loop(n)
    }
    fn walk_do_while_loop(&mut self, n: &'ast DoWhileLoop) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.body)?;
        self.visit_node(&n.cond)?;
        if let Some(name) = &n.label {
            self.visit_sym(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_function_decl(&mut self, n: &'ast FunctionDecl) -> Result<Self::Output, Self::Error> {
        self.walk_function_decl(n)
    }
    fn walk_function_decl(&mut self, n: &'ast FunctionDecl) -> Result<Self::Output, Self::Error> {
        self.visit_sym(&n.name)?;
        self.visit_formal_parameter_list(&n.parameters)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_function_expr(&mut self, n: &'ast FunctionExpr) -> Result<Self::Output, Self::Error> {
        self.walk_function_expr(n)
    }
    fn walk_function_expr(&mut self, n: &'ast FunctionExpr) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &n.name {
            self.visit_sym(name)?;
        }
        self.visit_formal_parameter_list(&n.parameters)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_get_const_field(
        &mut self,
        n: &'ast GetConstField,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_get_const_field(n)
    }
    fn walk_get_const_field(
        &mut self,
        n: &'ast GetConstField,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.obj)?;
        self.visit_sym(&n.field)?;
        Self::get_default_ok()
    }

    fn visit_get_private_field(
        &mut self,
        n: &'ast GetPrivateField,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_get_private_field(n)
    }
    fn walk_get_private_field(
        &mut self,
        n: &'ast GetPrivateField,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.obj)?;
        self.visit_sym(&n.field)?;
        Self::get_default_ok()
    }

    fn visit_get_field(&mut self, n: &'ast GetField) -> Result<Self::Output, Self::Error> {
        self.walk_get_field(n)
    }
    fn walk_get_field(&mut self, n: &'ast GetField) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.obj)?;
        self.visit_node(&n.field)?;
        Self::get_default_ok()
    }

    fn visit_get_super_field(
        &mut self,
        n: &'ast GetSuperField,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_get_super_field(n)
    }
    fn walk_get_super_field(
        &mut self,
        n: &'ast GetSuperField,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            GetSuperField::Const(sym) => self.visit_sym(sym),
            GetSuperField::Expr(n) => self.visit_node(n),
        }
    }

    fn visit_for_loop(&mut self, n: &'ast ForLoop) -> Result<Self::Output, Self::Error> {
        self.walk_for_loop(n)
    }
    fn walk_for_loop(&mut self, n: &'ast ForLoop) -> Result<Self::Output, Self::Error> {
        if let Some(init) = &n.inner.init {
            self.visit_node(init)?;
        }
        if let Some(condition) = &n.inner.condition {
            self.visit_node(condition)?;
        }
        if let Some(final_expr) = &n.inner.final_expr {
            self.visit_node(final_expr)?;
        }
        self.visit_node(&n.inner.body)?;
        if let Some(name) = &n.label {
            self.visit_sym(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_for_in_loop(&mut self, n: &'ast ForInLoop) -> Result<Self::Output, Self::Error> {
        self.walk_for_in_loop(n)
    }
    fn walk_for_in_loop(&mut self, n: &'ast ForInLoop) -> Result<Self::Output, Self::Error> {
        self.visit_iterable_loop_initializer(&n.init)?;
        self.visit_node(&n.expr)?;
        self.visit_node(&n.body)?;
        if let Some(name) = &n.label {
            self.visit_sym(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_for_of_loop(&mut self, n: &'ast ForOfLoop) -> Result<Self::Output, Self::Error> {
        self.walk_for_of_loop(n)
    }
    fn walk_for_of_loop(&mut self, n: &'ast ForOfLoop) -> Result<Self::Output, Self::Error> {
        self.visit_iterable_loop_initializer(&n.init)?;
        self.visit_node(&n.iterable)?;
        self.visit_node(&n.body)?;
        if let Some(name) = &n.label {
            self.visit_sym(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_if(&mut self, n: &'ast If) -> Result<Self::Output, Self::Error> {
        self.walk_if(n)
    }
    fn walk_if(&mut self, n: &'ast If) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.cond)?;
        self.visit_node(&n.body)?;
        if let Some(else_node) = &n.else_node {
            self.visit_node(else_node)?;
        }
        Self::get_default_ok()
    }

    fn visit_identifier(&mut self, n: &'ast Identifier) -> Result<Self::Output, Self::Error> {
        self.walk_identifier(n)
    }
    fn walk_identifier(&mut self, n: &'ast Identifier) -> Result<Self::Output, Self::Error> {
        self.visit_sym(&n.ident)?;
        Self::get_default_ok()
    }

    fn visit_new(&mut self, n: &'ast New) -> Result<Self::Output, Self::Error> {
        self.walk_new(n)
    }
    fn walk_new(&mut self, n: &'ast New) -> Result<Self::Output, Self::Error> {
        self.visit_call(&n.call)?;
        Self::get_default_ok()
    }

    fn visit_object(&mut self, n: &'ast Object) -> Result<Self::Output, Self::Error> {
        self.walk_object(n)
    }
    fn walk_object(&mut self, n: &'ast Object) -> Result<Self::Output, Self::Error> {
        for pd in n.properties.iter() {
            self.visit_property_definition(pd)?;
        }
        Self::get_default_ok()
    }

    fn visit_return(&mut self, n: &'ast Return) -> Result<Self::Output, Self::Error> {
        self.walk_return(n)
    }
    fn walk_return(&mut self, n: &'ast Return) -> Result<Self::Output, Self::Error> {
        if let Some(expr) = &n.expr {
            self.visit_node(expr)?;
        }
        if let Some(name) = &n.label {
            self.visit_sym(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_switch(&mut self, n: &'ast Switch) -> Result<Self::Output, Self::Error> {
        self.walk_switch(n)
    }
    fn walk_switch(&mut self, n: &'ast Switch) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.val)?;
        for case in n.cases.iter() {
            self.visit_case(case)?;
        }
        if let Some(default) = &n.default {
            self.visit_statement_list(default)?;
        }
        Self::get_default_ok()
    }

    fn visit_spread(&mut self, n: &'ast Spread) -> Result<Self::Output, Self::Error> {
        self.walk_spread(n)
    }
    fn walk_spread(&mut self, n: &'ast Spread) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.val)?;
        Self::get_default_ok()
    }

    fn visit_tagged_template(
        &mut self,
        n: &'ast TaggedTemplate,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_tagged_template(n)
    }
    fn walk_tagged_template(
        &mut self,
        n: &'ast TaggedTemplate,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.tag)?;
        for raw in n.raws.iter() {
            self.visit_sym(raw)?;
        }
        for cooked in n.cookeds.iter().flatten() {
            self.visit_sym(cooked)?;
        }
        for expr in n.exprs.iter() {
            self.visit_node(expr)?;
        }
        Self::get_default_ok()
    }

    fn visit_template_lit(&mut self, n: &'ast TemplateLit) -> Result<Self::Output, Self::Error> {
        self.walk_template_lit(n)
    }
    fn walk_template_lit(&mut self, n: &'ast TemplateLit) -> Result<Self::Output, Self::Error> {
        for te in n.elements.iter() {
            self.visit_template_element(te)?;
        }
        Self::get_default_ok()
    }

    fn visit_throw(&mut self, n: &'ast Throw) -> Result<Self::Output, Self::Error> {
        self.walk_throw(n)
    }
    fn walk_throw(&mut self, n: &'ast Throw) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.expr)?;
        Self::get_default_ok()
    }

    fn visit_try(&mut self, n: &'ast Try) -> Result<Self::Output, Self::Error> {
        self.walk_try(n)
    }
    fn walk_try(&mut self, n: &'ast Try) -> Result<Self::Output, Self::Error> {
        self.visit_block(&n.block)?;
        if let Some(catch) = &n.catch {
            self.visit_catch(catch)?;
        }
        if let Some(finally) = &n.finally {
            self.visit_finally(finally)?;
        }
        Self::get_default_ok()
    }

    fn visit_unary_op(&mut self, n: &'ast UnaryOp) -> Result<Self::Output, Self::Error> {
        self.walk_unary_op(n)
    }
    fn walk_unary_op(&mut self, n: &'ast UnaryOp) -> Result<Self::Output, Self::Error> {
        self.visit_raw_unary_op(&n.op)?;
        self.visit_node(&n.target)?;
        Self::get_default_ok()
    }

    fn visit_declaration_list(
        &mut self,
        n: &'ast DeclarationList,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_list(n)
    }
    fn walk_declaration_list(
        &mut self,
        n: &'ast DeclarationList,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            DeclarationList::Const(decls)
            | DeclarationList::Let(decls)
            | DeclarationList::Var(decls) => {
                for decl in decls.iter() {
                    self.visit_declaration(decl)?;
                }
            }
        }
        Self::get_default_ok()
    }

    fn visit_while_loop(&mut self, n: &'ast WhileLoop) -> Result<Self::Output, Self::Error> {
        self.walk_while_loop(n)
    }
    fn walk_while_loop(&mut self, n: &'ast WhileLoop) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.cond)?;
        self.visit_node(&n.body)?;
        if let Some(name) = &n.label {
            self.visit_sym(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_yield(&mut self, n: &'ast Yield) -> Result<Self::Output, Self::Error> {
        self.walk_yield(n)
    }
    fn walk_yield(&mut self, n: &'ast Yield) -> Result<Self::Output, Self::Error> {
        if let Some(expr) = &n.expr {
            self.visit_node(expr)?;
        }
        Self::get_default_ok()
    }

    fn visit_generator_decl(
        &mut self,
        n: &'ast GeneratorDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_generator_decl(n)
    }
    fn walk_generator_decl(&mut self, n: &'ast GeneratorDecl) -> Result<Self::Output, Self::Error> {
        self.visit_sym(&n.name)?;
        self.visit_formal_parameter_list(&n.parameters)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_generator_expr(
        &mut self,
        n: &'ast GeneratorExpr,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_generator_expr(n)
    }
    fn walk_generator_expr(&mut self, n: &'ast GeneratorExpr) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &n.name {
            self.visit_sym(name)?;
        }
        self.visit_formal_parameter_list(&n.parameters)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_class_decl(&mut self, n: &'ast Class) -> Result<Self::Output, Self::Error> {
        self.walk_class_decl(n)
    }
    fn walk_class_decl(&mut self, n: &'ast Class) -> Result<Self::Output, Self::Error> {
        self.visit_class(n)?;
        Self::get_default_ok()
    }

    fn visit_class_expr(&mut self, n: &'ast Class) -> Result<Self::Output, Self::Error> {
        self.walk_class_expr(n)
    }
    fn walk_class_expr(&mut self, n: &'ast Class) -> Result<Self::Output, Self::Error> {
        self.visit_class(n)?;
        Self::get_default_ok()
    }

    fn visit_class(&mut self, n: &'ast Class) -> Result<Self::Output, Self::Error> {
        self.walk_class(n)
    }
    fn walk_class(&mut self, n: &'ast Class) -> Result<Self::Output, Self::Error> {
        self.visit_sym(&n.name)?;
        if let Some(super_ref) = &n.super_ref {
            self.visit_node(super_ref)?;
        }
        if let Some(constructor) = &n.constructor {
            self.visit_function_expr(constructor)?;
        }
        for elem in n.elements.iter() {
            self.visit_class_element(elem)?;
        }
        Self::get_default_ok()
    }

    fn visit_class_element(&mut self, n: &'ast ClassElement) -> Result<Self::Output, Self::Error> {
        self.walk_class_element(n)
    }
    fn walk_class_element(&mut self, n: &'ast ClassElement) -> Result<Self::Output, Self::Error> {
        match n {
            ClassElement::MethodDefinition(pn, md)
            | ClassElement::StaticMethodDefinition(pn, md) => {
                self.visit_property_name(pn)?;
                self.visit_method_definition(md)?;
            }
            ClassElement::FieldDefinition(pn, fd) | ClassElement::StaticFieldDefinition(pn, fd) => {
                self.visit_property_name(pn)?;
                if let Some(n) = fd {
                    self.visit_node(n)?;
                }
            }
            ClassElement::PrivateMethodDefinition(s, md)
            | ClassElement::PrivateStaticMethodDefinition(s, md) => {
                self.visit_sym(s)?;
                self.visit_method_definition(md)?;
            }
            ClassElement::PrivateFieldDefinition(s, fd)
            | ClassElement::PrivateStaticFieldDefinition(s, fd) => {
                self.visit_sym(s)?;
                if let Some(n) = fd {
                    self.visit_node(n)?;
                }
            }
            ClassElement::StaticBlock(sl) => {
                self.visit_statement_list(sl)?;
            }
        }
        Self::get_default_ok()
    }

    fn visit_super_call(&mut self, n: &'ast SuperCall) -> Result<Self::Output, Self::Error> {
        self.walk_super_call(n)
    }
    fn walk_super_call(&mut self, n: &'ast SuperCall) -> Result<Self::Output, Self::Error> {
        for arg in n.args.iter() {
            self.visit_node(arg)?;
        }
        Self::get_default_ok()
    }

    fn visit_sym(&mut self, _n: &'ast Sym) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_formal_parameter_list(
        &mut self,
        n: &'ast FormalParameterList,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_formal_parameter_list(n)
    }
    fn walk_formal_parameter_list(
        &mut self,
        n: &'ast FormalParameterList,
    ) -> Result<Self::Output, Self::Error> {
        for p in n.parameters.iter() {
            self.visit_formal_parameter(p)?;
        }
        self.visit_formal_parameter_list_flags(&n.flags)?;
        Self::get_default_ok()
    }

    fn visit_statement_list(
        &mut self,
        n: &'ast StatementList,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_statement_list(n)
    }
    fn walk_statement_list(&mut self, n: &'ast StatementList) -> Result<Self::Output, Self::Error> {
        for inner in n.items.iter() {
            self.visit_node(inner)?;
        }
        Self::get_default_ok()
    }

    fn visit_assign_target(&mut self, n: &'ast AssignTarget) -> Result<Self::Output, Self::Error> {
        self.walk_assign_target(n)
    }
    fn walk_assign_target(&mut self, n: &'ast AssignTarget) -> Result<Self::Output, Self::Error> {
        match n {
            AssignTarget::Identifier(ident) => self.visit_identifier(ident),
            AssignTarget::GetPrivateField(gpf) => self.visit_get_private_field(gpf),
            AssignTarget::GetConstField(gcf) => self.visit_get_const_field(gcf),
            AssignTarget::GetField(gf) => self.visit_get_field(gf),
            AssignTarget::DeclarationPattern(dp) => self.visit_declaration_pattern(dp),
        }
    }

    fn visit_raw_binop(&mut self, n: &'ast op::BinOp) -> Result<Self::Output, Self::Error> {
        self.walk_raw_binop(n)
    }
    fn walk_raw_binop(&mut self, n: &'ast op::BinOp) -> Result<Self::Output, Self::Error> {
        match n {
            op::BinOp::Num(op) => self.visit_raw_num_op(op),
            op::BinOp::Bit(op) => self.visit_raw_bit_op(op),
            op::BinOp::Comp(op) => self.visit_raw_comp_op(op),
            op::BinOp::Log(op) => self.visit_raw_log_op(op),
            op::BinOp::Assign(op) => self.visit_raw_assign_op(op),
            op::BinOp::Comma => Self::get_default_ok(),
        }
    }

    fn visit_declaration(&mut self, n: &'ast Declaration) -> Result<Self::Output, Self::Error> {
        self.walk_declaration(n)
    }
    fn walk_declaration(&mut self, n: &'ast Declaration) -> Result<Self::Output, Self::Error> {
        match n {
            Declaration::Identifier { ident, init } => {
                self.visit_identifier(ident)?;
                if let Some(init) = init {
                    self.visit_node(init)?;
                }
                Self::get_default_ok()
            }
            Declaration::Pattern(dp) => self.visit_declaration_pattern(dp),
        }
    }

    fn visit_iterable_loop_initializer(
        &mut self,
        n: &'ast IterableLoopInitializer,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_iterable_loop_initializer(n)
    }
    fn walk_iterable_loop_initializer(
        &mut self,
        n: &'ast IterableLoopInitializer,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            IterableLoopInitializer::Identifier(ident) => self.visit_identifier(ident),
            IterableLoopInitializer::Var(decl)
            | IterableLoopInitializer::Let(decl)
            | IterableLoopInitializer::Const(decl) => self.visit_declaration(decl),
            IterableLoopInitializer::DeclarationPattern(dp) => self.visit_declaration_pattern(dp),
        }
    }

    fn visit_property_definition(
        &mut self,
        n: &'ast PropertyDefinition,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_property_definition(n)
    }
    fn walk_property_definition(
        &mut self,
        n: &'ast PropertyDefinition,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            PropertyDefinition::IdentifierReference(s) => self.visit_sym(s),
            PropertyDefinition::Property(pn, inner) => {
                self.visit_property_name(pn)?;
                self.visit_node(inner)?;
                Self::get_default_ok()
            }
            PropertyDefinition::MethodDefinition(md, pn) => {
                self.visit_method_definition(md)?;
                self.visit_property_name(pn)?;
                Self::get_default_ok()
            }
            PropertyDefinition::SpreadObject(inner) => self.visit_node(inner),
        }
    }

    fn visit_case(&mut self, n: &'ast Case) -> Result<Self::Output, Self::Error> {
        self.walk_case(n)
    }
    fn walk_case(&mut self, n: &'ast Case) -> Result<Self::Output, Self::Error> {
        self.visit_node(&n.condition)?;
        self.visit_statement_list(&n.body)?;
        Self::get_default_ok()
    }

    fn visit_template_element(
        &mut self,
        n: &'ast TemplateElement,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_template_element(n)
    }
    fn walk_template_element(
        &mut self,
        n: &'ast TemplateElement,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            TemplateElement::String(s) => self.visit_sym(s),
            TemplateElement::Expr(inner) => self.visit_node(inner),
        }
    }

    fn visit_catch(&mut self, n: &'ast Catch) -> Result<Self::Output, Self::Error> {
        self.walk_catch(n)
    }
    fn walk_catch(&mut self, n: &'ast Catch) -> Result<Self::Output, Self::Error> {
        if let Some(parameter) = &n.parameter {
            self.visit_declaration(parameter)?;
        }
        self.visit_block(&n.block)?;
        Self::get_default_ok()
    }

    fn visit_finally(&mut self, n: &'ast Finally) -> Result<Self::Output, Self::Error> {
        self.walk_finally(n)
    }
    fn walk_finally(&mut self, n: &'ast Finally) -> Result<Self::Output, Self::Error> {
        self.visit_block(&n.block)?;
        Self::get_default_ok()
    }

    fn visit_raw_unary_op(&mut self, _n: &'ast op::UnaryOp) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_formal_parameter(
        &mut self,
        n: &'ast FormalParameter,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_formal_parameter(n)
    }
    fn walk_formal_parameter(
        &mut self,
        n: &'ast FormalParameter,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_declaration(&n.declaration)?;
        Self::get_default_ok()
    }

    fn visit_formal_parameter_list_flags(
        &mut self,
        _n: &'ast FormalParameterListFlags,
    ) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_declaration_pattern(
        &mut self,
        n: &'ast DeclarationPattern,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_pattern(n)
    }
    fn walk_declaration_pattern(
        &mut self,
        n: &'ast DeclarationPattern,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            DeclarationPattern::Object(o) => self.visit_declaration_pattern_object(o),
            DeclarationPattern::Array(a) => self.visit_declaration_pattern_array(a),
        }
    }

    fn visit_raw_num_op(&mut self, _n: &'ast op::NumOp) -> Result<Self::Output, Self::Error> {
        Self::get_default_ok()
    }

    fn visit_raw_bit_op(&mut self, _n: &'ast op::BitOp) -> Result<Self::Output, Self::Error> {
        Self::get_default_ok()
    }

    fn visit_raw_comp_op(&mut self, _n: &'ast op::CompOp) -> Result<Self::Output, Self::Error> {
        Self::get_default_ok()
    }

    fn visit_raw_log_op(&mut self, _n: &'ast op::LogOp) -> Result<Self::Output, Self::Error> {
        Self::get_default_ok()
    }

    fn visit_raw_assign_op(&mut self, _n: &'ast op::AssignOp) -> Result<Self::Output, Self::Error> {
        Self::get_default_ok()
    }

    fn visit_property_name(&mut self, n: &'ast PropertyName) -> Result<Self::Output, Self::Error> {
        self.walk_property_name(n)
    }
    fn walk_property_name(&mut self, n: &'ast PropertyName) -> Result<Self::Output, Self::Error> {
        match n {
            PropertyName::Literal(s) => self.visit_sym(s),
            PropertyName::Computed(inner) => self.visit_node(inner),
        }
    }

    fn visit_method_definition(
        &mut self,
        n: &'ast MethodDefinition,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_method_definition(n)
    }
    fn walk_method_definition(
        &mut self,
        n: &'ast MethodDefinition,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            MethodDefinition::Get(fe)
            | MethodDefinition::Set(fe)
            | MethodDefinition::Ordinary(fe) => self.visit_function_expr(fe),
            MethodDefinition::Generator(ge) => self.visit_generator_expr(ge),
            MethodDefinition::AsyncGenerator(age) => self.visit_async_generator_expr(age),
            MethodDefinition::Async(afe) => self.visit_async_function_expr(afe),
        }
    }

    fn visit_declaration_pattern_object(
        &mut self,
        n: &'ast DeclarationPatternObject,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_pattern_object(n)
    }
    fn walk_declaration_pattern_object(
        &mut self,
        n: &'ast DeclarationPatternObject,
    ) -> Result<Self::Output, Self::Error> {
        for binding in &n.bindings {
            self.visit_binding_pattern_type_object(binding)?;
        }
        if let Some(init) = &n.init {
            self.visit_node(init)?;
        }
        Self::get_default_ok()
    }

    fn visit_declaration_pattern_array(
        &mut self,
        n: &'ast DeclarationPatternArray,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_pattern_array(n)
    }
    fn walk_declaration_pattern_array(
        &mut self,
        n: &'ast DeclarationPatternArray,
    ) -> Result<Self::Output, Self::Error> {
        for binding in &n.bindings {
            self.visit_binding_pattern_type_array(binding)?;
        }
        if let Some(init) = &n.init {
            self.visit_node(init)?;
        }
        Self::get_default_ok()
    }

    fn visit_binding_pattern_type_object(
        &mut self,
        n: &'ast BindingPatternTypeObject,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_binding_pattern_type_object(n)
    }
    fn walk_binding_pattern_type_object(
        &mut self,
        n: &'ast BindingPatternTypeObject,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            BindingPatternTypeObject::Empty => {}
            BindingPatternTypeObject::SingleName {
                ident,
                property_name,
                default_init,
            } => {
                self.visit_sym(ident)?;
                self.visit_property_name(property_name)?;
                if let Some(init) = default_init {
                    self.visit_node(init)?;
                }
            }
            BindingPatternTypeObject::RestProperty {
                ident,
                excluded_keys,
            } => {
                self.visit_sym(ident)?;
                for key in excluded_keys.iter() {
                    self.visit_sym(key)?;
                }
            }
            BindingPatternTypeObject::RestGetConstField {
                get_const_field,
                excluded_keys,
            } => {
                self.visit_get_const_field(get_const_field)?;
                for key in excluded_keys.iter() {
                    self.visit_sym(key)?;
                }
            }
            BindingPatternTypeObject::BindingPattern {
                ident,
                pattern,
                default_init,
            } => {
                self.visit_property_name(ident)?;
                self.visit_declaration_pattern(pattern)?;
                if let Some(init) = default_init {
                    self.visit_node(init)?;
                }
            }
        }
        Self::get_default_ok()
    }

    fn visit_binding_pattern_type_array(
        &mut self,
        n: &'ast BindingPatternTypeArray,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_binding_pattern_type_array(n)
    }
    fn walk_binding_pattern_type_array(
        &mut self,
        n: &'ast BindingPatternTypeArray,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            BindingPatternTypeArray::SingleName {
                ident,
                default_init,
            } => {
                self.visit_sym(ident)?;
                if let Some(init) = default_init {
                    self.visit_node(init)?;
                }
                Self::get_default_ok()
            }
            BindingPatternTypeArray::GetField { get_field }
            | BindingPatternTypeArray::GetFieldRest { get_field } => {
                self.visit_get_field(get_field)
            }
            BindingPatternTypeArray::GetConstField { get_const_field }
            | BindingPatternTypeArray::GetConstFieldRest { get_const_field } => {
                self.visit_get_const_field(get_const_field)
            }
            BindingPatternTypeArray::BindingPattern { pattern }
            | BindingPatternTypeArray::BindingPatternRest { pattern } => {
                self.visit_declaration_pattern(pattern)
            }
            BindingPatternTypeArray::SingleNameRest { ident } => self.visit_sym(ident),
            BindingPatternTypeArray::Empty | BindingPatternTypeArray::Elision => {
                Self::get_default_ok()
            }
        }
    }

    fn visit_node_mut(&mut self, n: &'ast mut Node) -> Result<Self::Output, Self::Error> {
        self.walk_node_mut(n)
    }
    fn walk_node_mut(&mut self, n: &'ast mut Node) -> Result<Self::Output, Self::Error> {
        match n {
            Node::ArrayDecl(n) => self.visit_array_decl_mut(n),
            Node::ArrowFunctionDecl(n) => self.visit_arrow_function_decl_mut(n),
            Node::Assign(n) => self.visit_assign_mut(n),
            Node::AsyncFunctionDecl(n) => self.visit_async_function_decl_mut(n),
            Node::AsyncFunctionExpr(n) => self.visit_async_function_expr_mut(n),
            Node::AsyncGeneratorExpr(n) => self.visit_async_generator_expr_mut(n),
            Node::AsyncGeneratorDecl(n) => self.visit_async_generator_decl_mut(n),
            Node::AwaitExpr(n) => self.visit_await_expr_mut(n),
            Node::BinOp(n) => self.visit_bin_op_mut(n),
            Node::Block(n) => self.visit_block_mut(n),
            Node::Break(n) => self.visit_break_mut(n),
            Node::Call(n) => self.visit_call_mut(n),
            Node::ConditionalOp(n) => self.visit_conditional_op_mut(n),
            Node::Const(n) => self.visit_const_mut(n),
            Node::ConstDeclList(n) | Node::LetDeclList(n) | Node::VarDeclList(n) => {
                self.visit_declaration_list_mut(n)
            }
            Node::Continue(n) => self.visit_continue_mut(n),
            Node::DoWhileLoop(n) => self.visit_do_while_loop_mut(n),
            Node::FunctionDecl(n) => self.visit_function_decl_mut(n),
            Node::FunctionExpr(n) => self.visit_function_expr_mut(n),
            Node::GetConstField(n) => self.visit_get_const_field_mut(n),
            Node::GetPrivateField(n) => self.visit_get_private_field_mut(n),
            Node::GetField(n) => self.visit_get_field_mut(n),
            Node::GetSuperField(n) => self.visit_get_super_field_mut(n),
            Node::ForLoop(n) => self.visit_for_loop_mut(n),
            Node::ForInLoop(n) => self.visit_for_in_loop_mut(n),
            Node::ForOfLoop(n) => self.visit_for_of_loop_mut(n),
            Node::If(n) => self.visit_if_mut(n),
            Node::Identifier(n) => self.visit_identifier_mut(n),
            Node::New(n) => self.visit_new_mut(n),
            Node::Object(n) => self.visit_object_mut(n),
            Node::Return(n) => self.visit_return_mut(n),
            Node::Switch(n) => self.visit_switch_mut(n),
            Node::Spread(n) => self.visit_spread_mut(n),
            Node::TaggedTemplate(n) => self.visit_tagged_template_mut(n),
            Node::TemplateLit(n) => self.visit_template_lit_mut(n),
            Node::Throw(n) => self.visit_throw_mut(n),
            Node::Try(n) => self.visit_try_mut(n),
            Node::UnaryOp(n) => self.visit_unary_op_mut(n),
            Node::WhileLoop(n) => self.visit_while_loop_mut(n),
            Node::Yield(n) => self.visit_yield_mut(n),
            Node::GeneratorDecl(n) => self.visit_generator_decl_mut(n),
            Node::GeneratorExpr(n) => self.visit_generator_expr_mut(n),
            Node::ClassDecl(n) => self.visit_class_decl_mut(n),
            Node::ClassExpr(n) => self.visit_class_expr_mut(n),
            Node::SuperCall(n) => self.visit_super_call_mut(n),
            Node::Empty | Node::This => Self::get_default_ok(),
        }
    }

    fn visit_array_decl_mut(
        &mut self,
        n: &'ast mut ArrayDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_array_decl_mut(n)
    }
    fn walk_array_decl_mut(&mut self, n: &'ast mut ArrayDecl) -> Result<Self::Output, Self::Error> {
        for inner in n.arr.iter_mut() {
            self.visit_node_mut(inner)?;
        }
        Self::get_default_ok()
    }

    fn visit_arrow_function_decl_mut(
        &mut self,
        n: &'ast mut ArrowFunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_arrow_function_decl_mut(n)
    }
    fn walk_arrow_function_decl_mut(
        &mut self,
        n: &'ast mut ArrowFunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &mut n.name {
            self.visit_sym_mut(name)?;
        }
        self.visit_formal_parameter_list_mut(&mut n.params)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_assign_mut(&mut self, n: &'ast mut Assign) -> Result<Self::Output, Self::Error> {
        self.walk_assign_mut(n)
    }
    fn walk_assign_mut(&mut self, n: &'ast mut Assign) -> Result<Self::Output, Self::Error> {
        self.visit_assign_target_mut(&mut n.lhs)?;
        self.visit_node_mut(&mut n.rhs)?;
        Self::get_default_ok()
    }

    fn visit_async_function_expr_mut(
        &mut self,
        n: &'ast mut AsyncFunctionExpr,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_async_function_expr_mut(n)
    }
    fn walk_async_function_expr_mut(
        &mut self,
        n: &'ast mut AsyncFunctionExpr,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &mut n.name {
            self.visit_sym_mut(name)?;
        }
        self.visit_formal_parameter_list_mut(&mut n.parameters)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_async_function_decl_mut(
        &mut self,
        n: &'ast mut AsyncFunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_async_function_decl_mut(n)
    }
    fn walk_async_function_decl_mut(
        &mut self,
        n: &'ast mut AsyncFunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_sym_mut(&mut n.name)?;
        self.visit_formal_parameter_list_mut(&mut n.parameters)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_async_generator_expr_mut(
        &mut self,
        n: &'ast mut AsyncGeneratorExpr,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_async_generator_expr_mut(n)
    }
    fn walk_async_generator_expr_mut(
        &mut self,
        n: &'ast mut AsyncGeneratorExpr,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &mut n.name {
            self.visit_sym_mut(name)?;
        }
        self.visit_formal_parameter_list_mut(&mut n.parameters)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_async_generator_decl_mut(
        &mut self,
        n: &'ast mut AsyncGeneratorDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_async_generator_decl_mut(n)
    }
    fn walk_async_generator_decl_mut(
        &mut self,
        n: &'ast mut AsyncGeneratorDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_sym_mut(&mut n.name)?;
        self.visit_formal_parameter_list_mut(&mut n.parameters)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_await_expr_mut(
        &mut self,
        n: &'ast mut AwaitExpr,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_await_expr_mut(n)
    }
    fn walk_await_expr_mut(&mut self, n: &'ast mut AwaitExpr) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.expr)?;
        Self::get_default_ok()
    }

    fn visit_bin_op_mut(&mut self, n: &'ast mut BinOp) -> Result<Self::Output, Self::Error> {
        self.walk_bin_op_mut(n)
    }
    fn walk_bin_op_mut(&mut self, n: &'ast mut BinOp) -> Result<Self::Output, Self::Error> {
        self.visit_raw_binop_mut(&mut n.op)?;
        self.visit_node_mut(&mut n.lhs)?;
        self.visit_node_mut(&mut n.rhs)?;
        Self::get_default_ok()
    }

    fn visit_block_mut(&mut self, n: &'ast mut Block) -> Result<Self::Output, Self::Error> {
        self.walk_block_mut(n)
    }
    fn walk_block_mut(&mut self, n: &'ast mut Block) -> Result<Self::Output, Self::Error> {
        self.visit_statement_list_mut(&mut n.statements)?;
        Self::get_default_ok()
    }

    fn visit_break_mut(&mut self, n: &'ast mut Break) -> Result<Self::Output, Self::Error> {
        self.walk_break_mut(n)
    }
    fn walk_break_mut(&mut self, n: &'ast mut Break) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &mut n.label {
            self.visit_sym_mut(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_call_mut(&mut self, n: &'ast mut Call) -> Result<Self::Output, Self::Error> {
        self.walk_call_mut(n)
    }
    fn walk_call_mut(&mut self, n: &'ast mut Call) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.expr)?;
        for inner in n.args.iter_mut() {
            self.visit_node_mut(inner)?;
        }
        Self::get_default_ok()
    }

    fn visit_conditional_op_mut(
        &mut self,
        n: &'ast mut ConditionalOp,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_conditional_op_mut(n)
    }
    fn walk_conditional_op_mut(
        &mut self,
        n: &'ast mut ConditionalOp,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.condition)?;
        self.visit_node_mut(&mut n.if_true)?;
        self.visit_node_mut(&mut n.if_false)?;
        Self::get_default_ok()
    }

    fn visit_const_mut(&mut self, n: &'ast mut Const) -> Result<Self::Output, Self::Error> {
        self.walk_const_mut(n)
    }
    fn walk_const_mut(&mut self, n: &'ast mut Const) -> Result<Self::Output, Self::Error> {
        if let Const::String(s) = n {
            self.visit_sym_mut(s)?;
        }
        Self::get_default_ok()
    }

    fn visit_continue_mut(&mut self, n: &'ast mut Continue) -> Result<Self::Output, Self::Error> {
        self.walk_continue_mut(n)
    }
    fn walk_continue_mut(&mut self, n: &'ast mut Continue) -> Result<Self::Output, Self::Error> {
        if let Some(s) = &mut n.label {
            self.visit_sym_mut(s)?;
        }
        Self::get_default_ok()
    }

    fn visit_do_while_loop_mut(
        &mut self,
        n: &'ast mut DoWhileLoop,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_do_while_loop_mut(n)
    }
    fn walk_do_while_loop_mut(
        &mut self,
        n: &'ast mut DoWhileLoop,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.body)?;
        self.visit_node_mut(&mut n.cond)?;
        if let Some(name) = &mut n.label {
            self.visit_sym_mut(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_function_decl_mut(
        &mut self,
        n: &'ast mut FunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_function_decl_mut(n)
    }
    fn walk_function_decl_mut(
        &mut self,
        n: &'ast mut FunctionDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_sym_mut(&mut n.name)?;
        self.visit_formal_parameter_list_mut(&mut n.parameters)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_function_expr_mut(
        &mut self,
        n: &'ast mut FunctionExpr,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_function_expr_mut(n)
    }
    fn walk_function_expr_mut(
        &mut self,
        n: &'ast mut FunctionExpr,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &mut n.name {
            self.visit_sym_mut(name)?;
        }
        self.visit_formal_parameter_list_mut(&mut n.parameters)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_get_const_field_mut(
        &mut self,
        n: &'ast mut GetConstField,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_get_const_field_mut(n)
    }
    fn walk_get_const_field_mut(
        &mut self,
        n: &'ast mut GetConstField,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.obj)?;
        self.visit_sym_mut(&mut n.field)?;
        Self::get_default_ok()
    }

    fn visit_get_private_field_mut(
        &mut self,
        n: &'ast mut GetPrivateField,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_get_private_field_mut(n)
    }
    fn walk_get_private_field_mut(
        &mut self,
        n: &'ast mut GetPrivateField,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.obj)?;
        self.visit_sym_mut(&mut n.field)?;
        Self::get_default_ok()
    }

    fn visit_get_field_mut(&mut self, n: &'ast mut GetField) -> Result<Self::Output, Self::Error> {
        self.walk_get_field_mut(n)
    }
    fn walk_get_field_mut(&mut self, n: &'ast mut GetField) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.obj)?;
        self.visit_node_mut(&mut n.field)?;
        Self::get_default_ok()
    }

    fn visit_get_super_field_mut(
        &mut self,
        n: &'ast mut GetSuperField,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_get_super_field_mut(n)
    }
    fn walk_get_super_field_mut(
        &mut self,
        n: &'ast mut GetSuperField,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            GetSuperField::Const(sym) => self.visit_sym_mut(sym),
            GetSuperField::Expr(n) => self.visit_node_mut(n.as_mut()),
        }
    }

    fn visit_for_loop_mut(&mut self, n: &'ast mut ForLoop) -> Result<Self::Output, Self::Error> {
        self.walk_for_loop_mut(n)
    }
    fn walk_for_loop_mut(&mut self, n: &'ast mut ForLoop) -> Result<Self::Output, Self::Error> {
        if let Some(init) = &mut n.inner.init {
            self.visit_node_mut(init)?;
        }
        if let Some(condition) = &mut n.inner.condition {
            self.visit_node_mut(condition)?;
        }
        if let Some(final_expr) = &mut n.inner.final_expr {
            self.visit_node_mut(final_expr)?;
        }
        self.visit_node_mut(&mut n.inner.body)?;
        if let Some(name) = &mut n.label {
            self.visit_sym_mut(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_for_in_loop_mut(
        &mut self,
        n: &'ast mut ForInLoop,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_for_in_loop_mut(n)
    }
    fn walk_for_in_loop_mut(
        &mut self,
        n: &'ast mut ForInLoop,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_iterable_loop_initializer_mut(&mut n.init)?;
        self.visit_node_mut(&mut n.expr)?;
        self.visit_node_mut(&mut n.body)?;
        if let Some(name) = &mut n.label {
            self.visit_sym_mut(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_for_of_loop_mut(
        &mut self,
        n: &'ast mut ForOfLoop,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_for_of_loop_mut(n)
    }
    fn walk_for_of_loop_mut(
        &mut self,
        n: &'ast mut ForOfLoop,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_iterable_loop_initializer_mut(&mut n.init)?;
        self.visit_node_mut(&mut n.iterable)?;
        self.visit_node_mut(&mut n.body)?;
        if let Some(name) = &mut n.label {
            self.visit_sym_mut(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_if_mut(&mut self, n: &'ast mut If) -> Result<Self::Output, Self::Error> {
        self.walk_if_mut(n)
    }
    fn walk_if_mut(&mut self, n: &'ast mut If) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.cond)?;
        self.visit_node_mut(&mut n.body)?;
        if let Some(else_node) = &mut n.else_node {
            self.visit_node_mut(else_node.as_mut())?;
        }
        Self::get_default_ok()
    }

    fn visit_identifier_mut(
        &mut self,
        n: &'ast mut Identifier,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_identifier_mut(n)
    }
    fn walk_identifier_mut(
        &mut self,
        n: &'ast mut Identifier,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_sym_mut(&mut n.ident)?;
        Self::get_default_ok()
    }

    fn visit_new_mut(&mut self, n: &'ast mut New) -> Result<Self::Output, Self::Error> {
        self.walk_new_mut(n)
    }
    fn walk_new_mut(&mut self, n: &'ast mut New) -> Result<Self::Output, Self::Error> {
        self.visit_call_mut(&mut n.call)?;
        Self::get_default_ok()
    }

    fn visit_object_mut(&mut self, n: &'ast mut Object) -> Result<Self::Output, Self::Error> {
        self.walk_object_mut(n)
    }
    fn walk_object_mut(&mut self, n: &'ast mut Object) -> Result<Self::Output, Self::Error> {
        for pd in n.properties.iter_mut() {
            self.visit_property_definition_mut(pd)?;
        }
        Self::get_default_ok()
    }

    fn visit_return_mut(&mut self, n: &'ast mut Return) -> Result<Self::Output, Self::Error> {
        self.walk_return_mut(n)
    }
    fn walk_return_mut(&mut self, n: &'ast mut Return) -> Result<Self::Output, Self::Error> {
        if let Some(expr) = &mut n.expr {
            self.visit_node_mut(expr.as_mut())?;
        }
        if let Some(name) = &mut n.label {
            self.visit_sym_mut(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_switch_mut(&mut self, n: &'ast mut Switch) -> Result<Self::Output, Self::Error> {
        self.walk_switch_mut(n)
    }
    fn walk_switch_mut(&mut self, n: &'ast mut Switch) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.val)?;
        for case in n.cases.iter_mut() {
            self.visit_case_mut(case)?;
        }
        if let Some(default) = &mut n.default {
            self.visit_statement_list_mut(default)?;
        }
        Self::get_default_ok()
    }

    fn visit_spread_mut(&mut self, n: &'ast mut Spread) -> Result<Self::Output, Self::Error> {
        self.walk_spread_mut(n)
    }
    fn walk_spread_mut(&mut self, n: &'ast mut Spread) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.val)?;
        Self::get_default_ok()
    }

    fn visit_tagged_template_mut(
        &mut self,
        n: &'ast mut TaggedTemplate,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_tagged_template_mut(n)
    }
    fn walk_tagged_template_mut(
        &mut self,
        n: &'ast mut TaggedTemplate,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.tag)?;
        for raw in n.raws.iter_mut() {
            self.visit_sym_mut(raw)?;
        }
        for cooked in n.cookeds.iter_mut().flatten() {
            self.visit_sym_mut(cooked)?;
        }
        for expr in n.exprs.iter_mut() {
            self.visit_node_mut(expr)?;
        }
        Self::get_default_ok()
    }

    fn visit_template_lit_mut(
        &mut self,
        n: &'ast mut TemplateLit,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_template_lit_mut(n)
    }
    fn walk_template_lit_mut(
        &mut self,
        n: &'ast mut TemplateLit,
    ) -> Result<Self::Output, Self::Error> {
        for te in n.elements.iter_mut() {
            self.visit_template_element_mut(te)?;
        }
        Self::get_default_ok()
    }

    fn visit_throw_mut(&mut self, n: &'ast mut Throw) -> Result<Self::Output, Self::Error> {
        self.walk_throw_mut(n)
    }
    fn walk_throw_mut(&mut self, n: &'ast mut Throw) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.expr)?;
        Self::get_default_ok()
    }

    fn visit_try_mut(&mut self, n: &'ast mut Try) -> Result<Self::Output, Self::Error> {
        self.walk_try_mut(n)
    }
    fn walk_try_mut(&mut self, n: &'ast mut Try) -> Result<Self::Output, Self::Error> {
        self.visit_block_mut(&mut n.block)?;
        if let Some(catch) = &mut n.catch {
            self.visit_catch_mut(catch)?;
        }
        if let Some(finally) = &mut n.finally {
            self.visit_finally_mut(finally)?;
        }
        Self::get_default_ok()
    }

    fn visit_unary_op_mut(&mut self, n: &'ast mut UnaryOp) -> Result<Self::Output, Self::Error> {
        self.walk_unary_op_mut(n)
    }
    fn walk_unary_op_mut(&mut self, n: &'ast mut UnaryOp) -> Result<Self::Output, Self::Error> {
        self.visit_raw_unary_op_mut(&mut n.op)?;
        self.visit_node_mut(&mut n.target)?;
        Self::get_default_ok()
    }

    fn visit_declaration_list_mut(
        &mut self,
        n: &'ast mut DeclarationList,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_list_mut(n)
    }
    fn walk_declaration_list_mut(
        &mut self,
        n: &'ast mut DeclarationList,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            DeclarationList::Const(decls)
            | DeclarationList::Let(decls)
            | DeclarationList::Var(decls) => {
                for decl in decls.iter_mut() {
                    self.visit_declaration_mut(decl)?;
                }
            }
        }
        Self::get_default_ok()
    }

    fn visit_while_loop_mut(
        &mut self,
        n: &'ast mut WhileLoop,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_while_loop_mut(n)
    }
    fn walk_while_loop_mut(&mut self, n: &'ast mut WhileLoop) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.cond)?;
        self.visit_node_mut(&mut n.body)?;
        if let Some(name) = &mut n.label {
            self.visit_sym_mut(name)?;
        }
        Self::get_default_ok()
    }

    fn visit_yield_mut(&mut self, n: &'ast mut Yield) -> Result<Self::Output, Self::Error> {
        self.walk_yield_mut(n)
    }
    fn walk_yield_mut(&mut self, n: &'ast mut Yield) -> Result<Self::Output, Self::Error> {
        if let Some(expr) = &mut n.expr {
            self.visit_node_mut(expr.as_mut())?;
        }
        Self::get_default_ok()
    }

    fn visit_generator_decl_mut(
        &mut self,
        n: &'ast mut GeneratorDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_generator_decl_mut(n)
    }
    fn walk_generator_decl_mut(
        &mut self,
        n: &'ast mut GeneratorDecl,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_sym_mut(&mut n.name)?;
        self.visit_formal_parameter_list_mut(&mut n.parameters)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_generator_expr_mut(
        &mut self,
        n: &'ast mut GeneratorExpr,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_generator_expr_mut(n)
    }
    fn walk_generator_expr_mut(
        &mut self,
        n: &'ast mut GeneratorExpr,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(name) = &mut n.name {
            self.visit_sym_mut(name)?;
        }
        self.visit_formal_parameter_list_mut(&mut n.parameters)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_class_decl_mut(&mut self, n: &'ast mut Class) -> Result<Self::Output, Self::Error> {
        self.walk_class_decl_mut(n)
    }
    fn walk_class_decl_mut(&mut self, n: &'ast mut Class) -> Result<Self::Output, Self::Error> {
        self.visit_class_mut(n)?;
        Self::get_default_ok()
    }

    fn visit_class_expr_mut(&mut self, n: &'ast mut Class) -> Result<Self::Output, Self::Error> {
        self.walk_class_expr_mut(n)
    }
    fn walk_class_expr_mut(&mut self, n: &'ast mut Class) -> Result<Self::Output, Self::Error> {
        self.visit_class_mut(n)?;
        Self::get_default_ok()
    }

    fn visit_class_mut(&mut self, n: &'ast mut Class) -> Result<Self::Output, Self::Error> {
        self.walk_class_mut(n)
    }
    fn walk_class_mut(&mut self, n: &'ast mut Class) -> Result<Self::Output, Self::Error> {
        self.visit_sym_mut(&mut n.name)?;
        if let Some(super_ref) = n.super_ref.as_deref_mut() {
            self.visit_node_mut(super_ref)?;
        }
        if let Some(constructor) = &mut n.constructor {
            self.visit_function_expr_mut(constructor)?;
        }
        for elem in n.elements.iter_mut() {
            self.visit_class_element_mut(elem)?;
        }
        Self::get_default_ok()
    }

    fn visit_class_element_mut(
        &mut self,
        n: &'ast mut ClassElement,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_class_element_mut(n)
    }
    fn walk_class_element_mut(
        &mut self,
        n: &'ast mut ClassElement,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            ClassElement::MethodDefinition(pn, md)
            | ClassElement::StaticMethodDefinition(pn, md) => {
                self.visit_property_name_mut(pn)?;
                self.visit_method_definition_mut(md)?;
            }
            ClassElement::FieldDefinition(pn, fd) | ClassElement::StaticFieldDefinition(pn, fd) => {
                self.visit_property_name_mut(pn)?;
                if let Some(n) = fd {
                    self.visit_node_mut(n)?;
                }
            }
            ClassElement::PrivateMethodDefinition(s, md)
            | ClassElement::PrivateStaticMethodDefinition(s, md) => {
                self.visit_sym_mut(s)?;
                self.visit_method_definition_mut(md)?;
            }
            ClassElement::PrivateFieldDefinition(s, fd)
            | ClassElement::PrivateStaticFieldDefinition(s, fd) => {
                self.visit_sym_mut(s)?;
                if let Some(n) = fd {
                    self.visit_node_mut(n)?;
                }
            }
            ClassElement::StaticBlock(sl) => {
                self.visit_statement_list_mut(sl)?;
            }
        }
        Self::get_default_ok()
    }

    fn visit_super_call_mut(
        &mut self,
        n: &'ast mut SuperCall,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_super_call_mut(n)
    }
    fn walk_super_call_mut(&mut self, n: &'ast mut SuperCall) -> Result<Self::Output, Self::Error> {
        for arg in n.args.iter_mut() {
            self.visit_node_mut(arg)?;
        }
        Self::get_default_ok()
    }

    fn visit_sym_mut(&mut self, _n: &'ast mut Sym) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_formal_parameter_list_mut(
        &mut self,
        n: &'ast mut FormalParameterList,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_formal_parameter_list_mut(n)
    }
    fn walk_formal_parameter_list_mut(
        &mut self,
        n: &'ast mut FormalParameterList,
    ) -> Result<Self::Output, Self::Error> {
        for p in n.parameters.iter_mut() {
            self.visit_formal_parameter_mut(p)?;
        }
        self.visit_formal_parameter_list_flags_mut(&mut n.flags)?;
        Self::get_default_ok()
    }

    fn visit_statement_list_mut(
        &mut self,
        n: &'ast mut StatementList,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_statement_list_mut(n)
    }
    fn walk_statement_list_mut(
        &mut self,
        n: &'ast mut StatementList,
    ) -> Result<Self::Output, Self::Error> {
        for inner in n.items.iter_mut() {
            self.visit_node_mut(inner)?;
        }
        Self::get_default_ok()
    }

    fn visit_assign_target_mut(
        &mut self,
        n: &'ast mut AssignTarget,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_assign_target_mut(n)
    }
    fn walk_assign_target_mut(
        &mut self,
        n: &'ast mut AssignTarget,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            AssignTarget::Identifier(ident) => self.visit_identifier_mut(ident),
            AssignTarget::GetPrivateField(gpf) => self.visit_get_private_field_mut(gpf),
            AssignTarget::GetConstField(gcf) => self.visit_get_const_field_mut(gcf),
            AssignTarget::GetField(gf) => self.visit_get_field_mut(gf),
            AssignTarget::DeclarationPattern(dp) => self.visit_declaration_pattern_mut(dp),
        }
    }

    fn visit_raw_binop_mut(&mut self, n: &'ast mut op::BinOp) -> Result<Self::Output, Self::Error> {
        self.walk_raw_binop_mut(n)
    }
    fn walk_raw_binop_mut(&mut self, n: &'ast mut op::BinOp) -> Result<Self::Output, Self::Error> {
        match n {
            op::BinOp::Num(op) => self.visit_raw_num_op_mut(op),
            op::BinOp::Bit(op) => self.visit_raw_bit_op_mut(op),
            op::BinOp::Comp(op) => self.visit_raw_comp_op_mut(op),
            op::BinOp::Log(op) => self.visit_raw_log_op_mut(op),
            op::BinOp::Assign(op) => self.visit_raw_assign_op_mut(op),
            op::BinOp::Comma => Self::get_default_ok(),
        }
    }

    fn visit_declaration_mut(
        &mut self,
        n: &'ast mut Declaration,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_mut(n)
    }
    fn walk_declaration_mut(
        &mut self,
        n: &'ast mut Declaration,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            Declaration::Identifier { ident, init } => {
                self.visit_identifier_mut(ident)?;
                if let Some(init) = init {
                    self.visit_node_mut(init)?;
                }
                Self::get_default_ok()
            }
            Declaration::Pattern(dp) => self.visit_declaration_pattern_mut(dp),
        }
    }

    fn visit_iterable_loop_initializer_mut(
        &mut self,
        n: &'ast mut IterableLoopInitializer,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_iterable_loop_initializer_mut(n)
    }
    fn walk_iterable_loop_initializer_mut(
        &mut self,
        n: &'ast mut IterableLoopInitializer,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            IterableLoopInitializer::Identifier(ident) => self.visit_identifier_mut(ident),
            IterableLoopInitializer::Var(decl)
            | IterableLoopInitializer::Let(decl)
            | IterableLoopInitializer::Const(decl) => self.visit_declaration_mut(decl),
            IterableLoopInitializer::DeclarationPattern(dp) => {
                self.visit_declaration_pattern_mut(dp)
            }
        }
    }

    fn visit_property_definition_mut(
        &mut self,
        n: &'ast mut PropertyDefinition,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_property_definition_mut(n)
    }
    fn walk_property_definition_mut(
        &mut self,
        n: &'ast mut PropertyDefinition,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            PropertyDefinition::IdentifierReference(s) => self.visit_sym_mut(s),
            PropertyDefinition::Property(pn, inner) => {
                self.visit_property_name_mut(pn)?;
                self.visit_node_mut(inner)?;
                Self::get_default_ok()
            }
            PropertyDefinition::MethodDefinition(md, pn) => {
                self.visit_method_definition_mut(md)?;
                self.visit_property_name_mut(pn)?;
                Self::get_default_ok()
            }
            PropertyDefinition::SpreadObject(inner) => self.visit_node_mut(inner),
        }
    }

    fn visit_case_mut(&mut self, n: &'ast mut Case) -> Result<Self::Output, Self::Error> {
        self.walk_case_mut(n)
    }
    fn walk_case_mut(&mut self, n: &'ast mut Case) -> Result<Self::Output, Self::Error> {
        self.visit_node_mut(&mut n.condition)?;
        self.visit_statement_list_mut(&mut n.body)?;
        Self::get_default_ok()
    }

    fn visit_template_element_mut(
        &mut self,
        n: &'ast mut TemplateElement,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_template_element_mut(n)
    }
    fn walk_template_element_mut(
        &mut self,
        n: &'ast mut TemplateElement,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            TemplateElement::String(s) => self.visit_sym_mut(s),
            TemplateElement::Expr(inner) => self.visit_node_mut(inner),
        }
    }

    fn visit_catch_mut(&mut self, n: &'ast mut Catch) -> Result<Self::Output, Self::Error> {
        self.walk_catch_mut(n)
    }
    fn walk_catch_mut(&mut self, n: &'ast mut Catch) -> Result<Self::Output, Self::Error> {
        if let Some(parameter) = &mut n.parameter {
            self.visit_declaration_mut(parameter.as_mut())?;
        }
        self.visit_block_mut(&mut n.block)?;
        Self::get_default_ok()
    }

    fn visit_finally_mut(&mut self, n: &'ast mut Finally) -> Result<Self::Output, Self::Error> {
        self.walk_finally_mut(n)
    }
    fn walk_finally_mut(&mut self, n: &'ast mut Finally) -> Result<Self::Output, Self::Error> {
        self.visit_block_mut(&mut n.block)?;
        Self::get_default_ok()
    }

    fn visit_raw_unary_op_mut(
        &mut self,
        _n: &'ast mut op::UnaryOp,
    ) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_formal_parameter_mut(
        &mut self,
        n: &'ast mut FormalParameter,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_formal_parameter_mut(n)
    }
    fn walk_formal_parameter_mut(
        &mut self,
        n: &'ast mut FormalParameter,
    ) -> Result<Self::Output, Self::Error> {
        self.visit_declaration_mut(&mut n.declaration)?;
        Self::get_default_ok()
    }

    fn visit_formal_parameter_list_flags_mut(
        &mut self,
        _n: &'ast mut FormalParameterListFlags,
    ) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_declaration_pattern_mut(
        &mut self,
        n: &'ast mut DeclarationPattern,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_pattern_mut(n)
    }
    fn walk_declaration_pattern_mut(
        &mut self,
        n: &'ast mut DeclarationPattern,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            DeclarationPattern::Object(o) => self.visit_declaration_pattern_object_mut(o),
            DeclarationPattern::Array(a) => self.visit_declaration_pattern_array_mut(a),
        }
    }

    fn visit_raw_num_op_mut(
        &mut self,
        _n: &'ast mut op::NumOp,
    ) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_raw_bit_op_mut(
        &mut self,
        _n: &'ast mut op::BitOp,
    ) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_raw_comp_op_mut(
        &mut self,
        _n: &'ast mut op::CompOp,
    ) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_raw_log_op_mut(
        &mut self,
        _n: &'ast mut op::LogOp,
    ) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_raw_assign_op_mut(
        &mut self,
        _n: &'ast mut op::AssignOp,
    ) -> Result<Self::Output, Self::Error> {
        /* do nothing */
        Self::get_default_ok()
    }

    fn visit_property_name_mut(
        &mut self,
        n: &'ast mut PropertyName,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_property_name_mut(n)
    }
    fn walk_property_name_mut(
        &mut self,
        n: &'ast mut PropertyName,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            PropertyName::Literal(s) => self.visit_sym_mut(s),
            PropertyName::Computed(inner) => self.visit_node_mut(inner),
        }
    }

    fn visit_method_definition_mut(
        &mut self,
        n: &'ast mut MethodDefinition,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_method_definition_mut(n)
    }
    fn walk_method_definition_mut(
        &mut self,
        n: &'ast mut MethodDefinition,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            MethodDefinition::Get(fe)
            | MethodDefinition::Set(fe)
            | MethodDefinition::Ordinary(fe) => self.visit_function_expr_mut(fe),
            MethodDefinition::Generator(ge) => self.visit_generator_expr_mut(ge),
            MethodDefinition::AsyncGenerator(age) => self.visit_async_generator_expr_mut(age),
            MethodDefinition::Async(afe) => self.visit_async_function_expr_mut(afe),
        }
    }

    fn visit_declaration_pattern_object_mut(
        &mut self,
        n: &'ast mut DeclarationPatternObject,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_pattern_object_mut(n)
    }
    fn walk_declaration_pattern_object_mut(
        &mut self,
        n: &'ast mut DeclarationPatternObject,
    ) -> Result<Self::Output, Self::Error> {
        for binding in &mut n.bindings {
            self.visit_binding_pattern_type_object_mut(binding)?;
        }
        if let Some(init) = &mut n.init {
            self.visit_node_mut(init)?;
        }
        Self::get_default_ok()
    }

    fn visit_declaration_pattern_array_mut(
        &mut self,
        n: &'ast mut DeclarationPatternArray,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_declaration_pattern_array_mut(n)
    }
    fn walk_declaration_pattern_array_mut(
        &mut self,
        n: &'ast mut DeclarationPatternArray,
    ) -> Result<Self::Output, Self::Error> {
        for binding in &mut n.bindings {
            self.visit_binding_pattern_type_array_mut(binding)?;
        }
        if let Some(init) = &mut n.init {
            self.visit_node_mut(init)?;
        }
        Self::get_default_ok()
    }

    fn visit_binding_pattern_type_object_mut(
        &mut self,
        n: &'ast mut BindingPatternTypeObject,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_binding_pattern_type_object_mut(n)
    }
    fn walk_binding_pattern_type_object_mut(
        &mut self,
        n: &'ast mut BindingPatternTypeObject,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            BindingPatternTypeObject::Empty => {}
            BindingPatternTypeObject::SingleName {
                ident,
                property_name,
                default_init,
            } => {
                self.visit_sym_mut(ident)?;
                self.visit_property_name_mut(property_name)?;
                if let Some(init) = default_init {
                    self.visit_node_mut(init)?;
                }
            }
            BindingPatternTypeObject::RestProperty {
                ident,
                excluded_keys,
            } => {
                self.visit_sym_mut(ident)?;
                for key in excluded_keys.iter_mut() {
                    self.visit_sym_mut(key)?;
                }
            }
            BindingPatternTypeObject::RestGetConstField {
                get_const_field,
                excluded_keys,
            } => {
                self.visit_get_const_field_mut(get_const_field)?;
                for key in excluded_keys.iter_mut() {
                    self.visit_sym_mut(key)?;
                }
            }
            BindingPatternTypeObject::BindingPattern {
                ident,
                pattern,
                default_init,
            } => {
                self.visit_property_name_mut(ident)?;
                self.visit_declaration_pattern_mut(pattern)?;
                if let Some(init) = default_init {
                    self.visit_node_mut(init)?;
                }
            }
        }
        Self::get_default_ok()
    }

    fn visit_binding_pattern_type_array_mut(
        &mut self,
        n: &'ast mut BindingPatternTypeArray,
    ) -> Result<Self::Output, Self::Error> {
        self.walk_binding_pattern_type_array_mut(n)
    }
    fn walk_binding_pattern_type_array_mut(
        &mut self,
        n: &'ast mut BindingPatternTypeArray,
    ) -> Result<Self::Output, Self::Error> {
        match n {
            BindingPatternTypeArray::SingleName {
                ident,
                default_init,
            } => {
                self.visit_sym_mut(ident)?;
                if let Some(init) = default_init {
                    self.visit_node_mut(init)?;
                }
                Self::get_default_ok()
            }
            BindingPatternTypeArray::GetField { get_field }
            | BindingPatternTypeArray::GetFieldRest { get_field } => {
                self.visit_get_field_mut(get_field)
            }
            BindingPatternTypeArray::GetConstField { get_const_field }
            | BindingPatternTypeArray::GetConstFieldRest { get_const_field } => {
                self.visit_get_const_field_mut(get_const_field)
            }
            BindingPatternTypeArray::BindingPattern { pattern }
            | BindingPatternTypeArray::BindingPatternRest { pattern } => {
                self.visit_declaration_pattern_mut(pattern)
            }
            BindingPatternTypeArray::SingleNameRest { ident } => self.visit_sym_mut(ident),
            BindingPatternTypeArray::Empty | BindingPatternTypeArray::Elision => {
                Self::get_default_ok()
            }
        }
    }

    fn get_default_ok() -> Result<Self::Output, Self::Error>;
}
