//! Declaration nodes.

use super::{join_nodes, FormalParameter, Identifier, Node, StatementList};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `var` statement declares a variable, optionally initializing it to a value.
///
/// var declarations, wherever they occur, are processed before any code is executed. This is
/// called hoisting, and is discussed further below.
///
/// The scope of a variable declared with var is its current execution context, which is either
/// the enclosing function or, for variables declared outside any function, global. If you
/// re-declare a JavaScript variable, it will not lose its value.
///
/// Assigning a value to an undeclared variable implicitly creates it as a global variable (it
/// becomes a property of the global object) when the assignment is executed.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-VariableStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct VarDeclList {
    #[cfg_attr(feature = "serde", serde(flatten))]
    vars: Box<[VarDecl]>,
}

impl<T> From<T> for VarDeclList
where
    T: Into<Box<[VarDecl]>>,
{
    fn from(list: T) -> Self {
        Self { vars: list.into() }
    }
}

impl From<VarDecl> for VarDeclList {
    fn from(decl: VarDecl) -> Self {
        Self {
            vars: Box::new([decl]),
        }
    }
}

impl AsRef<[VarDecl]> for VarDeclList {
    fn as_ref(&self) -> &[VarDecl] {
        &self.vars
    }
}

impl fmt::Display for VarDeclList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.vars.is_empty() {
            write!(f, "var ")?;
            join_nodes(f, &self.vars)
        } else {
            Ok(())
        }
    }
}

impl From<VarDeclList> for Node {
    fn from(list: VarDeclList) -> Self {
        Self::VarDeclList(list)
    }
}

/// Individual variable declaration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct VarDecl {
    name: Identifier,
    init: Option<Node>,
}

impl fmt::Display for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)?;
        if let Some(ref init) = self.init {
            write!(f, " = {}", init)?;
        }
        Ok(())
    }
}

impl VarDecl {
    /// Creates a new variable declaration.
    pub(in crate::syntax) fn new<N, I>(name: N, init: I) -> Self
    where
        N: Into<Identifier>,
        I: Into<Option<Node>>,
    {
        Self {
            name: name.into(),
            init: init.into(),
        }
    }

    /// Gets the name of the variable.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Gets the initialization node for the variable, if any.
    pub fn init(&self) -> Option<&Node> {
        self.init.as_ref()
    }
}

/// The `function` expression defines a function with the specified parameters.
///
/// A function created with a function expression is a `Function` object and has all the
/// properties, methods and behavior of `Function`.
///
/// A function can also be created using a declaration (see function expression).
///
/// By default, functions return `undefined`. To return any other value, the function must have
/// a return statement that specifies the value to return.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-function
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct FunctionExpr {
    name: Option<Box<str>>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl FunctionExpr {
    /// Creates a new function expression
    pub(in crate::syntax) fn new<N, P, B>(name: N, parameters: P, body: B) -> Self
    where
        N: Into<Option<Box<str>>>,
        P: Into<Box<[FormalParameter]>>,
        B: Into<StatementList>,
    {
        Self {
            name: name.into(),
            parameters: parameters.into(),
            body: body.into(),
        }
    }

    /// Gets the name of the function declaration.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(Box::as_ref)
    }

    /// Gets the list of parameters of the function declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    pub fn body(&self) -> &[Node] {
        self.body.statements()
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        f.write_str("function")?;
        if let Some(ref name) = self.name {
            write!(f, " {}", name)?;
        }
        f.write_str("(")?;
        join_nodes(f, &self.parameters)?;
        f.write_str(") {{")?;

        self.body.display(f, indentation + 1)?;

        writeln!(f, "}}")
    }
}

impl fmt::Display for FunctionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<FunctionExpr> for Node {
    fn from(expr: FunctionExpr) -> Self {
        Self::FunctionExpr(expr)
    }
}

/// The `function` declaration (function statement) defines a function with the specified
/// parameters.
///
/// A function created with a function declaration is a `Function` object and has all the
/// properties, methods and behavior of `Function`.
///
/// A function can also be created using an expression (see [function expression][func_expr]).
///
/// By default, functions return `undefined`. To return any other value, the function must have
/// a return statement that specifies the value to return.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-function
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
/// [func_expr]: ../enum.Node.html#variant.FunctionExpr
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct FunctionDecl {
    name: Box<str>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl FunctionDecl {
    /// Creates a new function declaration.
    pub(in crate::syntax) fn new<N, P, B>(name: N, parameters: P, body: B) -> Self
    where
        N: Into<Box<str>>,
        P: Into<Box<[FormalParameter]>>,
        B: Into<StatementList>,
    {
        Self {
            name: name.into(),
            parameters: parameters.into(),
            body: body.into(),
        }
    }

    /// Gets the name of the function declaration.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the list of parameters of the function declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    pub fn body(&self) -> &[Node] {
        self.body.statements()
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        write!(f, "function {}(", self.name)?;
        join_nodes(f, &self.parameters)?;
        f.write_str(") {{")?;

        self.body.display(f, indentation + 1)?;

        writeln!(f, "}}")
    }
}

impl From<FunctionDecl> for Node {
    fn from(decl: FunctionDecl) -> Self {
        Self::FunctionDecl(decl)
    }
}

impl fmt::Display for FunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

