//! Statement and declaration parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements
//! [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-statements-and-declarations

mod block;
mod break_stm;
mod continue_stm;
mod declaration;
mod expression;
mod if_stm;
mod iteration;
mod labelled_stm;
mod return_stm;
mod switch;
mod throw;
mod try_stm;
mod variable;

use self::{
    block::BlockStatement,
    break_stm::BreakStatement,
    continue_stm::ContinueStatement,
    declaration::Declaration,
    expression::ExpressionStatement,
    if_stm::IfStatement,
    iteration::{DoWhileStatement, ForStatement, WhileStatement},
    return_stm::ReturnStatement,
    switch::SwitchStatement,
    throw::ThrowStatement,
    try_stm::TryStatement,
    variable::VariableStatement,
};

use super::{AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, TokenParser};

use crate::{
    syntax::{
        ast::{node, Keyword, Node, Punctuator},
        lexer::{Error as LexError, InputElement, Position, TokenKind},
        parser::expression::await_expr::AwaitExpression,
    },
    BoaProfiler,
};
use labelled_stm::LabelledStatement;

use std::collections::HashSet;
use std::io::Read;

/// Statement parsing.
///
/// This can be one of the following:
///
///  - `BlockStatement`
///  - `VariableStatement`
///  - `EmptyStatement`
///  - `ExpressionStatement`
///  - `IfStatement`
///  - `BreakableStatement`
///  - `ContinueStatement`
///  - `BreakStatement`
///  - `ReturnStatement`
///  - `WithStatement`
///  - `LabelledStatement`
///  - `ThrowStatement`
///  - `SwitchStatement`
///  - `TryStatement`
///  - `DebuggerStatement`
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements
/// [spec]: https://tc39.es/ecma262/#prod-Statement
#[derive(Debug, Clone, Copy)]
pub(super) struct Statement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Statement {
    /// Creates a new `Statement` parser.
    pub(super) fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
        }
    }
}

impl<R> TokenParser<R> for Statement
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Statement", "Parsing");
        // TODO: add BreakableStatement and divide Whiles, fors and so on to another place.
        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Await) => AwaitExpression::new(self.allow_yield)
                .parse(cursor)
                .map(Node::from),
            TokenKind::Keyword(Keyword::If) => {
                IfStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::Var) => {
                VariableStatement::new(self.allow_yield, self.allow_await)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::While) => {
                WhileStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::Do) => {
                DoWhileStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::For) => {
                ForStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::Return) => {
                if self.allow_return.0 {
                    ReturnStatement::new(self.allow_yield, self.allow_await)
                        .parse(cursor)
                        .map(Node::from)
                } else {
                    Err(ParseError::unexpected(tok.clone(), "statement"))
                }
            }
            TokenKind::Keyword(Keyword::Break) => {
                BreakStatement::new(self.allow_yield, self.allow_await)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::Continue) => {
                ContinueStatement::new(self.allow_yield, self.allow_await)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::Try) => {
                TryStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::Throw) => {
                ThrowStatement::new(self.allow_yield, self.allow_await)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::Switch) => {
                SwitchStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                BlockStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Punctuator(Punctuator::Semicolon) => {
                // parse the EmptyStatement
                cursor.next().expect("semicolon disappeared");
                Ok(Node::Empty)
            }
            TokenKind::Identifier(_) => {
                // Labelled Statement check
                cursor.set_goal(InputElement::Div);
                let tok = cursor.peek(1)?;
                if tok.is_some()
                    && matches!(
                        tok.unwrap().kind(),
                        TokenKind::Punctuator(Punctuator::Colon)
                    )
                {
                    return LabelledStatement::new(
                        self.allow_yield,
                        self.allow_await,
                        self.allow_return,
                    )
                    .parse(cursor)
                    .map(Node::from);
                }

                ExpressionStatement::new(self.allow_yield, self.allow_await).parse(cursor)
            }

            _ => ExpressionStatement::new(self.allow_yield, self.allow_await).parse(cursor),
        }
    }
}

/// Reads a list of statements.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-StatementList
#[derive(Debug, Clone, Copy)]
pub(super) struct StatementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
    in_block: bool,
    break_nodes: &'static [TokenKind],
}

impl StatementList {
    /// Creates a new `StatementList` parser.
    pub(super) fn new<Y, A, R>(
        allow_yield: Y,
        allow_await: A,
        allow_return: R,
        in_block: bool,
        break_nodes: &'static [TokenKind],
    ) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
            in_block,
            break_nodes,
        }
    }
}

