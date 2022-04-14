use crate::syntax::{
    ast::{
        node::{self, Identifier},
        Keyword, Punctuator,
    },
    lexer::TokenKind,
    parser::{
        statement::{block::Block, ArrayBindingPattern, BindingIdentifier, ObjectBindingPattern},
        AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use rustc_hash::FxHashSet;
use std::io::Read;

/// Catch parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-Catch
#[derive(Debug, Clone, Copy)]
pub(super) struct Catch {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Catch {
    /// Creates a new `Catch` block parser.
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

impl<R> TokenParser<R> for Catch
where
    R: Read,
{
    type Output = node::Catch;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("Catch", "Parsing");
        cursor.expect((Keyword::Catch, false), "try statement", interner)?;
        let position = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .span()
            .start();
        let catch_param = if cursor.next_if(Punctuator::OpenParen, interner)?.is_some() {
            let catch_param =
                CatchParameter::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

            cursor.expect(Punctuator::CloseParen, "catch in try statement", interner)?;
            Some(catch_param)
        } else {
            None
        };

        // It is a Syntax Error if BoundNames of CatchParameter contains any duplicate elements.
        // https://tc39.es/ecma262/#sec-try-statement-static-semantics-early-errors
        if let Some(node::Declaration::Pattern(pattern)) = &catch_param {
            let mut set = FxHashSet::default();
            for ident in pattern.idents() {
                if !set.insert(ident) {
                    return Err(ParseError::general(
                        "duplicate catch parameter identifier",
                        position,
                    ));
                }
            }
        }

        let position = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .span()
            .start();
        let catch_block = Block::new(self.allow_yield, self.allow_await, self.allow_return)
            .parse(cursor, interner)?;

        // It is a Syntax Error if any element of the BoundNames of CatchParameter also occurs in the LexicallyDeclaredNames of Block.
        // It is a Syntax Error if any element of the BoundNames of CatchParameter also occurs in the VarDeclaredNames of Block unless CatchParameter is CatchParameter : BindingIdentifier .
        // https://tc39.es/ecma262/#sec-try-statement-static-semantics-early-errors
        // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
        let lexically_declared_names = catch_block.lexically_declared_names();
        match &catch_param {
            Some(node::Declaration::Identifier { ident, .. }) => {
                if lexically_declared_names.contains(&(ident.sym(), false)) {
                    return Err(ParseError::general(
                        "catch parameter identifier declared in catch body",
                        position,
                    ));
                }
                if lexically_declared_names.contains(&(ident.sym(), true)) {
                    return Err(ParseError::general(
                        "catch parameter identifier declared in catch body",
                        position,
                    ));
                }
            }
            Some(node::Declaration::Pattern(pattern)) => {
                let mut var_declared_names = FxHashSet::default();
                for node in catch_block.items() {
                    node.var_declared_names(&mut var_declared_names);
                }
                for ident in pattern.idents() {
                    if lexically_declared_names.contains(&(ident, false)) {
                        return Err(ParseError::general(
                            "catch parameter identifier declared in catch body",
                            position,
                        ));
                    }
                    if lexically_declared_names.contains(&(ident, true)) {
                        return Err(ParseError::general(
                            "catch parameter identifier declared in catch body",
                            position,
                        ));
                    }
                    if var_declared_names.contains(&ident) {
                        return Err(ParseError::general(
                            "catch parameter identifier declared in catch body",
                            position,
                        ));
                    }
                }
            }
            _ => {}
        }

        let catch_node = node::Catch::new::<_, node::Declaration, _>(catch_param, catch_block);
        Ok(catch_node)
    }
}

/// `CatchParameter` parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-CatchParameter
#[derive(Debug, Clone, Copy)]
pub(super) struct CatchParameter {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl CatchParameter {
    /// Creates a new `CatchParameter` parser.
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

impl<R> TokenParser<R> for CatchParameter
where
    R: Read,
{
    type Output = node::Declaration;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;

        match token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let pat = ObjectBindingPattern::new(true, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                Ok(node::Declaration::new_with_object_pattern(pat, None))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let pat = ArrayBindingPattern::new(true, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                Ok(node::Declaration::new_with_array_pattern(pat, None))
            }
            TokenKind::Identifier(_) => {
                let ident = BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)
                    .map(Identifier::new)?;
                Ok(node::Declaration::new_with_identifier(ident, None))
            }
            _ => Err(ParseError::unexpected(
                token.to_string(interner),
                token.span(),
                None,
            )),
        }
    }
}
