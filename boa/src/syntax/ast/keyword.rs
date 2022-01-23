//! This module implements the `Keyword` structure, which represents reserved words of the JavaScript language.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://www.ecma-international.org/ecma-262/#sec-keywords
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#Keywords

use crate::syntax::ast::op::{BinOp, CompOp};
use boa_interner::{Interner, Sym};
use std::{convert::TryInto, error, fmt, str::FromStr};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// Keywords are tokens that have special meaning in JavaScript.
///
/// In JavaScript you cannot use these reserved words as variables, labels, or function names.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://www.ecma-international.org/ecma-262/#sec-keywords
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#Keywords
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Keyword {
    /// The `await` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AwaitExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
    Await,

    /// The `async` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AsyncMethod
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function
    Async,

    /// The `break` keyword.
    ///
    /// More information:
    ///  - [break `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-BreakStatement
    /// [node]: ../node/enum.Node.html#variant.Break
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
    Break,

    /// The `case` keyword.
    ///
    /// More information:
    ///  - [switch `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-CaseClause
    /// [node]: ../node/enum.Node.html#variant.Switch
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
    Case,

    /// The `catch` keyword.
    ///
    /// More information:
    ///  - [try `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-Catch
    /// [node]: ../node/enum.Node.html#variant.Try
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Catch,

    /// The `class` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ClassDeclaration
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
    Class,

    /// The `continue` keyword.
    ///
    /// More information:
    ///  - [continue `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ContinueStatement
    /// [node]: ../node/enum.Node.html#variant.Continue
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
    Continue,

    /// The `const` keyword.
    ///
    /// More information:
    ///  - [const `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
    /// [node]: ../node/enum.Node.html#variant.ConstDecl
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const
    Const,

    /// The `debugger` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-debugger-statement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/debugger
    Debugger,

    /// The `default` keyword.
    ///
    /// More information:
    ///  - [switch `Node` documentation][node]
    ///  - [ECMAScript reference default clause][spec-clause]
    ///  - [ECMAScript reference default export][spec-export]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.Switch
    /// [spec-clause]: https://tc39.es/ecma262/#prod-DefaultClause
    /// [spec-export]: https://tc39.es/ecma262/#prod-ImportedDefaultBinding
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/default
    Default,

    /// The `delete` keyword.
    ///
    /// More information:
    ///  - [delete `UnaryOp` documentation][unary]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-delete-operator
    /// [unary]: ../op/enum.UnaryOp.html#variant.Delete
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/delete
    Delete,

    /// The `do` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-do-while-statement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
    Do,

    /// The `else` keyword.
    ///
    /// More information:
    ///  - [if `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.If
    /// [spec]: https://tc39.es/ecma262/#prod-IfStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
    Else,

    /// The `enum` keyword.
    ///
    /// Future reserved keyword.
    Enum,

    /// The `export` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-exports
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/export
    Export,

    /// The `extends` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ClassHeritage
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/extends
    Extends,

    /// The `false` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-BooleanLiteral
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean
    False,

    /// The `finally` keyword.
    ///
    /// More information:
    ///  - [try `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.Try
    /// [spec]: https://tc39.es/ecma262/#prod-Finally
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Finally,

    /// The `for` keyword.
    ///
    /// More information:
    ///  - [for loop `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.ForLoop
    /// [spec]: https://tc39.es/ecma262/#prod-ForDeclaration
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
    For,

    /// The `function` keyword.
    ///
    /// More information:
    ///  - [function `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.FunctionDecl
    /// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-function
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
    Function,

    /// The `if` keyword.
    ///
    /// More information:
    ///  - [if `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.If
    /// [spec]: https://tc39.es/ecma262/#prod-IfStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
    If,

    /// The `in` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-RelationalExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/in
    In,

    /// The `instanceof` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-instanceofoperator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/instanceof
    InstanceOf,

    /// The `import` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-imports
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import
    Import,

    /// The `let` keyword.
    ///
    /// More information:
    ///  - [let `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.LetDecl
    /// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let
    Let,

    /// The `new` keyword.
    ///
    /// More information:
    ///  - [new `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.New
    /// [spec]: https://tc39.es/ecma262/#prod-NewExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new
    New,

    /// The `null` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-NullLiteral
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/null
    Null,

    /// The `of` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-for-in-and-for-of-statements
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...of
    Of,

    /// The `return` keyword
    ///
    /// More information:
    ///  - [return `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.Return
    /// [spec]: https://tc39.es/ecma262/#prod-ReturnStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
    Return,

    /// The `super` keyword
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-super-keyword
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/super
    Super,

    /// The `switch` keyword.
    ///
    /// More information:
    ///  - [switch `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.Switch
    /// [spec]: https://tc39.es/ecma262/#prod-SwitchStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
    Switch,

    /// The `this` keyword.
    ///
    /// More information:
    ///  - [this `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.This
    /// [spec]: https://tc39.es/ecma262/#sec-this-keyword
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this
    This,

    /// The `throw` keyword.
    ///
    /// More information:
    ///  - [throw `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.Throw
    /// [spec]: https://tc39.es/ecma262/#prod-ArrowFunction
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
    Throw,

    /// The `true` keyword
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-BooleanLiteral
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean
    True,

    /// The `try` keyword.
    ///
    /// More information:
    ///  - [try `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.Try
    /// [spec]: https://tc39.es/ecma262/#prod-TryStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Try,

    /// The `typeof` keyword.
    ///
    /// More information:
    ///  - [typeof `UnaryOp` documentation][unary]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [unary]: ../op/enum.UnaryOp.html#variant.TypeOf
    /// [spec]: https://tc39.es/ecma262/#sec-typeof-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof
    TypeOf,

    /// The `var` keyword.
    ///
    /// More information:
    ///  - [var `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.VarDecl
    /// [spec]: https://tc39.es/ecma262/#prod-VariableStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
    Var,

    /// The `void` keyword.
    ///
    /// More information:
    ///  - [void `UnaryOp` documentation][unary]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [unary]: ../op/enum.UnaryOp.html#variant.Void
    /// [spec]: https://tc39.es/ecma262/#sec-void-operator
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/void
    Void,

    /// The `while` keyword.
    ///
    /// More information:
    ///  - [while `Node` documentation][node]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [node]: ../node/enum.Node.html#variant.While
    /// [spec]: https://tc39.es/ecma262/#prod-grammar-notation-WhileStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while
    While,

    /// The `with` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-WithStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/with
    With,

    /// The 'yield' keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-YieldExpression
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
    Yield,
}

