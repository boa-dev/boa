//! This module implements the `Node` structure, which composes the AST.

pub mod array;
pub mod await_expr;
pub mod block;
pub mod break_node;
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
    break_node::Break,
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
    iteration::{Continue, DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, WhileLoop},
    new::New,
    object::Object,
    operator::{Assign, BinOp, UnaryOp},
    r#yield::Yield,
    return_smt::Return,
    spread::Spread,
    statement_list::{RcStatementList, StatementList},
    switch::{Case, Switch},
    template::{TaggedTemplate, TemplateLit},
    throw::Throw,
    try_node::{Catch, Finally, Try},
};
use super::Const;
use crate::gc::{empty_trace, Finalize, Trace};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
};

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
    TaggedTemplate(TaggedTemplate),

    /// A template literal. [More information](./template/struct.TemplateLit.html).
    TemplateLit(TemplateLit),

    /// A throw statement. [More information](./throw/struct.Throw.html).
    Throw(Throw),

    /// A `try...catch` node. [More information](./try_node/struct.Try.htl).
    Try(Try),

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

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<Const> for Node {
    fn from(c: Const) -> Self {
        Self::Const(c)
    }
}

impl Node {
    /// Returns a node ordering based on the hoistability of each node.
    pub(crate) fn hoistable_order(a: &Node, b: &Node) -> Ordering {
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

    /// Displays the value of the node with the given indentation. For example, an indent
    /// level of 2 would produce this:
    ///
    /// ```js
    ///         function hello() {
    ///             console.log("hello");
    ///         }
    ///         hello();
    ///         a = 2;
    /// ```
    fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        let indent = "    ".repeat(indentation);
        match *self {
            Self::Block(_) => {}
            _ => write!(f, "{}", indent)?,
        }
        self.display_no_indent(f, indentation)
    }

    /// Implements the display formatting with indentation. This will not prefix the value with
    /// any indentation. If you want to prefix this with proper indents, use [`display`](Self::display).
    fn display_no_indent(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        match *self {
            Self::Call(ref expr) => Display::fmt(expr, f),
            Self::Const(ref c) => write!(f, "{}", c),
            Self::ConditionalOp(ref cond_op) => Display::fmt(cond_op, f),
            Self::ForLoop(ref for_loop) => for_loop.display(f, indentation),
            Self::ForOfLoop(ref for_of) => for_of.display(f, indentation),
            Self::ForInLoop(ref for_in) => for_in.display(f, indentation),
            Self::This => write!(f, "this"),
            Self::Try(ref try_catch) => try_catch.display(f, indentation),
            Self::Break(ref break_smt) => Display::fmt(break_smt, f),
            Self::Continue(ref cont) => Display::fmt(cont, f),
            Self::Spread(ref spread) => Display::fmt(spread, f),
            Self::Block(ref block) => block.display(f, indentation),
            Self::Identifier(ref s) => Display::fmt(s, f),
            Self::New(ref expr) => Display::fmt(expr, f),
            Self::GetConstField(ref get_const_field) => Display::fmt(get_const_field, f),
            Self::GetField(ref get_field) => Display::fmt(get_field, f),
            Self::WhileLoop(ref while_loop) => while_loop.display(f, indentation),
            Self::DoWhileLoop(ref do_while) => do_while.display(f, indentation),
            Self::If(ref if_smt) => if_smt.display(f, indentation),
            Self::Switch(ref switch) => switch.display(f, indentation),
            Self::Object(ref obj) => obj.display(f, indentation),
            Self::ArrayDecl(ref arr) => Display::fmt(arr, f),
            Self::VarDeclList(ref list) => Display::fmt(list, f),
            Self::FunctionDecl(ref decl) => decl.display(f, indentation),
            Self::FunctionExpr(ref expr) => expr.display(f, indentation),
            Self::ArrowFunctionDecl(ref decl) => decl.display(f, indentation),
            Self::BinOp(ref op) => Display::fmt(op, f),
            Self::UnaryOp(ref op) => Display::fmt(op, f),
            Self::Return(ref ret) => Display::fmt(ret, f),
            Self::TaggedTemplate(ref template) => Display::fmt(template, f),
            Self::TemplateLit(ref template) => Display::fmt(template, f),
            Self::Throw(ref throw) => Display::fmt(throw, f),
            Self::Assign(ref op) => Display::fmt(op, f),
            Self::LetDeclList(ref decl) => Display::fmt(decl, f),
            Self::ConstDeclList(ref decl) => Display::fmt(decl, f),
            Self::AsyncFunctionDecl(ref decl) => decl.display(f, indentation),
            Self::AsyncFunctionExpr(ref expr) => expr.display(f, indentation),
            Self::AwaitExpr(ref expr) => Display::fmt(expr, f),
            Self::Empty => write!(f, ";"),
            Self::Yield(ref y) => Display::fmt(y, f),
            Self::GeneratorDecl(ref decl) => Display::fmt(decl, f),
            Self::GeneratorExpr(ref expr) => expr.display(f, indentation),
            Self::AsyncGeneratorExpr(ref expr) => expr.display(f, indentation),
            Self::AsyncGeneratorDecl(ref decl) => decl.display(f, indentation),
        }
    }
}

