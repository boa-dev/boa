//! This module implements the `Node` structure, which composes the AST.

mod parameters;

pub mod array;
pub mod await_expr;
pub mod block;
pub mod call;
pub mod conditional;
pub mod declaration;
pub mod field;
pub mod identifier;
pub mod iteration;
pub mod new;
pub mod object;
pub mod operator;
pub mod return_smt;
pub mod spread;
pub mod statement_list;
pub mod switch;
pub mod template;
pub mod throw;
pub mod try_node;
pub mod r#yield;

pub use self::{
    array::ArrayDecl,
    await_expr::AwaitExpr,
    block::Block,
    call::Call,
    conditional::{ConditionalOp, If},
    declaration::{
        async_generator_decl::AsyncGeneratorDecl, async_generator_expr::AsyncGeneratorExpr,
        class_decl::Class, generator_decl::GeneratorDecl, generator_expr::GeneratorExpr,
        ArrowFunctionDecl, AsyncFunctionDecl, AsyncFunctionExpr, Declaration, DeclarationList,
        DeclarationPattern, FunctionDecl, FunctionExpr,
    },
    field::{get_private_field::GetPrivateField, GetConstField, GetField},
    identifier::Identifier,
    iteration::{Break, Continue, DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, WhileLoop},
    new::New,
    object::Object,
    operator::{Assign, BinOp, UnaryOp},
    parameters::{FormalParameter, FormalParameterList},
    r#yield::Yield,
    return_smt::Return,
    spread::Spread,
    statement_list::StatementList,
    switch::{Case, Switch},
    template::{TaggedTemplate, TemplateLit},
    throw::Throw,
    try_node::{Catch, Finally, Try},
};
use self::{
    declaration::class_decl::ClassElement,
    iteration::IterableLoopInitializer,
    object::{MethodDefinition, PropertyDefinition},
};

pub(crate) use self::parameters::FormalParameterListFlags;

use super::Const;
use boa_interner::{Interner, Sym, ToInternedString};
use rustc_hash::FxHashSet;
use std::cmp::Ordering;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

