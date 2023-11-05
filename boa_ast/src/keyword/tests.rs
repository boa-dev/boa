#![allow(clippy::cognitive_complexity)]

use super::*;

/// Gets a list of all the keywords.
fn all_keywords() -> impl Iterator<Item = Keyword> {
    [
        Keyword::Await,
        Keyword::Async,
        Keyword::Break,
        Keyword::Case,
        Keyword::Catch,
        Keyword::Class,
        Keyword::Continue,
        Keyword::Const,
        Keyword::Debugger,
        Keyword::Default,
        Keyword::Delete,
        Keyword::Do,
        Keyword::Else,
        Keyword::Enum,
        Keyword::Export,
        Keyword::Extends,
        Keyword::False,
        Keyword::Finally,
        Keyword::For,
        Keyword::Function,
        Keyword::If,
        Keyword::In,
        Keyword::InstanceOf,
        Keyword::Import,
        Keyword::Let,
        Keyword::New,
        Keyword::Null,
        Keyword::Of,
        Keyword::Return,
        Keyword::Super,
        Keyword::Switch,
        Keyword::This,
        Keyword::Throw,
        Keyword::True,
        Keyword::Try,
        Keyword::TypeOf,
        Keyword::Var,
        Keyword::Void,
        Keyword::While,
        Keyword::With,
        Keyword::Yield,
    ]
    .into_iter()
}

#[test]
fn as_binary_op() {
    for k in all_keywords() {
        match k.as_binary_op() {
            Some(BinaryOp::Relational(RelationalOp::InstanceOf)) => {
                assert_eq!(k, Keyword::InstanceOf);
            }
            Some(BinaryOp::Relational(RelationalOp::In)) => assert_eq!(k, Keyword::In),
            None => {
                assert_ne!(k, Keyword::InstanceOf);
                assert_ne!(k, Keyword::In);
            }
            _ => unreachable!("unknown binary operator for keyword {k:?} found"),
        }
    }
}

#[test]
fn as_str() {
    for k in all_keywords() {
        match k.as_str() {
            "await" => {
                assert_eq!(k, Keyword::Await);
            }
            "async" => {
                assert_eq!(k, Keyword::Async);
            }
            "break" => {
                assert_eq!(k, Keyword::Break);
            }
            "case" => {
                assert_eq!(k, Keyword::Case);
            }
            "catch" => {
                assert_eq!(k, Keyword::Catch);
            }
            "class" => {
                assert_eq!(k, Keyword::Class);
            }
            "continue" => {
                assert_eq!(k, Keyword::Continue);
            }
            "const" => {
                assert_eq!(k, Keyword::Const);
            }
            "debugger" => {
                assert_eq!(k, Keyword::Debugger);
            }
            "default" => {
                assert_eq!(k, Keyword::Default);
            }
            "delete" => {
                assert_eq!(k, Keyword::Delete);
            }
            "do" => {
                assert_eq!(k, Keyword::Do);
            }
            "else" => {
                assert_eq!(k, Keyword::Else);
            }
            "enum" => {
                assert_eq!(k, Keyword::Enum);
            }
            "extends" => {
                assert_eq!(k, Keyword::Extends);
            }
            "export" => {
                assert_eq!(k, Keyword::Export);
            }
            "false" => {
                assert_eq!(k, Keyword::False);
            }
            "finally" => {
                assert_eq!(k, Keyword::Finally);
            }
            "for" => {
                assert_eq!(k, Keyword::For);
            }
            "function" => {
                assert_eq!(k, Keyword::Function);
            }
            "if" => {
                assert_eq!(k, Keyword::If);
            }
            "in" => {
                assert_eq!(k, Keyword::In);
            }
            "instanceof" => {
                assert_eq!(k, Keyword::InstanceOf);
            }
            "import" => {
                assert_eq!(k, Keyword::Import);
            }
            "let" => {
                assert_eq!(k, Keyword::Let);
            }
            "new" => {
                assert_eq!(k, Keyword::New);
            }
            "null" => {
                assert_eq!(k, Keyword::Null);
            }
            "of" => {
                assert_eq!(k, Keyword::Of);
            }
            "return" => {
                assert_eq!(k, Keyword::Return);
            }
            "super" => {
                assert_eq!(k, Keyword::Super);
            }
            "switch" => {
                assert_eq!(k, Keyword::Switch);
            }
            "this" => {
                assert_eq!(k, Keyword::This);
            }
            "throw" => {
                assert_eq!(k, Keyword::Throw);
            }
            "true" => {
                assert_eq!(k, Keyword::True);
            }
            "try" => {
                assert_eq!(k, Keyword::Try);
            }
            "typeof" => {
                assert_eq!(k, Keyword::TypeOf);
            }
            "var" => {
                assert_eq!(k, Keyword::Var);
            }
            "void" => {
                assert_eq!(k, Keyword::Void);
            }
            "while" => {
                assert_eq!(k, Keyword::While);
            }
            "with" => {
                assert_eq!(k, Keyword::With);
            }
            "yield" => {
                assert_eq!(k, Keyword::Yield);
            }
            _ => unreachable!("unknown keyword {k:?} found"),
        }
    }
}