/// Utility to join multiple Nodes into a single string.
fn join_nodes<N>(f: &mut fmt::Formatter<'_>, nodes: &[N]) -> fmt::Result
where
    N: Display,
{
    let mut first = true;
    for e in nodes {
        if !first {
            f.write_str(", ")?;
        }
        first = false;
        Display::fmt(e, f)?;
    }
    Ok(())
}

/// "Formal parameter" is a fancy way of saying "function parameter".
///
/// In the declaration of a function, the parameters must be identifiers,
/// not any value like numbers, strings, or objects.
///```text
///function foo(formalParameter1, formalParameter2) {
///}
///```
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameter
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Errors/Missing_formal_parameter
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub struct FormalParameter {
    declaration: Declaration,
    is_rest_param: bool,
}

impl FormalParameter {
    /// Creates a new formal parameter.
    pub(in crate::syntax) fn new<D>(declaration: D, is_rest_param: bool) -> Self
    where
        D: Into<Declaration>,
    {
        Self {
            declaration: declaration.into(),
            is_rest_param,
        }
    }

    /// Gets the name of the formal parameter.
    pub fn names(&self) -> Vec<&str> {
        match &self.declaration {
            Declaration::Identifier { ident, .. } => vec![ident.as_ref()],
            Declaration::Pattern(pattern) => match pattern {
                DeclarationPattern::Object(object_pattern) => object_pattern.idents(),

                DeclarationPattern::Array(array_pattern) => array_pattern.idents(),
            },
        }
    }

    /// Get the declaration of the formal parameter
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }

    /// Gets the initialization node of the formal parameter, if any.
    pub fn init(&self) -> Option<&Node> {
        self.declaration.init()
    }

    /// Gets wether the parameter is a rest parameter.
    pub fn is_rest_param(&self) -> bool {
        self.is_rest_param
    }

    pub fn is_identifier(&self) -> bool {
        matches!(&self.declaration, Declaration::Identifier { .. })
    }
}

impl Display for FormalParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_rest_param {
            write!(f, "...")?;
        }
        write!(f, "{}", self.declaration)?;
        Ok(())
    }
}

/// A JavaScript property is a characteristic of an object, often describing attributes associated with a data structure.
///
/// A property has a name (a string) and a value (primitive, method, or object reference).
/// Note that when we say that "a property holds an object", that is shorthand for "a property holds an object reference".
/// This distinction matters because the original referenced object remains unchanged when you change the property's value.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/property/JavaScript
// TODO: Support all features: https://tc39.es/ecma262/#prod-PropertyDefinition
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub enum PropertyDefinition {
    /// Puts a variable into an object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-IdentifierReference
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Property_definitions
    IdentifierReference(Box<str>),

    /// Binds a property name to a JavaScript value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Property_definitions
    Property(PropertyName, Node),

    /// A property of an object can also refer to a function or a getter or setter method.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Method_definitions
    MethodDefinition(MethodDefinitionKind, PropertyName, FunctionExpr),

    /// The Rest/Spread Properties for ECMAScript proposal (stage 4) adds spread properties to object literals.
    /// It copies own enumerable properties from a provided object onto a new object.
    ///
    /// Shallow-cloning (excluding `prototype`) or merging objects is now possible using a shorter syntax than `Object.assign()`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Spread_properties
    SpreadObject(Node),
}

impl PropertyDefinition {
    /// Creates an `IdentifierReference` property definition.
    pub fn identifier_reference<I>(ident: I) -> Self
    where
        I: Into<Box<str>>,
    {
        Self::IdentifierReference(ident.into())
    }