// TODO: This should be split into Expression and Statement.
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// Array declaration node. [More information](./array/struct.ArrayDecl.html).
    ArrayDecl(ArrayDecl),

    /// An arrow function expression node. [More information](./arrow_function/struct.ArrowFunctionDecl.html).
    ArrowFunctionDecl(ArrowFunctionDecl),

    /// An assignment operator node. [More information](./operator/struct.Assign.html).
    Assign(Assign),

    /// An async function declaration node. [More information](./declaration/struct.AsyncFunctionDecl.html).
    AsyncFunctionDecl(AsyncFunctionDecl),

    /// An async function expression node. [More information](./declaration/struct.AsyncFunctionExpr.html).
    AsyncFunctionExpr(AsyncFunctionExpr),

    /// An async generator expression node.
    AsyncGeneratorExpr(AsyncGeneratorExpr),

    /// An async generator declaration node.
    AsyncGeneratorDecl(AsyncGeneratorDecl),

    /// An await expression node. [More information](./await_expr/struct.AwaitExpression.html).
    AwaitExpr(AwaitExpr),

    /// A binary operator node. [More information](./operator/struct.BinOp.html).
    BinOp(BinOp),

    /// A Block node. [More information](./block/struct.Block.html).
    Block(Block),

    /// A break node. [More information](./break/struct.Break.html).
    Break(Break),

    /// A function call. [More information](./expression/struct.Call.html).
    Call(Call),

    /// A javascript conditional operand ( x ? y : z ). [More information](./conditional/struct.ConditionalOp.html).
    ConditionalOp(ConditionalOp),

    /// Literals represent values in JavaScript.
    ///
    /// These are fixed values not variables that you literally provide in your script.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-primary-expression-literals
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Literals
    Const(Const),

    /// A constant declaration list. [More information](./declaration/enum.DeclarationList.html#variant.Const).
    ConstDeclList(DeclarationList),

    /// A continue statement. [More information](./iteration/struct.Continue.html).
    Continue(Continue),

    /// A do ... while statement. [More information](./iteration/struct.DoWhileLoop.html).
    DoWhileLoop(DoWhileLoop),

    /// A function declaration node. [More information](./declaration/struct.FunctionDecl.html).
    FunctionDecl(FunctionDecl),

    /// A function expression node. [More information](./declaration/struct.FunctionExpr.html).
    FunctionExpr(FunctionExpr),

    /// Provides access to an object types' constant properties. [More information](./declaration/struct.GetConstField.html).
    GetConstField(GetConstField),

    /// Provides access to an object types' private properties. [More information](./declaration/struct.GetPrivateField.html).
    GetPrivateField(GetPrivateField),

    /// Provides access to object fields. [More information](./declaration/struct.GetField.html).
    GetField(GetField),

    /// A `for` statement. [More information](./iteration/struct.ForLoop.html).
    ForLoop(ForLoop),

    /// A `for...of` or `for..in` statement. [More information](./iteration/struct.ForIn.html).
    ForInLoop(ForInLoop),

    /// A `for...of` statement. [More information](./iteration/struct.ForOf.html).
    ForOfLoop(ForOfLoop),

    /// An 'if' statement. [More information](./conditional/struct.If.html).
    If(If),

    /// A `let` declaration list. [More information](./declaration/enum.DeclarationList.html#variant.Let).
    LetDeclList(DeclarationList),

    /// A local identifier node. [More information](./identifier/struct.Identifier.html).
    Identifier(Identifier),

    /// A `new` expression. [More information](./expression/struct.New.html).
    New(New),

    /// An object. [More information](./object/struct.Object.html).
    Object(Object),

    /// A return statement. [More information](./object/struct.Return.html).
    Return(Return),

    /// A switch {case} statement. [More information](./switch/struct.Switch.html).
    Switch(Switch),

    /// A spread (...x) statement. [More information](./spread/struct.Spread.html).
    Spread(Spread),

    /// A tagged template. [More information](./template/struct.TaggedTemplate.html).
    TaggedTemplate(Box<TaggedTemplate>),

    /// A template literal. [More information](./template/struct.TemplateLit.html).
    TemplateLit(TemplateLit),

    /// A throw statement. [More information](./throw/struct.Throw.html).
    Throw(Throw),

    /// A `try...catch` node. [More information](./try_node/struct.Try.htl).
    Try(Box<Try>),

    /// The JavaScript `this` keyword refers to the object it belongs to.
    ///
    /// A property of an execution context (global, function or eval) that,
    /// in non–strict mode, is always a reference to an object and in strict
    /// mode can be any value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-this-keyword
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this
    This,

    /// Unary operation node. [More information](./operator/struct.UnaryOp.html)
    UnaryOp(UnaryOp),

    /// Array declaration node. [More information](./declaration/enum.DeclarationList.html#variant.Var).
    VarDeclList(DeclarationList),

    /// A 'while {...}' node. [More information](./iteration/struct.WhileLoop.html).
    WhileLoop(WhileLoop),

    /// A empty node.
    ///
    /// Empty statement do nothing, just return undefined.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-EmptyStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/Empty
    Empty,

    /// A `yield` node. [More information](./yield/struct.Yield.html).
    Yield(Yield),

    /// A generator function declaration node. [More information](./declaration/struct.GeneratorDecl.html).
    GeneratorDecl(GeneratorDecl),

    /// A generator function expression node. [More information](./declaration/struct.GeneratorExpr.html).
    GeneratorExpr(GeneratorExpr),

    /// A class declaration. [More information](./declaration/struct.class_decl.Class.html).
    ClassDecl(Class),

    /// A class declaration. [More information](./declaration/struct.class_decl.Class.html).
    ClassExpr(Class),
}

impl From<Const> for Node {
    fn from(c: Const) -> Self {
        Self::Const(c)
    }
}

