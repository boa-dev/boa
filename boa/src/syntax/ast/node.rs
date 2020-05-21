//! This module implements the `Node` structure, which composes the AST.

use crate::syntax::ast::{
    constant::Const,
    op::{BinOp, Operator, UnaryOp},
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Javascript AST Node.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum Node {
    /// An array is an ordered collection of data (either primitive or object depending upon the
    /// language).
    ///
    /// Arrays are used to store multiple values in a single variable.
    /// This is compared to a variable that can store only one value.
    ///
    /// Each item in an array has a number attached to it, called a numeric index, that allows you
    /// to access it. In JavaScript, arrays start at index zero and can be manipulated with various
    /// methods.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ArrayLiteral
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array
    ArrayDecl(Box<[Node]>),

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
    ArrowFunctionDecl(Box<[FormalParameter]>, Box<Node>),

    /// An assignment operator assigns a value to its left operand based on the value of its right
    /// operand.
    ///
    /// Assignment operator (`=`), assigns the value of its right operand to its left operand.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators
    Assign(Box<Node>, Box<Node>),

    /// Binary operators requires two operands, one before the operator and one after the operator.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Operators
    BinOp(BinOp, Box<Node>, Box<Node>),

    /// A `block` statement (or compound statement in other languages) is used to group zero or
    /// more statements.
    ///
    /// The block statement is often called compound statement in other languages.
    /// It allows you to use multiple statements where JavaScript expects only one statement.
    /// Combining statements into blocks is a common practice in JavaScript. The opposite behavior
    /// is possible using an empty statement, where you provide no statement, although one is
    /// required.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-BlockStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
    Block(Box<[Node]>),

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
    Break(Option<String>),

    /// Calling the function actually performs the specified actions with the indicated parameters.
    ///
    /// Defining a function does not execute it. Defining it simply names the function and
    /// specifies what to do when the function is called. Functions must be in scope when they are
    /// called, but the function declaration can be hoisted. The scope of a function is the
    /// function in which it is declared (or the entire program, if it is declared at the top
    /// level).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-CallExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions#Calling_functions
    Call(Box<Node>, Box<[Node]>),

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
    ConstDecl(Box<[(String, Node)]>),

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
    Continue(Option<String>),

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

    /// The `function` declaration (function statement) defines a function with the specified
    /// parameters.
    ///
    /// A function created with a function declaration is a `Function` object and has all the
    /// properties, methods and behavior of `Function`.
    ///
    /// A function can also be created using an expression (see function expression).
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
    FunctionDecl(String, Box<[FormalParameter]>, Box<Node>),

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
    FunctionExpr(Option<String>, Box<[FormalParameter]>, Box<Node>),

    /// This property accessor provides access to an object's properties by using the
    /// [dot notation][mdn].
    ///
    /// In the object.property syntax, the property must be a valid JavaScript identifier.
    /// (In the ECMAScript standard, the names of properties are technically "IdentifierNames", not
    /// "Identifiers", so reserved words can be used but are not recommended).
    ///
    /// One can think of an object as an associative array (a.k.a. map, dictionary, hash, lookup
    /// table). The keys in this array are the names of the object's properties.
    ///
    /// It's typical when speaking of an object's properties to make a distinction between
    /// properties and methods. However, the property/method distinction is little more than a
    /// convention. A method is simply a property that can be called (for example, if it has a
    /// reference to a Function instance as its value).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-property-accessors
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_accessors#Dot_notation
    GetConstField(Box<Node>, String),

    /// This property accessor provides access to an object's properties by using the
    /// [bracket notation][mdn].
    ///
    /// In the object[property_name] syntax, the property_name is just a string or
    /// [Symbol][symbol]. So, it can be any string, including '1foo', '!bar!', or even ' ' (a
    /// space).
    ///
    /// One can think of an object as an associative array (a.k.a. map, dictionary, hash, lookup
    /// table). The keys in this array are the names of the object's properties.
    ///
    /// It's typical when speaking of an object's properties to make a distinction between
    /// properties and methods. However, the property/method distinction is little more than a
    /// convention. A method is simply a property that can be called (for example, if it has a
    /// reference to a Function instance as its value).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-property-accessors
    /// [symbol]: https://developer.mozilla.org/en-US/docs/Glossary/Symbol
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_accessors#Bracket_notation
    GetField(Box<Node>, Box<Node>),

    /// The `for` statement creates a loop that consists of three optional expressions.
    ///
    /// A `for` loop repeats until a specified condition evaluates to `false`.
    /// The JavaScript for loop is similar to the Java and C for loop.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ForDeclaration
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
    ForLoop(
        Option<Box<Node>>,
        Option<Box<Node>>,
        Option<Box<Node>>,
        Box<Node>,
    ),

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
    LetDecl(Box<[(String, Option<Node>)]>),

    /// An `identifier` is a sequence of characters in the code that identifies a variable,
    /// function, or property.
    ///
    /// In JavaScript, identifiers are case-sensitive and can contain Unicode letters, $, _, and
    /// digits (0-9), but may not start with a digit.
    ///
    /// An identifier differs from a string in that a string is data, while an identifier is part
    /// of the code. In JavaScript, there is no way to convert identifiers to strings, but
    /// sometimes it is possible to parse strings into identifiers.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-Identifier
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Identifier
    Local(String),

    /// The `new` operator lets developers create an instance of a user-defined object type or of
    /// one of the built-in object types that has a constructor function.
    ///
    /// The new keyword does the following things:
    ///  - Creates a blank, plain JavaScript object;
    ///  - Links (sets the constructor of) this object to another object;
    ///  - Passes the newly created object from Step 1 as the this context;
    ///  - Returns this if the function doesn't return its own object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-NewExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new
    New(Box<Node>),

    /// Objects in JavaScript may be defined as an unordered collection of related data, of
    /// primitive or reference types, in the form of “key: value” pairs.
    ///
    /// Objects can be initialized using `new Object()`, `Object.create()`, or using the literal
    /// notation.
    ///
    /// An object initializer is an expression that describes the initialization of an
    /// [`Object`][object]. Objects consist of properties, which are used to describe an object.
    /// Values of object properties can either contain [`primitive`][primitive] data types or other
    /// objects.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ObjectLiteral
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer
    /// [object]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object
    /// [primitive]: https://developer.mozilla.org/en-US/docs/Glossary/primitive
    Object(Box<[PropertyDefinition]>),

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

    /// The `switch` statement evaluates an expression, matching the expression's value to a case
    /// clause, and executes statements associated with that case, as well as statements in cases
    /// that follow the matching case.
    ///
    /// A `switch` statement first evaluates its expression. It then looks for the first case
    /// clause whose expression evaluates to the same value as the result of the input expression
    /// (using the strict comparison, `===`) and transfers control to that clause, executing the
    /// associated statements. (If multiple cases match the provided value, the first case that
    /// matches is selected, even if the cases are not equal to each other.)
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-SwitchStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
    Switch(Box<Node>, Box<[(Node, Box<[Node]>)]>, Option<Box<Node>>),

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

    /// Similar to `Node::Block` but without the braces
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-StatementList
    StatementList(Box<[Node]>),

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

    /// The `try...catch` statement marks a block of statements to try and specifies a response
    /// should an exception be thrown.
    ///
    /// The `try` statement consists of a `try`-block, which contains one or more statements. `{}`
    /// must always be used, even for single statements. At least one `catch`-block, or a
    /// `finally`-block, must be present.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-TryStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Try(
        Box<Node>,
        Option<Box<Node>>,
        Option<Box<Node>>,
        Option<Box<Node>>,
    ),

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

    /// A unary operation is an operation with only one operand.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
    UnaryOp(UnaryOp, Box<Node>),

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
    VarDecl(Box<[(String, Option<Node>)]>),

    /// The `while` statement creates a loop that executes a specified statement as long as the
    /// test condition evaluates to `true`.
    ///
    /// The condition is evaluated before executing the statement.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-grammar-notation-WhileStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while
    WhileLoop(Box<Node>, Box<Node>),
}

