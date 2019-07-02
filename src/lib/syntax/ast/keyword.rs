use crate::syntax::ast::keyword::Keyword::*;
use std::error;
use std::fmt::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
/// A Javascript Keyword
/// As specificed by https://www.ecma-international.org/ecma-262/#sec-keywords
pub enum Keyword {
    /// The `await` keyword
    Await,
    /// The `break` keyword
    Break,
    /// The `case` keyword
    Case,
    /// The `catch` keyword
    Catch,
    /// The `class` keyword, which is reserved for future use
    Class,
    /// The `continue` keyword
    Continue,
    /// The `const` keyword
    Const,
    /// The `debugger` keyword
    Debugger,
    /// The `default` keyword
    Default,
    /// The `delete` keyword
    Delete,
    /// The `do` keyword
    Do,
    /// The `else` keyword
    Else,
    /// The `enum` keyword
    Enum,
    /// The `export` keyword
    Export,
    /// The `extends` keyword
    Extends,
    /// The `finally` keyword
    Finally,
    /// The `for` keyword
    For,
    /// The `function` keyword
    Function,
    /// The `if` keyword
    If,
    /// The `in` keyword
    In,
    /// The `instanceof` keyword
    InstanceOf,
    /// The `import` keyword
    Import,
    /// The `let` keyword
    Let,
    /// The `new` keyword
    New,
    /// The `return` keyword
    Return,
    /// The `super` keyword
    Super,
    /// The `switch` keyword
    Switch,
    /// The `this` keyword
    This,
    /// The `throw` keyword
    Throw,
    /// The `try` keyword
    Try,
    /// The `typeof` keyword
    TypeOf,
    /// The `var` keyword
    Var,
    /// The `void` keyword
    Void,
    /// The `while` keyword
    While,
    /// The `with` keyword
    With,
    /// The 'yield' keyword
    Yield,
}

#[derive(Debug, Clone)]
pub struct KeywordError;
impl Display for KeywordError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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
    fn from_str(s: &str) -> Result<Keyword, Self::Err> {
        match s {
            "await" => Ok(Await),
            "break" => Ok(Break),
            "case" => Ok(Case),
            "catch" => Ok(Catch),
            "class" => Ok(Class),
            "continue" => Ok(Continue),
            "const" => Ok(Const),
            "debugger" => Ok(Debugger),
            "default" => Ok(Default),
            "delete" => Ok(Delete),
            "do" => Ok(Do),
            "else" => Ok(Else),
            "enum" => Ok(Enum),
            "extends" => Ok(Extends),
            "export" => Ok(Export),
            "finally" => Ok(Finally),
            "for" => Ok(For),
            "function" => Ok(Function),
            "if" => Ok(If),
            "in" => Ok(In),
            "instanceof" => Ok(InstanceOf),
            "import" => Ok(Import),
            "let" => Ok(Let),
            "new" => Ok(New),
            "return" => Ok(Return),
            "super" => Ok(Super),
            "switch" => Ok(Switch),
            "this" => Ok(This),
            "throw" => Ok(Throw),
            "try" => Ok(Try),
            "typeof" => Ok(TypeOf),
            "var" => Ok(Var),
            "void" => Ok(Void),
            "while" => Ok(While),
            "with" => Ok(With),
            "yield" => Ok(Yield),
            _ => Err(KeywordError),
        }
    }
}
impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match *self {
                Await => "await",
                Break => "break",
                Case => "case",
                Catch => "catch",
                Class => "class",
                Continue => "continue",
                Const => "const",
                Debugger => "debugger",
                Default => "default",
                Delete => "delete",
                Do => "do",
                Else => "else",
                Enum => "enum",
                Extends => "extends",
                Export => "export",
                Finally => "finally",
                For => "for",
                Function => "function",
                If => "if",
                In => "in",
                InstanceOf => "instanceof",
                Import => "import",
                Let => "let",
                New => "new",
                Return => "return",
                Super => "super",
                Switch => "switch",
                This => "this",
                Throw => "throw",
                Try => "try",
                TypeOf => "typeof",
                Var => "var",
                Void => "void",
                While => "while",
                With => "with",
                Yield => "yield",
            }
        )
    }
}
