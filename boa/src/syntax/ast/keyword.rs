//! This module implements the `Keyword` structure, which represents reserved words of the JavaScript language.

use std::{
    error,
    fmt::{Display, Error, Formatter},
    str::FromStr,
};

#[cfg(feature = "serde-ast")]
use serde::{Deserialize, Serialize};

/// Keywords are tokens that have special meaning in JavaScript.
///
/// In JavaScript you cannot use these reserved words as variables, labels, or function names.
///
/// More information:
///  - [ECMAScript reference](https://www.ecma-international.org/ecma-262/#sec-keywords)
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#Keywords
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Keyword {
    /// The `await` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AwaitExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
    Await,

    /// The `break` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-BreakStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
    Break,

    /// The `case` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-CaseClause)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
    Case,

    /// The `catch` keyword.
    /// 
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-Catch)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Catch,

    /// The `class` keyword.
    ///
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ClassDeclaration)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
    Class,

    /// The continue keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ContinueStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
    Continue,

    /// The `const` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-let-and-const-declarations)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const
    Const,

    /// The `debugger` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-debugger-statement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/debugger
    Debugger,

    /// The `default` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference default clause](https://tc39.es/ecma262/#prod-DefaultClause)
    ///  - [ECMAScript reference default export](https://tc39.es/ecma262/#prod-ImportedDefaultBinding)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/default
    Default,

    /// The `delete` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-delete-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/delete
    Delete,

    /// The `do` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-do-while-statement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
    Do,

    ///
    Else,

    /// The `enum` keyword
    ///
    /// Future reserved keywords.
    Enum,

    /// The `export` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-exports)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/export
    Export,

    /// The `extends` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ClassHeritage)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/extends
    Extends,

    /// The `finally` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-Finally)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Finally,

    /// The `for` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ForDeclaration)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
    For,

    /// The `function` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-terms-and-definitions-function)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
    Function,

    /// The `if` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-IfStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
    If,

    /// The `in` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-RelationalExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/in
    In,

    /// The `instanceof` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-instanceofoperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/instanceof
    InstanceOf,

    /// The`import` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-imports)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import
    Import,

    /// The `let` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-let-and-const-declarations)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let
    Let,

    /// The `new` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-NewExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new
    New,

    /// The `return` keyword
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ReturnStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
    Return,

    /// The `super` keyword
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-super-keyword)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/super
    Super,

    /// The `switch` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-SwitchStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
    Switch,

    /// The `this` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-this-keyword)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this
    This,

    /// The `throw` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ArrowFunction)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
    Throw,

    /// The `try` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-TryStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Try,

    /// The `typeof` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-typeof-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof
    TypeOf,

    /// The `var` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-VariableStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
    Var,

    /// The `void` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-void-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/void
    Void,

    /// The `while` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-grammar-notation-WhileStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while
    While,

    /// The `with` keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-WithStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/with
    With,

    /// The 'yield' keyword.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-YieldExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
    Yield,
}

#[derive(Debug, Clone, Copy)]
pub struct KeywordError;
impl Display for KeywordError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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
            "await" => Ok(Keyword::Await),
            "break" => Ok(Keyword::Break),
            "case" => Ok(Keyword::Case),
            "catch" => Ok(Keyword::Catch),
            "class" => Ok(Keyword::Class),
            "continue" => Ok(Keyword::Continue),
            "const" => Ok(Keyword::Const),
            "debugger" => Ok(Keyword::Debugger),
            "default" => Ok(Keyword::Default),
            "delete" => Ok(Keyword::Delete),
            "do" => Ok(Keyword::Do),
            "else" => Ok(Keyword::Else),
            "enum" => Ok(Keyword::Enum),
            "extends" => Ok(Keyword::Extends),
            "export" => Ok(Keyword::Export),
            "finally" => Ok(Keyword::Finally),
            "for" => Ok(Keyword::For),
            "function" => Ok(Keyword::Function),
            "if" => Ok(Keyword::If),
            "in" => Ok(Keyword::In),
            "instanceof" => Ok(Keyword::InstanceOf),
            "import" => Ok(Keyword::Import),
            "let" => Ok(Keyword::Let),
            "new" => Ok(Keyword::New),
            "return" => Ok(Keyword::Return),
            "super" => Ok(Keyword::Super),
            "switch" => Ok(Keyword::Switch),
            "this" => Ok(Keyword::This),
            "throw" => Ok(Keyword::Throw),
            "try" => Ok(Keyword::Try),
            "typeof" => Ok(Keyword::TypeOf),
            "var" => Ok(Keyword::Var),
            "void" => Ok(Keyword::Void),
            "while" => Ok(Keyword::While),
            "with" => Ok(Keyword::With),
            "yield" => Ok(Keyword::Yield),
            _ => Err(KeywordError),
        }
    }
}
impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match *self {
                Keyword::Await => "await",
                Keyword::Break => "break",
                Keyword::Case => "case",
                Keyword::Catch => "catch",
                Keyword::Class => "class",
                Keyword::Continue => "continue",
                Keyword::Const => "const",
                Keyword::Debugger => "debugger",
                Keyword::Default => "default",
                Keyword::Delete => "delete",
                Keyword::Do => "do",
                Keyword::Else => "else",
                Keyword::Enum => "enum",
                Keyword::Extends => "extends",
                Keyword::Export => "export",
                Keyword::Finally => "finally",
                Keyword::For => "for",
                Keyword::Function => "function",
                Keyword::If => "if",
                Keyword::In => "in",
                Keyword::InstanceOf => "instanceof",
                Keyword::Import => "import",
                Keyword::Let => "let",
                Keyword::New => "new",
                Keyword::Return => "return",
                Keyword::Super => "super",
                Keyword::Switch => "switch",
                Keyword::This => "this",
                Keyword::Throw => "throw",
                Keyword::Try => "try",
                Keyword::TypeOf => "typeof",
                Keyword::Var => "var",
                Keyword::Void => "void",
                Keyword::While => "while",
                Keyword::With => "with",
                Keyword::Yield => "yield",
            }
        )
    }
}