impl Operator for Node {
    fn get_assoc(&self) -> bool {
        match *self {
            Self::UnaryOp(_, _) | Self::If(_, _, _) | Self::Assign(_, _) => false,
            _ => true,
        }
    }

    fn get_precedence(&self) -> u64 {
        match self {
            Self::GetField(_, _) | Self::GetConstField(_, _) => 1,
            Self::Call(_, _) => 2,
            Self::UnaryOp(UnaryOp::IncrementPost, _)
            | Self::UnaryOp(UnaryOp::IncrementPre, _)
            | Self::UnaryOp(UnaryOp::DecrementPost, _)
            | Self::UnaryOp(UnaryOp::DecrementPre, _) => 3,
            Self::UnaryOp(UnaryOp::Not, _)
            | Self::UnaryOp(UnaryOp::Tilde, _)
            | Self::UnaryOp(UnaryOp::Minus, _)
            | Self::UnaryOp(UnaryOp::TypeOf, _) => 4,
            Self::BinOp(op, _, _) => op.get_precedence(),
            Self::If(_, _, _) => 15,
            // 16 should be yield
            Self::Assign(_, _) => 17,
            _ => 19,
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl Node {
    /// Creates an `ArrayDecl` AST node.
    pub fn array_decl<N>(nodes: N) -> Self
    where
        N: Into<Box<[Self]>>,
    {
        Self::ArrayDecl(nodes.into())
    }

    /// Creates an `ArraowFunctionDecl` AST node.
    pub fn arrow_function_decl<P, B>(params: P, body: B) -> Self
    where
        P: Into<Box<[FormalParameter]>>,
        B: Into<Box<Self>>,
    {
        Self::ArrowFunctionDecl(params.into(), body.into())
    }

    /// Creates an `Assign` AST node.
    pub fn assign<L, R>(lhs: L, rhs: R) -> Self
    where
        L: Into<Box<Self>>,
        R: Into<Box<Self>>,
    {
        Self::Assign(lhs.into(), rhs.into())
    }

    /// Creates a `BinOp` AST node.
    pub fn bin_op<O, L, R>(op: O, lhs: L, rhs: R) -> Self
    where
        O: Into<BinOp>,
        L: Into<Box<Self>>,
        R: Into<Box<Self>>,
    {
        Self::BinOp(op.into(), lhs.into(), rhs.into())
    }

    /// Creates a `Block` AST node.
    pub fn block<N>(nodes: N) -> Self
    where
        N: Into<Box<[Self]>>,
    {
        Self::Block(nodes.into())
    }

    /// Creates a `Break` AST node.
    pub fn break_node<OL, L>(label: OL) -> Self
    where
        L: Into<String>,
        OL: Into<Option<L>>,
    {
        Self::Break(label.into().map(L::into))
    }

    /// Creates a `Call` AST node.
    pub fn call<F, P>(function: F, params: P) -> Self
    where
        F: Into<Box<Self>>,
        P: Into<Box<[Self]>>,
    {
        Self::Call(function.into(), params.into())
    }

    /// Creates a `ConditionalOp` AST node.
    pub fn conditional_op<C, T, F>(condition: C, if_true: T, if_false: F) -> Self
    where
        C: Into<Box<Self>>,
        T: Into<Box<Self>>,
        F: Into<Box<Self>>,
    {
        Self::ConditionalOp(condition.into(), if_true.into(), if_false.into())
    }

    /// Creates a `Const` AST node.
    pub fn const_node<C>(node: C) -> Self
    where
        C: Into<Const>,
    {
        Self::Const(node.into())
    }

    /// Creates a `ConstDecl` AST node.
    pub fn const_decl<D>(decl: D) -> Self
    where
        D: Into<Box<[(String, Self)]>>,
    {
        Self::ConstDecl(decl.into())
    }

    /// Creates a `Continue` AST node.
    pub fn continue_node<OL, L>(label: OL) -> Self
    where
        L: Into<String>,
        OL: Into<Option<L>>,
    {
        Self::Continue(label.into().map(L::into))
    }

    /// Creates a `DoWhileLoop` AST node.
    pub fn do_while_loop<B, C>(body: B, condition: C) -> Self
    where
        B: Into<Box<Self>>,
        C: Into<Box<Self>>,
    {
        Self::DoWhileLoop(body.into(), condition.into())
    }

    /// Creates a `FunctionDecl` AST node.
    pub fn function_decl<N, P, B>(name: N, params: P, body: B) -> Self
    where
        N: Into<String>,
        P: Into<Box<[FormalParameter]>>,
        B: Into<Box<Self>>,
    {
        Self::FunctionDecl(name.into(), params.into(), body.into())
    }

    /// Creates a `FunctionDecl` AST node.
    pub fn function_expr<ON, N, P, B>(name: ON, params: P, body: B) -> Self
    where
        N: Into<String>,
        ON: Into<Option<N>>,
        P: Into<Box<[FormalParameter]>>,
        B: Into<Box<Self>>,
    {
        Self::FunctionExpr(name.into().map(N::into), params.into(), body.into())
    }

    /// Creates a `GetConstField` AST node.
    pub fn get_const_field<V, L>(value: V, label: L) -> Self
    where
        V: Into<Box<Self>>,
        L: Into<String>,
    {
        Self::GetConstField(value.into(), label.into())
    }

    /// Creates a `GetField` AST node.
    pub fn get_field<V, F>(value: V, field: F) -> Self
    where
        V: Into<Box<Self>>,
        F: Into<Box<Self>>,
    {
        Self::GetField(value.into(), field.into())
    }

    /// Creates a `ForLoop` AST node.
    pub fn for_loop<OI, OC, OS, I, C, S, B>(init: OI, condition: OC, step: OS, body: B) -> Self
    where
        OI: Into<Option<I>>,
        OC: Into<Option<C>>,
        OS: Into<Option<S>>,
        I: Into<Box<Self>>,
        C: Into<Box<Self>>,
        S: Into<Box<Self>>,
        B: Into<Box<Self>>,
    {
        Self::ForLoop(
            init.into().map(I::into),
            condition.into().map(C::into),
            step.into().map(S::into),
            body.into(),
        )
    }

    /// Creates an `If` AST node.
    pub fn if_node<C, B, E, OE>(condition: C, body: B, else_node: OE) -> Self
    where
        C: Into<Box<Self>>,
        B: Into<Box<Self>>,
        E: Into<Box<Self>>,
        OE: Into<Option<E>>,
    {
        Self::If(condition.into(), body.into(), else_node.into().map(E::into))
    }

    /// Creates a `LetDecl` AST node.
    pub fn let_decl<I>(init: I) -> Self
    where
        I: Into<Box<[(String, Option<Self>)]>>,
    {
        Self::LetDecl(init.into())
    }

    /// Creates a `Local` AST node.
    pub fn local<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self::Local(name.into())
    }

    /// Creates a `New` AST node.
    pub fn new<N>(node: N) -> Self
    where
        N: Into<Box<Self>>,
    {
        Self::New(node.into())
    }

    /// Creates an `Object` AST node.
    pub fn object<D>(def: D) -> Self
    where
        D: Into<Box<[PropertyDefinition]>>,
    {
        Self::Object(def.into())
    }

    /// Creates a `Return` AST node.
    pub fn return_node<E, OE>(expr: OE) -> Self
    where
        E: Into<Box<Self>>,
        OE: Into<Option<E>>,
    {
        Self::Return(expr.into().map(E::into))
    }

    /// Creates a `Switch` AST node.
    pub fn switch<V, C, OD, D>(val: V, cases: C, default: OD) -> Self
    where
        V: Into<Box<Self>>,
        C: Into<Box<[(Self, Box<[Self]>)]>>,
        OD: Into<Option<D>>,
        D: Into<Box<Self>>,
    {
        Self::Switch(val.into(), cases.into(), default.into().map(D::into))
    }

    /// Creates a `Spread` AST node.
    pub fn spread<V>(val: V) -> Self
    where
        V: Into<Box<Self>>,
    {
        Self::Spread(val.into())
    }

    /// Creates a `StatementList` AST node.
    pub fn statement_list<L>(list: L) -> Self
    where
        L: Into<Box<[Self]>>,
    {
        Self::StatementList(list.into())
    }

    /// Creates a `Throw` AST node.
    pub fn throw<V>(val: V) -> Self
    where
        V: Into<Box<Self>>,
    {
        Self::Throw(val.into())
    }

    /// Creates a `Try` AST node.
    pub fn try_node<T, OC, OP, OF, C, P, F>(try_node: T, catch: OC, param: OP, finally: OF) -> Self
    where
        T: Into<Box<Self>>,
        OC: Into<Option<C>>,
        OP: Into<Option<P>>,
        OF: Into<Option<F>>,
        C: Into<Box<Self>>,
        P: Into<Box<Self>>,
        F: Into<Box<Self>>,
    {
        let catch = catch.into().map(C::into);
        let finally = finally.into().map(F::into);

        debug_assert!(
            catch.is_some() || finally.is_some(),
            "try/catch must have a catch or a finally block"
        );

        Self::Try(try_node.into(), catch, param.into().map(P::into), finally)
    }

    /// Creates a `This` AST node.
    pub fn this() -> Self {
        Self::This
    }

    /// Creates a `UnaryOp` AST node.
    pub fn unary_op<V>(op: UnaryOp, val: V) -> Self
    where
        V: Into<Box<Self>>,
    {
        Self::UnaryOp(op, val.into())
    }

    /// Creates a `VarDecl` AST node.
    pub fn var_decl<I>(init: I) -> Self
    where
        I: Into<Box<[(String, Option<Self>)]>>,
    {
        Self::VarDecl(init.into())
    }

    /// Creates a `WhileLoop` AST node.
    pub fn while_loop<C, B>(condition: C, body: B) -> Self
    where
        C: Into<Box<Self>>,
        B: Into<Box<Self>>,
    {
        Self::WhileLoop(condition.into(), body.into())
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
            Self::ForLoop(_, _, _, _) => write!(f, "for loop"), // TODO
            Self::This => write!(f, "this"),
            Self::Try(_, _, _, _) => write!(f, "try/catch/finally"), // TODO
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
            Self::Block(ref block) => {
                writeln!(f, "{{")?;
                for node in block.iter() {
                    node.display(f, indentation + 1)?;

                    match node {
                        Self::Block(_)
                        | Self::If(_, _, _)
                        | Self::Switch(_, _, _)
                        | Self::FunctionDecl(_, _, _)
                        | Self::WhileLoop(_, _)
                        | Self::StatementList(_) => {}
                        _ => write!(f, ";")?,
                    }
                    writeln!(f)?;
                }
                write!(f, "{}}}", indent)
            }
            Self::StatementList(ref list) => {
                for node in list.iter() {
                    node.display(f, indentation + 1)?;

                    match node {
                        Self::Block(_)
                        | Self::If(_, _, _)
                        | Self::Switch(_, _, _)
                        | Self::FunctionDecl(_, _, _)
                        | Self::WhileLoop(_, _)
                        | Self::StatementList(_) => {}
                        _ => write!(f, ";")?,
                    }
                    writeln!(f)?;
                }
                Ok(())
            }
            Self::Local(ref s) => write!(f, "{}", s),
            Self::GetConstField(ref ex, ref field) => write!(f, "{}.{}", ex, field),
            Self::GetField(ref ex, ref field) => write!(f, "{}[{}]", ex, field),
            Self::Call(ref ex, ref args) => {
                write!(f, "{}(", ex)?;
                let arg_strs: Box<[String]> = args.iter().map(ToString::to_string).collect();
                write!(f, "{})", arg_strs.join(", "))
            }
            Self::New(ref call) => {
                let (func, args) = match call.as_ref() {
                    Self::Call(func, args) => (func, args),
                    _ => unreachable!("Node::New(ref call): 'call' must only be Node::Call type."),
                };

                write!(f, "new {}", func)?;
                f.write_str("(")?;
                let mut first = true;
                for e in args.iter() {
                    if !first {
                        f.write_str(", ")?;
                    }
                    first = false;
                    write!(f, "{}", e)?;
                }
                f.write_str(")")
            }
            Self::WhileLoop(ref cond, ref node) => {
                write!(f, "while ({}) ", cond)?;
                node.display(f, indentation)
            }
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
            Self::Switch(ref val, ref vals, None) => {
                writeln!(f, "switch ({}) {{", val)?;
                for e in vals.iter() {
                    writeln!(f, "{}case {}:", indent, e.0)?;
                    join_nodes(f, &e.1)?;
                }
                writeln!(f, "{}}}", indent)
            }
            Self::Switch(ref val, ref vals, Some(ref def)) => {
                writeln!(f, "switch ({}) {{", val)?;
                for e in vals.iter() {
                    writeln!(f, "{}case {}:", indent, e.0)?;
                    join_nodes(f, &e.1)?;
                }
                writeln!(f, "{}default:", indent)?;
                def.display(f, indentation + 1)?;
                write!(f, "{}}}", indent)
            }
            Self::Object(ref properties) => {
                f.write_str("{\n")?;
                for property in properties.iter() {
                    match property {
                        PropertyDefinition::IdentifierReference(key) => {
                            write!(f, "{}    {},", indent, key)?;
                        }
                        PropertyDefinition::Property(key, value) => {
                            write!(f, "{}    {}: {},", indent, key, value)?;
                        }
                        PropertyDefinition::SpreadObject(key) => {
                            write!(f, "{}    ...{},", indent, key)?;
                        }
                        PropertyDefinition::MethodDefinition(_kind, _key, _node) => {
                            // TODO: Implement display for PropertyDefinition::MethodDefinition.
                            unimplemented!("Display for PropertyDefinition::MethodDefinition");
                        }
                    }
                }
                f.write_str("}")
            }
            Self::ArrayDecl(ref arr) => {
                f.write_str("[")?;
                join_nodes(f, arr)?;
                f.write_str("]")
            }
            Self::FunctionDecl(ref name, ref _args, ref node) => {
                write!(f, "function {} {{", name)?;
                //join_nodes(f, args)?; TODO: port
                f.write_str("} ")?;
                node.display(f, indentation + 1)
            }
            Self::FunctionExpr(ref name, ref args, ref node) => {
                write!(f, "function ")?;
                if let Some(func_name) = name {
                    write!(f, "{}", func_name)?;
                }
                write!(f, "{{")?;
                join_nodes(f, args)?;
                f.write_str("} ")?;
                node.display(f, indentation + 1)
            }
            Self::ArrowFunctionDecl(ref args, ref node) => {
                write!(f, "(")?;
                join_nodes(f, args)?;
                f.write_str(") => ")?;
                node.display(f, indentation)
            }
            Self::BinOp(ref op, ref a, ref b) => write!(f, "{} {} {}", a, op, b),
            Self::UnaryOp(ref op, ref a) => write!(f, "{}{}", op, a),
            Self::Return(Some(ref ex)) => write!(f, "return {}", ex),
            Self::Return(None) => write!(f, "return"),
            Self::Throw(ref ex) => write!(f, "throw {}", ex),
            Self::Assign(ref ref_e, ref val) => write!(f, "{} = {}", ref_e, val),
            Self::VarDecl(ref vars) | Self::LetDecl(ref vars) => {
                if let Self::VarDecl(_) = *self {
                    f.write_str("var ")?;
                } else {
                    f.write_str("let ")?;
                }
                for (key, val) in vars.iter() {
                    match val {
                        Some(x) => write!(f, "{} = {}", key, x)?,
                        None => write!(f, "{}", key)?,
                    }
                }
                Ok(())
            }
            Self::ConstDecl(ref vars) => {
                f.write_str("const ")?;
                for (key, val) in vars.iter() {
                    write!(f, "{} = {}", key, val)?
                }
                Ok(())
            }
        }
    }
}

/// Utility to join multiple Nodes into a single string.
fn join_nodes<N>(f: &mut fmt::Formatter<'_>, nodes: &[N]) -> fmt::Result
where
    N: fmt::Display,
{
    let mut first = true;
    for e in nodes {
        if !first {
            f.write_str(", ")?;
        }
        first = false;
        write!(f, "{}", e)?;
    }
    Ok(())
}

/// "Formal parameter" is a fancy way of saying "function parameter".
///
/// In the declaration of a function, the parameters must be identifiers,
/// not any value like numbers, strings, or objects.
///```text
///function foo(formalParametar1, formalParametar2) {
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
    pub name: String,
    pub init: Option<Box<Node>>,
    pub is_rest_param: bool,
}

impl FormalParameter {
    pub fn new<N>(name: N, init: Option<Box<Node>>, is_rest_param: bool) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            init,
            is_rest_param,
        }
    }
}

impl fmt::Display for FormalParameter {
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
    IdentifierReference(String),

    /// Binds a property name to a JavaScript value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Property_definitions
    Property(String, Node),

    /// A property of an object can also refer to a function or a getter or setter method.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#Method_definitions
    MethodDefinition(MethodDefinitionKind, String, Node),

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
        I: Into<String>,
    {
        Self::IdentifierReference(ident.into())
    }

    /// Creates a `Property` definition.
    pub fn property<N, V>(name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<Node>,
    {
        Self::Property(name.into(), value.into())
    }

    /// Creates a `MethodDefinition`.
    pub fn method_definition<N, B>(kind: MethodDefinitionKind, name: N, body: B) -> Self
    where
        N: Into<String>,
        B: Into<Node>,
    {
        Self::MethodDefinition(kind, name.into(), body.into())
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
