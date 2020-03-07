use std::{
    error,
    fmt::{Display, Error, Formatter},
    str::FromStr,
};

/// A Javascript Keyword
/// As specificed by <https://www.ecma-international.org/ecma-262/#sec-keywords>
#[derive(Clone, Copy, PartialEq, Debug)]
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

#[derive(Debug, Clone, Copy)]
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
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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