impl Node {
    /// Returns a node ordering based on the hoistability of each node.
    #[allow(clippy::match_same_arms)]
    pub(crate) fn hoistable_order(a: &Self, b: &Self) -> Ordering {
        match (a, b) {
            (Node::FunctionDecl(_), Node::FunctionDecl(_)) => Ordering::Equal,
            (_, Node::FunctionDecl(_)) => Ordering::Greater,
            (Node::FunctionDecl(_), _) => Ordering::Less,

            (_, _) => Ordering::Equal,
        }
    }

    /// Creates a `This` AST node.
    pub fn this() -> Self {
        Self::This
    }

    /// Creates a string of the value of the node with the given indentation. For example, an
    /// indent level of 2 would produce this:
    ///
    /// ```js
    ///         function hello() {
    ///             console.log("hello");
    ///         }
    ///         hello();
    ///         a = 2;
    /// ```
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = match *self {
            Self::Block(_) => String::new(),
            _ => "    ".repeat(indentation),
        };

        buf.push_str(&self.to_no_indent_string(interner, indentation));

        buf
    }

    /// Implements the display formatting with indentation.
    ///
    /// This will not prefix the value with any indentation. If you want to prefix this with proper
    /// indents, use [`to_indented_string()`](Self::to_indented_string).
    fn to_no_indent_string(&self, interner: &Interner, indentation: usize) -> String {
        match *self {
            Self::Call(ref expr) => expr.to_interned_string(interner),
            Self::Const(ref c) => c.to_interned_string(interner),
            Self::ConditionalOp(ref cond_op) => cond_op.to_interned_string(interner),
            Self::ForLoop(ref for_loop) => for_loop.to_indented_string(interner, indentation),
            Self::ForOfLoop(ref for_of) => for_of.to_indented_string(interner, indentation),
            Self::ForInLoop(ref for_in) => for_in.to_indented_string(interner, indentation),
            Self::This => "this".to_owned(),
            Self::Try(ref try_catch) => try_catch.to_indented_string(interner, indentation),
            Self::Break(ref break_smt) => break_smt.to_interned_string(interner),
            Self::Continue(ref cont) => cont.to_interned_string(interner),
            Self::Spread(ref spread) => spread.to_interned_string(interner),
            Self::Block(ref block) => block.to_indented_string(interner, indentation),
            Self::Identifier(ref ident) => ident.to_interned_string(interner),
            Self::New(ref expr) => expr.to_interned_string(interner),
            Self::GetConstField(ref get_const_field) => {
                get_const_field.to_interned_string(interner)
            }
            Self::GetPrivateField(ref get_private_field) => {
                get_private_field.to_interned_string(interner)
            }
            Self::GetField(ref get_field) => get_field.to_interned_string(interner),
            Self::WhileLoop(ref while_loop) => while_loop.to_indented_string(interner, indentation),
            Self::DoWhileLoop(ref do_while) => do_while.to_indented_string(interner, indentation),
            Self::If(ref if_smt) => if_smt.to_indented_string(interner, indentation),
            Self::Switch(ref switch) => switch.to_indented_string(interner, indentation),
            Self::Object(ref obj) => obj.to_indented_string(interner, indentation),
            Self::ArrayDecl(ref arr) => arr.to_interned_string(interner),
            Self::VarDeclList(ref list) => list.to_interned_string(interner),
            Self::FunctionDecl(ref decl) => decl.to_indented_string(interner, indentation),
            Self::FunctionExpr(ref expr) => expr.to_indented_string(interner, indentation),
            Self::ArrowFunctionDecl(ref decl) => decl.to_indented_string(interner, indentation),
            Self::BinOp(ref op) => op.to_interned_string(interner),
            Self::UnaryOp(ref op) => op.to_interned_string(interner),
            Self::Return(ref ret) => ret.to_interned_string(interner),
            Self::TaggedTemplate(ref template) => template.to_interned_string(interner),
            Self::TemplateLit(ref template) => template.to_interned_string(interner),
            Self::Throw(ref throw) => throw.to_interned_string(interner),
            Self::Assign(ref op) => op.to_interned_string(interner),
            Self::LetDeclList(ref decl) | Self::ConstDeclList(ref decl) => {
                decl.to_interned_string(interner)
            }
            Self::AsyncFunctionDecl(ref decl) => decl.to_indented_string(interner, indentation),
            Self::AsyncFunctionExpr(ref expr) => expr.to_indented_string(interner, indentation),
            Self::AwaitExpr(ref expr) => expr.to_interned_string(interner),
            Self::Empty => ";".to_owned(),
            Self::Yield(ref y) => y.to_interned_string(interner),
            Self::GeneratorDecl(ref decl) => decl.to_interned_string(interner),
            Self::GeneratorExpr(ref expr) => expr.to_indented_string(interner, indentation),
            Self::AsyncGeneratorExpr(ref expr) => expr.to_indented_string(interner, indentation),
            Self::AsyncGeneratorDecl(ref decl) => decl.to_indented_string(interner, indentation),
            Self::ClassDecl(ref decl) => decl.to_indented_string(interner, indentation),
            Self::ClassExpr(ref expr) => expr.to_indented_string(interner, indentation),
        }
    }

    pub(crate) fn var_declared_names(&self, vars: &mut FxHashSet<Sym>) {
        match self {
            Node::Block(block) => {
                for node in block.items() {
                    node.var_declared_names(vars);
                }
            }
            Node::VarDeclList(DeclarationList::Var(declarations)) => {
                for declaration in declarations.iter() {
                    match declaration {
                        Declaration::Identifier { ident, .. } => {
                            vars.insert(ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                vars.insert(ident);
                            }
                        }
                    }
                }
            }
            Node::If(if_statement) => {
                if_statement.body().var_declared_names(vars);
                if let Some(node) = if_statement.else_node() {
                    node.var_declared_names(vars);
                }
            }
            Node::DoWhileLoop(do_while_loop) => {
                do_while_loop.body().var_declared_names(vars);
            }
            Node::WhileLoop(while_loop) => {
                while_loop.body().var_declared_names(vars);
            }
            Node::ForLoop(for_loop) => {
                if let Some(Node::VarDeclList(DeclarationList::Var(declarations))) = for_loop.init()
                {
                    for declaration in declarations.iter() {
                        match declaration {
                            Declaration::Identifier { ident, .. } => {
                                vars.insert(ident.sym());
                            }
                            Declaration::Pattern(pattern) => {
                                for ident in pattern.idents() {
                                    vars.insert(ident);
                                }
                            }
                        }
                    }
                }
                for_loop.body().var_declared_names(vars);
            }
            Node::ForInLoop(for_in_loop) => {
                if let IterableLoopInitializer::Var(declaration) = for_in_loop.init() {
                    match declaration {
                        Declaration::Identifier { ident, .. } => {
                            vars.insert(ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                vars.insert(ident);
                            }
                        }
                    }
                }
                for_in_loop.body().var_declared_names(vars);
            }
            Node::ForOfLoop(for_of_loop) => {
                if let IterableLoopInitializer::Var(declaration) = for_of_loop.init() {
                    match declaration {
                        Declaration::Identifier { ident, .. } => {
                            vars.insert(ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                vars.insert(ident);
                            }
                        }
                    }
                }
                for_of_loop.body().var_declared_names(vars);
            }
            Node::Switch(switch) => {
                for case in switch.cases() {
                    for node in case.body().items() {
                        node.var_declared_names(vars);
                    }
                }
                if let Some(nodes) = switch.default() {
                    for node in nodes {
                        node.var_declared_names(vars);
                    }
                }
            }
            Node::Try(try_statement) => {
                for node in try_statement.block().items() {
                    node.var_declared_names(vars);
                }
                if let Some(catch) = try_statement.catch() {
                    for node in catch.block().items() {
                        node.var_declared_names(vars);
                    }
                }
                if let Some(finally) = try_statement.finally() {
                    for node in finally.items() {
                        node.var_declared_names(vars);
                    }
                }
            }
            _ => {}
        }
    }

    /// Returns true if the node contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Node::Identifier(ident) if ident.sym() == Sym::ARGUMENTS => return true,
            Node::ArrayDecl(array) => {
                for node in array.as_ref() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::ArrowFunctionDecl(decl) => {
                for node in decl.body().items() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::Assign(assign) => {
                if assign.rhs().contains_arguments() {
                    return true;
                }
            }
            Node::AwaitExpr(r#await) => {
                if r#await.expr().contains_arguments() {
                    return true;
                }
            }
            Node::BinOp(bin_op) => {
                if bin_op.lhs().contains_arguments() || bin_op.rhs().contains_arguments() {
                    return true;
                }
            }
            Node::Block(block) => {
                for node in block.items() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::Call(call) => {
                if call.expr().contains_arguments() {
                    return true;
                }
                for node in call.args() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::ConditionalOp(conditional) => {
                if conditional.cond().contains_arguments() {
                    return true;
                }
                if conditional.if_true().contains_arguments() {
                    return true;
                }
                if conditional.if_false().contains_arguments() {
                    return true;
                }
            }
            Node::DoWhileLoop(do_while_loop) => {
                if do_while_loop.body().contains_arguments() {
                    return true;
                }
                if do_while_loop.cond().contains_arguments() {
                    return true;
                }
            }
            Node::GetConstField(get_const_field) => {
                if get_const_field.obj().contains_arguments() {
                    return true;
                }
            }
            Node::GetPrivateField(get_private_field) => {
                if get_private_field.obj().contains_arguments() {
                    return true;
                }
            }
            Node::GetField(get_field) => {
                if get_field.obj().contains_arguments() {
                    return true;
                }
                if get_field.field().contains_arguments() {
                    return true;
                }
            }
            Node::ForLoop(for_loop) => {
                if let Some(node) = for_loop.init() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
                if let Some(node) = for_loop.condition() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
                if let Some(node) = for_loop.final_expr() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
                if for_loop.body().contains_arguments() {
                    return true;
                }
            }
            Node::ForInLoop(for_in_loop) => {
                match for_in_loop.init() {
                    IterableLoopInitializer::Var(declaration)
                    | IterableLoopInitializer::Let(declaration)
                    | IterableLoopInitializer::Const(declaration) => match declaration {
                        Declaration::Identifier { init, .. } => {
                            if let Some(init) = init {
                                {
                                    if init.contains_arguments() {
                                        return true;
                                    }
                                }
                            }
                        }
                        Declaration::Pattern(pattern) => {
                            if pattern.contains_arguments() {
                                return true;
                            }
                        }
                    },
                    IterableLoopInitializer::DeclarationPattern(pattern) => {
                        if pattern.contains_arguments() {
                            return true;
                        }
                    }
                    IterableLoopInitializer::Identifier(_) => {}
                }
                if for_in_loop.expr().contains_arguments() {
                    return true;
                }
                if for_in_loop.body().contains_arguments() {
                    return true;
                }
            }
            Node::ForOfLoop(for_of_loop) => {
                match for_of_loop.init() {
                    IterableLoopInitializer::Var(declaration)
                    | IterableLoopInitializer::Let(declaration)
                    | IterableLoopInitializer::Const(declaration) => match declaration {
                        Declaration::Identifier { init, .. } => {
                            if let Some(init) = init {
                                {
                                    if init.contains_arguments() {
                                        return true;
                                    }
                                }
                            }
                        }
                        Declaration::Pattern(pattern) => {
                            if pattern.contains_arguments() {
                                return true;
                            }
                        }
                    },
                    IterableLoopInitializer::DeclarationPattern(pattern) => {
                        if pattern.contains_arguments() {
                            return true;
                        }
                    }
                    IterableLoopInitializer::Identifier(_) => {}
                }
                if for_of_loop.iterable().contains_arguments() {
                    return true;
                }
                if for_of_loop.body().contains_arguments() {
                    return true;
                }
            }
            Node::If(r#if) => {
                if r#if.cond().contains_arguments() {
                    return true;
                }
                if r#if.body().contains_arguments() {
                    return true;
                }
                if let Some(node) = r#if.else_node() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::VarDeclList(decl_list)
            | Node::ConstDeclList(decl_list)
            | Node::LetDeclList(decl_list) => match decl_list {
                DeclarationList::Const(declarations)
                | DeclarationList::Let(declarations)
                | DeclarationList::Var(declarations) => {
                    for declaration in declarations.iter() {
                        match declaration {
                            Declaration::Identifier { init, .. } => {
                                if let Some(init) = init {
                                    {
                                        if init.contains_arguments() {
                                            return true;
                                        }
                                    }
                                }
                            }
                            Declaration::Pattern(pattern) => {
                                if pattern.contains_arguments() {
                                    return true;
                                }
                            }
                        }
                    }
                }
            },
            Node::New(new) => {
                if new.expr().contains_arguments() {
                    return true;
                }
                for node in new.args() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::Object(object) => {
                for property in object.properties() {
                    match property {
                        PropertyDefinition::IdentifierReference(ident) => {
                            if *ident == Sym::ARGUMENTS {
                                return true;
                            }
                        }
                        PropertyDefinition::Property(_, node)
                        | PropertyDefinition::SpreadObject(node) => {
                            if node.contains_arguments() {
                                return true;
                            }
                        }
                        PropertyDefinition::MethodDefinition(method, _) => match method {
                            MethodDefinition::Get(function)
                            | MethodDefinition::Set(function)
                            | MethodDefinition::Ordinary(function) => {
                                if let Some(Sym::ARGUMENTS) = function.name() {
                                    return true;
                                }
                            }
                            MethodDefinition::Generator(generator) => {
                                if let Some(Sym::ARGUMENTS) = generator.name() {
                                    return true;
                                }
                            }
                            MethodDefinition::AsyncGenerator(async_generator) => {
                                if let Some(Sym::ARGUMENTS) = async_generator.name() {
                                    return true;
                                }
                            }
                            MethodDefinition::Async(function) => {
                                if let Some(Sym::ARGUMENTS) = function.name() {
                                    return true;
                                }
                            }
                        },
                    }
                }
            }
            Node::Return(r#return) => {
                if let Some(node) = r#return.expr() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::Switch(r#switch) => {
                if r#switch.val().contains_arguments() {
                    return true;
                }
                for case in r#switch.cases() {
                    if case.condition().contains_arguments() {
                        return true;
                    }
                    for node in case.body().items() {
                        if node.contains_arguments() {
                            return true;
                        }
                    }
                }
            }
            Node::Spread(spread) => {
                if spread.val().contains_arguments() {
                    return true;
                }
            }
            Node::TaggedTemplate(tagged_template) => {
                if tagged_template.tag().contains_arguments() {
                    return true;
                }
                for node in tagged_template.exprs() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::TemplateLit(template_lit) => {
                for element in template_lit.elements() {
                    if let template::TemplateElement::Expr(node) = element {
                        if node.contains_arguments() {
                            return false;
                        }
                    }
                }
            }
            Node::Throw(throw) => {
                if throw.expr().contains_arguments() {
                    return true;
                }
            }
            Node::Try(r#try) => {
                for node in r#try.block().items() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
                if let Some(catch) = r#try.catch() {
                    for node in catch.block().items() {
                        if node.contains_arguments() {
                            return true;
                        }
                    }
                }
                if let Some(finally) = r#try.finally() {
                    for node in finally.items() {
                        if node.contains_arguments() {
                            return true;
                        }
                    }
                }
            }
            Node::UnaryOp(unary_op) => {
                if unary_op.target().contains_arguments() {
                    return true;
                }
            }
            Node::WhileLoop(while_loop) => {
                if while_loop.cond().contains_arguments() {
                    return true;
                }
                if while_loop.body().contains_arguments() {
                    return true;
                }
            }
            Node::Yield(r#yield) => {
                if let Some(node) = r#yield.expr() {
                    if node.contains_arguments() {
                        return true;
                    }
                }
            }
            Node::ClassExpr(class) | Node::ClassDecl(class) => {
                if let Some(node) = class.super_ref() {
                    if node.contains_arguments() {
                        return true;
                    }
                    for element in class.elements() {
                        match element {
                            ClassElement::MethodDefinition(_, method)
                            | ClassElement::StaticMethodDefinition(_, method) => match method {
                                MethodDefinition::Get(function)
                                | MethodDefinition::Set(function)
                                | MethodDefinition::Ordinary(function) => {
                                    if let Some(Sym::ARGUMENTS) = function.name() {
                                        return true;
                                    }
                                }
                                MethodDefinition::Generator(generator) => {
                                    if let Some(Sym::ARGUMENTS) = generator.name() {
                                        return true;
                                    }
                                }
                                MethodDefinition::AsyncGenerator(async_generator) => {
                                    if let Some(Sym::ARGUMENTS) = async_generator.name() {
                                        return true;
                                    }
                                }
                                MethodDefinition::Async(function) => {
                                    if let Some(Sym::ARGUMENTS) = function.name() {
                                        return true;
                                    }
                                }
                            },
                            ClassElement::FieldDefinition(_, node)
                            | ClassElement::StaticFieldDefinition(_, node)
                            | ClassElement::PrivateFieldDefinition(_, node)
                            | ClassElement::PrivateStaticFieldDefinition(_, node) => {
                                if let Some(node) = node {
                                    if node.contains_arguments() {
                                        return true;
                                    }
                                }
                            }
                            ClassElement::StaticBlock(statement_list) => {
                                for node in statement_list.items() {
                                    if node.contains_arguments() {
                                        return true;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }
}

impl ToInternedString for Node {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

/// Utility to join multiple Nodes into a single string.
fn join_nodes<N>(interner: &Interner, nodes: &[N]) -> String
where
    N: ToInternedString,
{
    let mut first = true;
    let mut buf = String::new();
    for e in nodes {
        if first {
            first = false;
        } else {
            buf.push_str(", ");
        }
        buf.push_str(&e.to_interned_string(interner));
    }
    buf
}

/// This parses the given source code, and then makes sure that
/// the resulting `StatementList` is formatted in the same manner
/// as the source code. This is expected to have a preceding
/// newline.
///
/// This is a utility function for tests. It was made in case people
/// are using different indents in their source files. This fixes
/// any strings which may have been changed in a different indent
/// level.
#[cfg(test)]
fn test_formatting(source: &'static str) {
    use crate::{syntax::Parser, Context};

    // Remove preceding newline.
    let source = &source[1..];

    // Find out how much the code is indented
    let first_line = &source[..source.find('\n').unwrap()];
    let trimmed_first_line = first_line.trim();
    let characters_to_remove = first_line.len() - trimmed_first_line.len();

    let scenario = source
        .lines()
        .map(|l| &l[characters_to_remove..]) // Remove preceding whitespace from each line
        .collect::<Vec<&'static str>>()
        .join("\n");
    let mut context = Context::default();
    let result = Parser::new(scenario.as_bytes())
        .parse_all(&mut context)
        .expect("parsing failed")
        .to_interned_string(context.interner());
    if scenario != result {
        eprint!("========= Expected:\n{scenario}");
        eprint!("========= Got:\n{result}");
        // Might be helpful to find differing whitespace
        eprintln!("========= Expected: {scenario:?}");
        eprintln!("========= Got:      {result:?}");
        panic!("parsing test did not give the correct result (see above)");
    }
}