/// An arrow function expression is a syntactically compact alternative to a regular function
/// expression.
///
/// Arrow function expressions are ill suited as methods, and they cannot be used as
/// constructors. Arrow functions cannot be used as constructors and will throw an error when
/// used with new.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ArrowFunction
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ArrowFunctionDecl {
    params: Box<[FormalParameter]>,
    body: StatementList,
}

impl ArrowFunctionDecl {
    /// Creates a new `ArrowFunctionDecl` AST node.
    pub(in crate::syntax) fn new<P, B>(params: P, body: B) -> Self
    where
        P: Into<Box<[FormalParameter]>>,
        B: Into<StatementList>,
    {
        Self {
            params: params.into(),
            body: body.into(),
        }
    }

    /// Gets the list of parameters of the arrow function.
    pub(crate) fn params(&self) -> &[FormalParameter] {
        &self.params
    }

    /// Gets the body of the arrow function.
    pub(crate) fn body(&self) -> &[Node] {
        &self.body.statements()
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        write!(f, "(")?;
        join_nodes(f, &self.params)?;
        f.write_str(") => ")?;
        self.body.display(f, indentation)
    }
}

impl fmt::Display for ArrowFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<ArrowFunctionDecl> for Node {
    fn from(decl: ArrowFunctionDecl) -> Self {
        Self::ArrowFunctionDecl(decl)
    }
}

/// The `const` statements are block-scoped, much like variables defined using the `let`
/// keyword.
///
/// This declaration creates a constant whose scope can be either global or local to the block
/// in which it is declared. Global constants do not become properties of the window object,
/// unlike var variables.
///
/// An initializer for a constant is required. You must specify its value in the same statement
/// in which it's declared. (This makes sense, given that it can't be changed later.)
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const
/// [identifier]: https://developer.mozilla.org/en-US/docs/Glossary/identifier
/// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ConstDeclList {
    #[cfg_attr(feature = "serde", serde(flatten))]
    list: Box<[ConstDecl]>,
}

impl<T> From<T> for ConstDeclList
where
    T: Into<Box<[ConstDecl]>>,
{
    fn from(list: T) -> Self {
        Self { list: list.into() }
    }
}

impl From<ConstDecl> for ConstDeclList {
    fn from(decl: ConstDecl) -> Self {
        Self {
            list: Box::new([decl]),
        }
    }
}

impl AsRef<[ConstDecl]> for ConstDeclList {
    fn as_ref(&self) -> &[ConstDecl] {
        &self.list
    }
}

impl fmt::Display for ConstDeclList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.list.is_empty() {
            write!(f, "const ")?;
            join_nodes(f, &self.list)
        } else {
            Ok(())
        }
    }
}

impl From<ConstDeclList> for Node {
    fn from(list: ConstDeclList) -> Self {
        Self::ConstDeclList(list)
    }
}

/// Individual constant declaration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ConstDecl {
    name: Identifier,
    init: Node,
}

impl fmt::Display for ConstDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.init)
    }
}

impl ConstDecl {
    /// Creates a new variable declaration.
    pub(in crate::syntax) fn new<N, I>(name: N, init: I) -> Self
    where
        N: Into<Identifier>,
        I: Into<Node>,
    {
        Self {
            name: name.into(),
            init: init.into(),
        }
    }

    /// Gets the name of the variable.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Gets the initialization node for the variable, if any.
    pub fn init(&self) -> &Node {
        &self.init
    }
}

/// The `let` statement declares a block scope local variable, optionally initializing it to a
/// value.
///
///
/// `let` allows you to declare variables that are limited to a scope of a block statement, or
/// expression on which it is used, unlike the `var` keyword, which defines a variable
/// globally, or locally to an entire function regardless of block scope.
///
/// Just like const the `let` does not create properties of the window object when declared
/// globally (in the top-most scope).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct LetDeclList {
    #[cfg_attr(feature = "serde", serde(flatten))]
    list: Box<[LetDecl]>,
}

impl<T> From<T> for LetDeclList
where
    T: Into<Box<[LetDecl]>>,
{
    fn from(list: T) -> Self {
        Self { list: list.into() }
    }
}

impl From<LetDecl> for LetDeclList {
    fn from(decl: LetDecl) -> Self {
        Self {
            list: Box::new([decl]),
        }
    }
}

impl AsRef<[LetDecl]> for LetDeclList {
    fn as_ref(&self) -> &[LetDecl] {
        &self.list
    }
}

impl fmt::Display for LetDeclList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.list.is_empty() {
            write!(f, "let ")?;
            join_nodes(f, &self.list)
        } else {
            Ok(())
        }
    }
}

impl From<LetDeclList> for Node {
    fn from(list: LetDeclList) -> Self {
        Self::LetDeclList(list)
    }
}

/// Individual constant declaration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct LetDecl {
    name: Identifier,
    init: Option<Node>,
}

impl fmt::Display for LetDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)?;
        if let Some(ref init) = self.init {
            write!(f, " = {}", init)?;
        }
        Ok(())
    }
}

impl LetDecl {
    /// Creates a new variable declaration.
    pub(in crate::syntax) fn new<N, I>(name: N, init: I) -> Self
    where
        N: Into<Identifier>,
        I: Into<Option<Node>>,
    {
        Self {
            name: name.into(),
            init: init.into(),
        }
    }

    /// Gets the name of the variable.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Gets the initialization node for the variable, if any.
    pub fn init(&self) -> Option<&Node> {
        self.init.as_ref()
    }
}