    /// Creates a `Property` definition.
    pub fn property<N, V>(name: N, value: V) -> Self
    where
        N: Into<PropertyName>,
        V: Into<Node>,
    {
        Self::Property(name.into(), value.into())
    }

    /// Creates a `MethodDefinition`.
    pub fn method_definition<N>(kind: MethodDefinitionKind, name: N, body: FunctionExpr) -> Self
    where
        N: Into<PropertyName>,
    {
        Self::MethodDefinition(kind, name.into(), body)
    }

    /// Creates a `SpreadObject`.
    pub fn spread_object<O>(obj: O) -> Self
    where
        O: Into<Node>,
    {
        Self::SpreadObject(obj.into())
    }
}

/// Method definition kinds.
///
/// Starting with ECMAScript 2015, a shorter syntax for method definitions on objects initializers is introduced.
/// It is a shorthand for a function assigned to the method's name.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Method_definitions
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Copy, Finalize)]
pub enum MethodDefinitionKind {
    /// The `get` syntax binds an object property to a function that will be called when that property is looked up.
    ///
    /// Sometimes it is desirable to allow access to a property that returns a dynamically computed value,
    /// or you may want to reflect the status of an internal variable without requiring the use of explicit method calls.
    /// In JavaScript, this can be accomplished with the use of a getter.
    ///
    /// It is not possible to simultaneously have a getter bound to a property and have that property actually hold a value,
    /// although it is possible to use a getter and a setter in conjunction to create a type of pseudo-property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/get
    Get,

    /// The `set` syntax binds an object property to a function to be called when there is an attempt to set that property.
    ///
    /// In JavaScript, a setter can be used to execute a function whenever a specified property is attempted to be changed.
    /// Setters are most often used in conjunction with getters to create a type of pseudo-property.
    /// It is not possible to simultaneously have a setter on a property that holds an actual value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/set
    Set,

    /// Starting with ECMAScript 2015, you are able to define own methods in a shorter syntax, similar to the getters and setters.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions#Method_definition_syntax
    Ordinary,

    /// Starting with ECMAScript 2015, you are able to define own methods in a shorter syntax, similar to the getters and setters.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Method_definitions#generator_methods
    Generator,

    /// Async generators can be used to define a method
    ///
    /// More information
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorMethod
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Method_definitions#async_generator_methods
    AsyncGenerator,

    /// Async function can be used to define a method
    ///
    /// More information
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AsyncMethod
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Method_definitions#async_methods
    Async,
}

unsafe impl Trace for MethodDefinitionKind {
    empty_trace!();
}

/// PropertyName can be either a literal or computed.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-PropertyName
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Finalize)]
pub enum PropertyName {
    /// A `Literal` property name can be either an identifier, a string or a numeric literal.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-LiteralPropertyName
    Literal(Box<str>),
    /// A `Computed` property name is an expression that gets evaluated and converted into a property name.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ComputedPropertyName
    Computed(Node),
}

impl Display for PropertyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyName::Literal(key) => write!(f, "{}", key),
            PropertyName::Computed(key) => write!(f, "{}", key),
        }
    }
}

impl<T> From<T> for PropertyName
where
    T: Into<Box<str>>,
{
    fn from(name: T) -> Self {
        Self::Literal(name.into())
    }
}

impl From<Node> for PropertyName {
    fn from(name: Node) -> Self {
        Self::Computed(name)
    }
}

unsafe impl Trace for PropertyName {
    empty_trace!();
}

/// This parses the given source code, and then makes sure that
/// the resulting StatementList is formatted in the same manner
/// as the source code. This is expected to have a preceding
/// newline.
///
/// This is a utility function for tests. It was made in case people
/// are using different indents in their source files. This fixes
/// any strings which may have been changed in a different indent
/// level.
#[cfg(test)]
fn test_formatting(source: &'static str) {
    use crate::{syntax::Parser, Interner};

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
    let mut interner = Interner::new();
    let result = format!(
        "{}",
        Parser::new(scenario.as_bytes(), false)
            .parse_all(&mut interner)
            .expect("parsing failed")
    );
    if scenario != result {
        eprint!("========= Expected:\n{}", scenario);
        eprint!("========= Got:\n{}", result);
        // Might be helpful to find differing whitespace
        eprintln!("========= Expected: {:?}", scenario);
        eprintln!("========= Got:      {:?}", result);
        panic!("parsing test did not give the correct result (see above)");
    }
}
