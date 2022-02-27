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
        generator_decl::GeneratorDecl, generator_expr::GeneratorExpr, ArrowFunctionDecl,
        AsyncFunctionDecl, AsyncFunctionExpr, Declaration, DeclarationList, DeclarationPattern,
        FunctionDecl, FunctionExpr,
    },
    field::{GetConstField, GetField},
    identifier::Identifier,
    iteration::{Break, Continue, DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, WhileLoop},
    new::New,
    object::Object,
    operator::{Assign, BinOp, UnaryOp},
    parameters::{FormalParameter, FormalParameterList},
    r#yield::Yield,
    return_smt::Return,
    spread::Spread,
    statement_list::{RcStatementList, StatementList},
    switch::{Case, Switch},
    template::{TaggedTemplate, TemplateLit},
    throw::Throw,
    try_node::{Catch, Finally, Try},
};

pub(crate) use self::parameters::FormalParameterListFlags;

use super::Const;
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, ToInternedString};
use std::cmp::Ordering;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

// TODO: This should be split into Expression and Statement.
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
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
        }
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
    use crate::syntax::Parser;

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
    let mut interner = Interner::default();
    let result = Parser::new(scenario.as_bytes(), false)
        .parse_all(&mut interner)
        .expect("parsing failed")
        .to_interned_string(&interner);
    if scenario != result {
        eprint!("========= Expected:\n{scenario}");
        eprint!("========= Got:\n{result}");
        // Might be helpful to find differing whitespace
        eprintln!("========= Expected: {scenario:?}");
        eprintln!("========= Got:      {result:?}");
        panic!("parsing test did not give the correct result (see above)");
    }
}
