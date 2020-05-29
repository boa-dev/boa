//! This module implements the `Node` structure, which composes the AST.

pub mod array;
pub mod block;
pub mod declaration;
pub mod expression;
pub mod field;
pub mod identifier;
pub mod iteration;
pub mod object;
pub mod operator;
pub mod statement_list;
pub mod switch;
pub mod try_node;

pub use self::{
    array::ArrayDecl,
    block::Block,
    declaration::{
        ArrowFunctionDecl, ConstDecl, ConstDeclList, FunctionDecl, FunctionExpr, LetDecl,
        LetDeclList, VarDecl, VarDeclList,
    },
    expression::{Call, New},
    field::{GetConstField, GetField},
    identifier::Identifier,
    iteration::{ForLoop, WhileLoop},
    object::Object,
    operator::{Assign, BinOp, UnaryOp},
    statement_list::StatementList,
    switch::Switch,
    try_node::{Catch, Finally, Try},
};
use super::Const;
use gc::{Finalize, Trace};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum Node {
    /// Array declaration node. [More information](./array/struct.ArrayDecl.html).
    ArrayDecl(ArrayDecl),

    /// An arrow function expression node. [More information](./arrow_function/struct.ArrowFunctionDecl.html).
    ArrowFunctionDecl(ArrowFunctionDecl),

    /// An assignment operator node. [More information](./operator/struct.Assign.html).
    Assign(Assign),

    /// A binary operator node. [More information](./operator/struct.BinOp.html).
    BinOp(BinOp),

    /// A Block node. [More information](./block/struct.Block.html).
    Block(Block),

    /// The `break` statement terminates the current loop, switch, or label statement and transfers
    /// program control to the statement following the terminated statement.
    ///
    /// The break statement includes an optional label that allows the program to break out of a
    /// labeled statement. The break statement needs to be nested within the referenced label. The
    /// labeled statement can be any block statement; it does not have to be preceded by a loop
    /// statement.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-BreakStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
    Break(Option<Box<str>>),

    /// A function call. [More information](./expression/struct.Call.html).
    Call(Call),

    /// The `conditional` (ternary) operator is the only JavaScript operator that takes three
    /// operands.
    ///
    /// This operator is the only JavaScript operator that takes three operands: a condition
    /// followed by a question mark (`?`), then an expression to execute `if` the condition is
    /// truthy followed by a colon (`:`), and finally the expression to execute if the condition
    /// is `false`. This operator is frequently used as a shortcut for the `if` statement.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ConditionalExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Literals
    ConditionalOp(Box<Node>, Box<Node>, Box<Node>),

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

    /// A constant declaration list. [More information](./declaration/struct.ConstDeclList.html).
    ConstDeclList(ConstDeclList),

    /// The `continue` statement terminates execution of the statements in the current iteration of
    /// the current or labeled loop, and continues execution of the loop with the next iteration.
    ///
    /// The continue statement can include an optional label that allows the program to jump to the
    /// next iteration of a labeled loop statement instead of the current loop. In this case, the
    /// continue statement needs to be nested within this labeled statement.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ContinueStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
    Continue(Option<Box<str>>),

    /// The `do...while` statement creates a loop that executes a specified statement until the
    /// test condition evaluates to false.
    ///
    /// The condition is evaluated after executing the statement, resulting in the specified
    /// statement executing at least once.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-do-while-statement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
    DoWhileLoop(Box<Node>, Box<Node>),

    /// A function declaration node. [More information](./declaration/struct.FunctionDecl.html).
    FunctionDecl(FunctionDecl),

    /// A function expressino node. [More information](./declaration/struct.FunctionExpr.html).
    FunctionExpr(FunctionExpr),

    /// Provides access to an object types' constant properties. [More information](./declaration/struct.GetConstField.html).
    GetConstField(GetConstField),

    /// Provides access to object fields. [More information](./declaration/struct.GetField.html).
    GetField(GetField),

    /// A `for` statement. [More information](./iteration.struct.ForLoop.html).
    ForLoop(ForLoop),

    /// The `if` statement executes a statement if a specified condition is [`truthy`][truthy]. If
    /// the condition is [`falsy`][falsy], another statement can be executed.
    ///
    /// Multiple `if...else` statements can be nested to create an else if clause.
    ///
    /// Note that there is no elseif (in one word) keyword in JavaScript.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-IfStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
    /// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/truthy
    /// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/falsy
    /// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
    If(Box<Node>, Box<Node>, Option<Box<Node>>),

    /// A `let` declaration list. [More information](./declaration/struct.LetDeclList.html).
    LetDeclList(LetDeclList),

    /// A local identifier node. [More information](./identifier/struct.Identifier.html).
    Identifier(Identifier),

    /// A `new` expression. [More information](./expression/struct.New.html).
    New(New),

    /// An object. [More information](./object/struct.Object.html).
    Object(Object),

    /// The `return` statement ends function execution and specifies a value to be returned to the
    /// function caller.
    ///
    /// Syntax: `return [expression];`
    ///
    /// `expression`:
    ///  > The expression whose value is to be returned. If omitted, `undefined` is returned
    ///  > nstead.
    ///
    /// When a `return` statement is used in a function body, the execution of the function is
    /// stopped. If specified, a given value is returned to the function caller.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ReturnStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
    Return(Option<Box<Node>>),

    Switch(Switch),

    /// The `spread` operator allows an iterable such as an array expression or string to be
    /// expanded.
    ///
    /// Syntax: `...x`
    ///
    /// It expands array expressions or strings in places where zero or more arguments (for
    /// function calls) or elements (for array literals)
    /// are expected, or an object expression to be expanded in places where zero or more key-value
    /// pairs (for object literals) are expected.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-SpreadElement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax
    Spread(Box<Node>),

    /// The `throw` statement throws a user-defined exception.
    ///
    /// Syntax: `throw expression;`
    ///
    /// Execution of the current function will stop (the statements after throw won't be executed),
    /// and control will be passed to the first catch block in the call stack. If no catch block
    /// exists among caller functions, the program will terminate.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ThrowStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw
    Throw(Box<Node>),

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

    /// Array declaration node. [More information](./declaration/struct.VarDeclList.html).
    VarDeclList(VarDeclList),

    WhileLoop(WhileLoop),
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

    /// Creates a `Break` AST node.
    pub fn break_node<OL, L>(label: OL) -> Self
    where
        L: Into<Box<str>>,
        OL: Into<Option<L>>,
    {
        Self::Break(label.into().map(L::into))
    }

    /// Creates a `ConditionalOp` AST node.
    pub fn conditional_op<C, T, F>(condition: C, if_true: T, if_false: F) -> Self
    where
        C: Into<Self>,
        T: Into<Self>,
        F: Into<Self>,
    {
        Self::ConditionalOp(
            Box::new(condition.into()),
            Box::new(if_true.into()),
            Box::new(if_false.into()),
        )
    }

    /// Creates a `Continue` AST node.
    pub fn continue_node<OL, L>(label: OL) -> Self
    where
        L: Into<Box<str>>,
        OL: Into<Option<L>>,
    {
        Self::Continue(label.into().map(L::into))
    }

    /// Creates a `DoWhileLoop` AST node.
    pub fn do_while_loop<B, C>(body: B, condition: C) -> Self
    where
        B: Into<Self>,
        C: Into<Self>,
    {
        Self::DoWhileLoop(Box::new(body.into()), Box::new(condition.into()))
    }

    /// Creates an `If` AST node.
    pub fn if_node<C, B, E, OE>(condition: C, body: B, else_node: OE) -> Self
    where
        C: Into<Self>,
        B: Into<Self>,
        E: Into<Self>,
        OE: Into<Option<E>>,
    {
        Self::If(
            Box::new(condition.into()),
            Box::new(body.into()),
            else_node.into().map(E::into).map(Box::new),
        )
    }

    /// Creates a `Return` AST node.
    pub fn return_node<E, OE>(expr: OE) -> Self
    where
        E: Into<Self>,
        OE: Into<Option<E>>,
    {
        Self::Return(expr.into().map(E::into).map(Box::new))
    }

    /// Creates a `Spread` AST node.
    pub fn spread<V>(val: V) -> Self
    where
        V: Into<Self>,
    {
        Self::Spread(Box::new(val.into()))
    }

    /// Creates a `Throw` AST node.
    pub fn throw<V>(val: V) -> Self
    where
        V: Into<Self>,
    {
        Self::Throw(Box::new(val.into()))
    }

    /// Creates a `This` AST node.
    pub fn this() -> Self {
        Self::This
    }

    /// Implements the display formatting with indentation.
    fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        let indent = "    ".repeat(indentation);
        match *self {
            Self::Block(_) => {}
            _ => write!(f, "{}", indent)?,
        }

        match *self {
            Self::Const(ref c) => write!(f, "{}", c),
            Self::ConditionalOp(ref cond, ref if_true, ref if_false) => {
                write!(f, "{} ? {} : {}", cond, if_true, if_false)
            }
            Self::ForLoop(ref for_loop) => for_loop.display(f, indentation),
            Self::This => write!(f, "this"),
            Self::Try(ref try_catch) => try_catch.display(f, indentation),
            Self::Break(ref l) => write!(
                f,
                "break{}",
                if let Some(label) = l {
                    format!(" {}", label)
                } else {
                    String::new()
                }
            ),
            Self::Continue(ref l) => write!(
                f,
                "continue{}",
                if let Some(label) = l {
                    format!(" {}", label)
                } else {
                    String::new()
                }
            ),
            Self::Spread(ref node) => write!(f, "...{}", node),
            Self::Block(ref block) => block.display(f, indentation),
            Self::Identifier(ref s) => Display::fmt(s, f),
            Self::GetConstField(ref get_const_field) => Display::fmt(get_const_field, f),
            Self::GetField(ref get_field) => Display::fmt(get_field, f),
            Self::Call(ref expr) => Display::fmt(expr, f),
            Self::New(ref expr) => Display::fmt(expr, f),
            Self::WhileLoop(ref while_loop) => while_loop.display(f, indentation),
            Self::DoWhileLoop(ref node, ref cond) => {
                write!(f, "do")?;
                node.display(f, indentation)?;
                write!(f, "while ({})", cond)
            }
            Self::If(ref cond, ref node, None) => {
                write!(f, "if ({}) ", cond)?;
                node.display(f, indentation)
            }
            Self::If(ref cond, ref node, Some(ref else_e)) => {
                write!(f, "if ({}) ", cond)?;
                node.display(f, indentation)?;
                f.write_str(" else ")?;
                else_e.display(f, indentation)
            }

            Self::Switch(ref switch) => switch.display(f, indentation),
            Self::Object(ref obj) => obj.display(f, indentation),
            Self::ArrayDecl(ref arr) => Display::fmt(arr, f),
            Self::VarDeclList(ref list) => Display::fmt(list, f),
            Self::FunctionDecl(ref decl) => decl.display(f, indentation),
            Self::FunctionExpr(ref expr) => expr.display(f, indentation),
            Self::ArrowFunctionDecl(ref decl) => decl.display(f, indentation),
            Self::BinOp(ref op) => Display::fmt(op, f),
            Self::UnaryOp(ref op) => Display::fmt(op, f),
            Self::Return(Some(ref ex)) => write!(f, "return {}", ex),
            Self::Return(None) => write!(f, "return"),
            Self::Throw(ref ex) => write!(f, "throw {}", ex),
            Self::Assign(ref op) => Display::fmt(op, f),
            Self::LetDeclList(ref decl) => Display::fmt(decl, f),
            Self::ConstDeclList(ref decl) => Display::fmt(decl, f),
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub struct FormalParameter {
    name: Box<str>,
    init: Option<Node>,
    is_rest_param: bool,
}

impl FormalParameter {
    /// Creates a new formal parameter.
    pub(in crate::syntax) fn new<N>(name: N, init: Option<Node>, is_rest_param: bool) -> Self
    where
        N: Into<Box<str>>,
    {
        Self {
            name: name.into(),
            init,
            is_rest_param,
        }
    }

    /// Gets the name of the formal parameter.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the initialization node of the formal parameter, if any.
    pub fn init(&self) -> Option<&Node> {
        self.init.as_ref()
    }

    /// Gets wether the parameter is a rest parameter.
    pub fn is_rest_param(&self) -> bool {
        self.is_rest_param
    }
}

impl Display for FormalParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_rest_param {
            write!(f, "...")?;
        }
        write!(f, "{}", self.name)?;
        if let Some(n) = self.init.as_ref() {
            write!(f, " = {}", n)?;
        }
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    Property(Box<str>, Node),

    /// A property of an object can also refer to a function or a getter or setter method.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Method_definitions
    MethodDefinition(MethodDefinitionKind, Box<str>, FunctionExpr),

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
        N: Into<Box<str>>,
        V: Into<Node>,
    {
        Self::Property(name.into(), value.into())
    }

    /// Creates a `MethodDefinition`.
    pub fn method_definition<N>(kind: MethodDefinitionKind, name: N, body: FunctionExpr) -> Self
    where
        N: Into<Box<str>>,
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
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
    // TODO: support other method definition kinds, like `Generator`.
}