impl<R> TokenParser<R> for StatementList
where
    R: Read,
{
    type Output = node::StatementList;

    /// The function parses a node::StatementList using the StatementList's
    /// break_nodes to know when to terminate.
    ///
    /// Returns a ParseError::AbruptEnd if end of stream is reached before a
    /// break token.
    ///
    /// Returns a ParseError::unexpected if an unexpected token is found.
    ///
    /// Note that the last token which causes the parse to finish is not
    /// consumed.
    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("StatementList", "Parsing");
        let mut items = Vec::new();

        loop {
            match cursor.peek(0)? {
                Some(token) if self.break_nodes.contains(token.kind()) => break,
                None => break,
                _ => {}
            }

            let item = StatementListItem::new(
                self.allow_yield,
                self.allow_await,
                self.allow_return,
                self.in_block,
            )
            .parse(cursor)?;
            items.push(item);

            // move the cursor forward for any consecutive semicolon.
            while cursor.next_if(Punctuator::Semicolon)?.is_some() {}
        }

        // Handle any redeclarations
        // https://tc39.es/ecma262/#sec-block-static-semantics-early-errors
        {
            let mut lexically_declared_names = HashSet::new();
            let mut var_declared_names = HashSet::new();

            // TODO: Use more helpful positions in errors when spans are added to Nodes
            for item in &items {
                match item {
                    Node::LetDeclList(decl_list) | Node::ConstDeclList(decl_list) => {
                        for decl in decl_list.as_ref() {
                            // if name in VarDeclaredNames or can't be added to
                            // LexicallyDeclaredNames, raise an error
                            if var_declared_names.contains(&decl.name())
                                || !lexically_declared_names.insert(decl.name())
                            {
                                return Err(ParseError::lex(LexError::Syntax(
                                    format!("Redeclaration of variable `{}`", decl.name()).into(),
                                    match cursor.peek(0)? {
                                        Some(token) => token.span().end(),
                                        None => Position::new(1, 1),
                                    },
                                )));
                            }
                        }
                    }
                    Node::VarDeclList(decl_list) => {
                        for decl in decl_list.as_ref() {
                            // if name in LexicallyDeclaredNames, raise an error
                            if lexically_declared_names.contains(&decl.name()) {
                                return Err(ParseError::lex(LexError::Syntax(
                                    format!("Redeclaration of variable `{}`", decl.name()).into(),
                                    match cursor.peek(0)? {
                                        Some(token) => token.span().end(),
                                        None => Position::new(1, 1),
                                    },
                                )));
                            }
                            // otherwise, add to VarDeclaredNames
                            var_declared_names.insert(decl.name());
                        }
                    }
                    _ => (),
                }
            }
        }

        items.sort_by(Node::hoistable_order);

        Ok(items.into())
    }
}

/// Statement list item parsing
///
/// A statement list item can either be an statement or a declaration.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements
/// [spec]: https://tc39.es/ecma262/#prod-StatementListItem
#[derive(Debug, Clone, Copy)]
struct StatementListItem {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
    in_block: bool,
}

impl StatementListItem {
    /// Creates a new `StatementListItem` parser.
    fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R, in_block: bool) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
            in_block,
        }
    }
}

impl<R> TokenParser<R> for StatementListItem
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("StatementListItem", "Parsing");
        let strict_mode = cursor.strict_mode();
        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match *tok.kind() {
            TokenKind::Keyword(Keyword::Function) | TokenKind::Keyword(Keyword::Async) => {
                if strict_mode && self.in_block {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Function declaration in blocks not allowed in strict mode".into(),
                        tok.span().start(),
                    )));
                }
                Declaration::new(self.allow_yield, self.allow_await, true).parse(cursor)
            }
            TokenKind::Keyword(Keyword::Const) | TokenKind::Keyword(Keyword::Let) => {
                Declaration::new(self.allow_yield, self.allow_await, true).parse(cursor)
            }
            _ => {
                Statement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)
            }
        }
    }
}

/// Label identifier parsing.
///
/// This seems to be the same as a `BindingIdentifier`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-LabelIdentifier
pub(super) type LabelIdentifier = BindingIdentifier;

/// Binding identifier parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-BindingIdentifier
#[derive(Debug, Clone, Copy)]
pub(super) struct BindingIdentifier {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BindingIdentifier {
    /// Creates a new `BindingIdentifier` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for BindingIdentifier
where
    R: Read,
{
    type Output = Box<str>;

    /// Strict mode parsing as per <https://tc39.es/ecma262/#sec-identifiers-static-semantics-early-errors>.
    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("BindingIdentifier", "Parsing");

        let next_token = cursor.next()?.ok_or(ParseError::AbruptEnd)?;

        match next_token.kind() {
            TokenKind::Identifier(ref s) => Ok(s.clone()),
            TokenKind::Keyword(k @ Keyword::Yield) if !self.allow_yield.0 => {
                if cursor.strict_mode() {
                    Err(ParseError::lex(LexError::Syntax(
                        "yield keyword in binding identifier not allowed in strict mode".into(),
                        next_token.span().start(),
                    )))
                } else {
                    Ok(k.as_str().into())
                }
            }
            TokenKind::Keyword(k @ Keyword::Await) if !self.allow_await.0 => {
                if cursor.strict_mode() {
                    Err(ParseError::lex(LexError::Syntax(
                        "await keyword in binding identifier not allowed in strict mode".into(),
                        next_token.span().start(),
                    )))
                } else {
                    Ok(k.as_str().into())
                }
            }
            _ => Err(ParseError::expected(
                vec![TokenKind::identifier("identifier")],
                next_token,
                "binding identifier",
            )),
        }
    }
}