impl Keyword {
    /// Gets the keyword as a binary operation, if this keyword is the `in` keyword.
    pub fn as_binop(self) -> Option<BinOp> {
        match self {
            Keyword::In => Some(BinOp::Comp(CompOp::In)),
            Keyword::InstanceOf => Some(BinOp::Comp(CompOp::InstanceOf)),
            _ => None,
        }
    }

    /// Gets the keyword as a string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Await => "await",
            Self::Async => "async",
            Self::Break => "break",
            Self::Case => "case",
            Self::Catch => "catch",
            Self::Class => "class",
            Self::Continue => "continue",
            Self::Const => "const",
            Self::Debugger => "debugger",
            Self::Default => "default",
            Self::Delete => "delete",
            Self::Do => "do",
            Self::Else => "else",
            Self::Enum => "enum",
            Self::Extends => "extends",
            Self::Export => "export",
            Self::False => "false",
            Self::Finally => "finally",
            Self::For => "for",
            Self::Function => "function",
            Self::If => "if",
            Self::In => "in",
            Self::InstanceOf => "instanceof",
            Self::Import => "import",
            Self::Let => "let",
            Self::New => "new",
            Self::Null => "null",
            Self::Of => "of",
            Self::Return => "return",
            Self::Super => "super",
            Self::Switch => "switch",
            Self::This => "this",
            Self::Throw => "throw",
            Self::True => "true",
            Self::Try => "try",
            Self::TypeOf => "typeof",
            Self::Var => "var",
            Self::Void => "void",
            Self::While => "while",
            Self::With => "with",
            Self::Yield => "yield",
        }
    }

    /// Converts the keyword to a symbol in the given interner.
    pub fn to_sym(self, interner: &mut Interner) -> Sym {
        interner.get_or_intern_static(self.as_str())
    }
}

impl TryInto<BinOp> for Keyword {
    type Error = String;
    fn try_into(self) -> Result<BinOp, Self::Error> {
        self.as_binop()
            .ok_or_else(|| format!("No binary operation for {}", self))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeywordError;
impl fmt::Display for KeywordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid token")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for KeywordError {
    fn description(&self) -> &str {
        "invalid token"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
impl FromStr for Keyword {
    type Err = KeywordError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "await" => Ok(Self::Await),
            "async" => Ok(Self::Async),
            "break" => Ok(Self::Break),
            "case" => Ok(Self::Case),
            "catch" => Ok(Self::Catch),
            "class" => Ok(Self::Class),
            "continue" => Ok(Self::Continue),
            "const" => Ok(Self::Const),
            "debugger" => Ok(Self::Debugger),
            "default" => Ok(Self::Default),
            "delete" => Ok(Self::Delete),
            "do" => Ok(Self::Do),
            "else" => Ok(Self::Else),
            "enum" => Ok(Self::Enum),
            "extends" => Ok(Self::Extends),
            "export" => Ok(Self::Export),
            "false" => Ok(Self::False),
            "finally" => Ok(Self::Finally),
            "for" => Ok(Self::For),
            "function" => Ok(Self::Function),
            "if" => Ok(Self::If),
            "in" => Ok(Self::In),
            "instanceof" => Ok(Self::InstanceOf),
            "import" => Ok(Self::Import),
            "let" => Ok(Self::Let),
            "new" => Ok(Self::New),
            "null" => Ok(Self::Null),
            "of" => Ok(Self::Of),
            "return" => Ok(Self::Return),
            "super" => Ok(Self::Super),
            "switch" => Ok(Self::Switch),
            "this" => Ok(Self::This),
            "throw" => Ok(Self::Throw),
            "true" => Ok(Self::True),
            "try" => Ok(Self::Try),
            "typeof" => Ok(Self::TypeOf),
            "var" => Ok(Self::Var),
            "void" => Ok(Self::Void),
            "while" => Ok(Self::While),
            "with" => Ok(Self::With),
            "yield" => Ok(Self::Yield),
            _ => Err(KeywordError),
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}