#[test]
fn to_sym() {
    for k in all_keywords() {
        match k.to_sym() {
            Sym::AWAIT => assert_eq!(k, Keyword::Await),
            Sym::ASYNC => assert_eq!(k, Keyword::Async),
            Sym::BREAK => assert_eq!(k, Keyword::Break),
            Sym::CASE => assert_eq!(k, Keyword::Case),
            Sym::CATCH => assert_eq!(k, Keyword::Catch),
            Sym::CLASS => assert_eq!(k, Keyword::Class),
            Sym::CONTINUE => assert_eq!(k, Keyword::Continue),
            Sym::CONST => assert_eq!(k, Keyword::Const),
            Sym::DEBUGGER => assert_eq!(k, Keyword::Debugger),
            Sym::DEFAULT => assert_eq!(k, Keyword::Default),
            Sym::DELETE => assert_eq!(k, Keyword::Delete),
            Sym::DO => assert_eq!(k, Keyword::Do),
            Sym::ELSE => assert_eq!(k, Keyword::Else),
            Sym::ENUM => assert_eq!(k, Keyword::Enum),
            Sym::EXPORT => assert_eq!(k, Keyword::Export),
            Sym::EXTENDS => assert_eq!(k, Keyword::Extends),
            Sym::FALSE => assert_eq!(k, Keyword::False),
            Sym::FINALLY => assert_eq!(k, Keyword::Finally),
            Sym::FOR => assert_eq!(k, Keyword::For),
            Sym::FUNCTION => assert_eq!(k, Keyword::Function),
            Sym::IF => assert_eq!(k, Keyword::If),
            Sym::IN => assert_eq!(k, Keyword::In),
            Sym::INSTANCEOF => assert_eq!(k, Keyword::InstanceOf),
            Sym::IMPORT => assert_eq!(k, Keyword::Import),
            Sym::LET => assert_eq!(k, Keyword::Let),
            Sym::NEW => assert_eq!(k, Keyword::New),
            Sym::NULL => assert_eq!(k, Keyword::Null),
            Sym::OF => assert_eq!(k, Keyword::Of),
            Sym::RETURN => assert_eq!(k, Keyword::Return),
            Sym::SUPER => assert_eq!(k, Keyword::Super),
            Sym::SWITCH => assert_eq!(k, Keyword::Switch),
            Sym::THIS => assert_eq!(k, Keyword::This),
            Sym::THROW => assert_eq!(k, Keyword::Throw),
            Sym::TRUE => assert_eq!(k, Keyword::True),
            Sym::TRY => assert_eq!(k, Keyword::Try),
            Sym::TYPEOF => assert_eq!(k, Keyword::TypeOf),
            Sym::VAR => assert_eq!(k, Keyword::Var),
            Sym::VOID => assert_eq!(k, Keyword::Void),
            Sym::WHILE => assert_eq!(k, Keyword::While),
            Sym::WITH => assert_eq!(k, Keyword::With),
            Sym::YIELD => assert_eq!(k, Keyword::Yield),
            _ => unreachable!("unknown keyword {k:?} found"),
        }
    }
}

#[test]
fn try_into_binary_op() {
    for k in all_keywords() {
        match k {
            Keyword::InstanceOf | Keyword::In => assert!(BinaryOp::try_from(k).is_ok()),
            Keyword::Await
            | Keyword::Async
            | Keyword::Break
            | Keyword::Case
            | Keyword::Catch
            | Keyword::Class
            | Keyword::Continue
            | Keyword::Const
            | Keyword::Debugger
            | Keyword::Default
            | Keyword::Delete
            | Keyword::Do
            | Keyword::Else
            | Keyword::Enum
            | Keyword::Export
            | Keyword::Extends
            | Keyword::False
            | Keyword::Finally
            | Keyword::For
            | Keyword::Function
            | Keyword::If
            | Keyword::Import
            | Keyword::Let
            | Keyword::New
            | Keyword::Null
            | Keyword::Of
            | Keyword::Return
            | Keyword::Super
            | Keyword::Switch
            | Keyword::This
            | Keyword::Throw
            | Keyword::True
            | Keyword::Try
            | Keyword::TypeOf
            | Keyword::Var
            | Keyword::Void
            | Keyword::While
            | Keyword::With
            | Keyword::Yield => assert!(BinaryOp::try_from(k).is_err()),
        }
    }
}

#[test]
fn from_str() {
    for k in all_keywords() {
        let str = k.as_str();
        assert_eq!(str.parse::<Keyword>().unwrap(), k);
    }

    for invalid in ["", "testing", "invalid keyword"] {
        let result = invalid.parse::<Keyword>();
        let error = result.unwrap_err();

        assert_eq!(error.to_string(), "invalid token");
    }
}
