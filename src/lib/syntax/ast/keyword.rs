use std::error;
use std::fmt::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use syntax::ast::keyword::Keyword::*;

#[derive(Clone, PartialEq)]
/// A Javascript Keyword
pub enum Keyword {
    /// The `break` keyword
    KBreak,
    /// The `case` keyword
    KCase,
    /// The `catch` keyword
    KCatch,
    /// The `class` keyword, which is reserved for future use
    KClass,
    /// The `continue` keyword
    KContinue,
    /// The `debugger` keyword
    KDebugger,
    /// The `default` keyword
    KDefault,
    /// The `delete` keyword
    KDelete,
    /// The `do` keyword
    KDo,
    /// The `else` keyword
    KElse,
    /// The `enum` keyword
    KEnum,
    /// The `extends` keyword
    KExtends,
    /// The `finally` keyword
    KFinally,
    /// The `for` keyword
    KFor,
    /// The `function` keyword
    KFunction,
    /// The `if` keyword
    KIf,
    /// The `in` keyword
    KIn,
    /// The `instanceof` keyword
    KInstanceOf,
    /// The `import` keyword
    KImport,
    /// The `new` keyword
    KNew,
    /// The `return` keyword
    KReturn,
    /// The `super` keyword
    KSuper,
    /// The `switch` keyword
    KSwitch,
    /// The `this` keyword
    KThis,
    /// The `throw` keyword
    KThrow,
    /// The `try` keyword
    KTry,
    /// The `typeof` keyword
    KTypeOf,
    /// The `var` keyword
    KVar,
    /// The `void` keyword
    KVoid,
    /// The `while` keyword
    KWhile,
    /// The `with` keyword
    KWith,
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

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
impl FromStr for Keyword {
    type Err = KeywordError;
    fn from_str(s: &str) -> Result<Keyword, Self::Err> {
        match s {
            "break" => Ok(KBreak),
            "case" => Ok(KCase),
            "catch" => Ok(KCatch),
            "class" => Ok(KClass),
            "continue" => Ok(KContinue),
            "debugger" => Ok(KDebugger),
            "default" => Ok(KDefault),
            "delete" => Ok(KDelete),
            "do" => Ok(KDo),
            "else" => Ok(KElse),
            "enum" => Ok(KEnum),
            "extends" => Ok(KExtends),
            "finally" => Ok(KFinally),
            "for" => Ok(KFor),
            "function" => Ok(KFunction),
            "if" => Ok(KIf),
            "in" => Ok(KIn),
            "instanceof" => Ok(KInstanceOf),
            "import" => Ok(KImport),
            "new" => Ok(KNew),
            "return" => Ok(KReturn),
            "super" => Ok(KSuper),
            "switch" => Ok(KSwitch),
            "this" => Ok(KThis),
            "throw" => Ok(KThrow),
            "try" => Ok(KTry),
            "typeof" => Ok(KTypeOf),
            "var" => Ok(KVar),
            "void" => Ok(KVoid),
            "while" => Ok(KWhile),
            "with" => Ok(KWith),
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
                KBreak => "break",
                KCase => "case",
                KCatch => "catch",
                KClass => "class",
                KContinue => "continue",
                KDebugger => "debugger",
                KDefault => "default",
                KDelete => "delete",
                KDo => "do",
                KElse => "else",
                KEnum => "enum",
                KExtends => "extends",
                KFinally => "finally",
                KFor => "for",
                KFunction => "function",
                KIf => "if",
                KIn => "in",
                KInstanceOf => "instanceof",
                KImport => "import",
                KNew => "new",
                KReturn => "return",
                KSuper => "super",
                KSwitch => "switch",
                KThis => "this",
                KThrow => "throw",
                KTry => "try",
                KTypeOf => "typeof",
                KVar => "var",
                KVoid => "void",
                KWhile => "while",
                KWith => "with",
            }
        )
    }
}
