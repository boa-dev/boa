use std::io::Read;

use crate::{
    syntax::{
        ast::{Keyword, Node, Punctuator},
        lexer::TokenKind,
        parser::{
            cursor::Cursor,
            error::ParseError,
            statement::{declaration::hoistable::FunctionDeclaration, Statement},
            TokenParser,
        },
    },
    BoaProfiler,
};

use super::BindingIdentifier;
/// Labelled Statement Parsing
///
/// More information
/// - [MDN documentation][mdn]
/// - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/label
/// [spec]: https://tc39.es/ecma262/#sec-labelled-statements
#[derive(Debug, Clone, Copy)]
pub(super) struct LabelledStatement<const YIELD: bool, const AWAIT: bool, const RETURN: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for LabelledStatement<YIELD, AWAIT, RETURN>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Label", "Parsing");

        let name = BindingIdentifier::<YIELD, AWAIT>.parse(cursor)?;

        cursor.expect(Punctuator::Colon, "Labelled Statement")?;

        let strict = cursor.strict_mode();
        let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
        let mut node = match next_token.kind() {
            // Early Error: It is a Syntax Error if any strict mode source code matches this rule.
            // https://tc39.es/ecma262/#sec-labelled-statements-static-semantics-early-errors
            // https://tc39.es/ecma262/#sec-labelled-function-declarations
            TokenKind::Keyword(Keyword::Function) if strict => {
                return Err(ParseError::general(
                    "In strict mode code, functions can only be declared at top level or inside a block.",
                    next_token.span().start()
                ))
            }
            TokenKind::Keyword(Keyword::Function) => {
                FunctionDeclaration::<YIELD, AWAIT, false>
                .parse(cursor)?
                .into()
            }
            _ => Statement::<YIELD, AWAIT, RETURN>.parse(cursor)?
        };

        set_label_for_node(&mut node, name);
        Ok(node)
    }
}

fn set_label_for_node(node: &mut Node, name: Box<str>) {
    match node {
        Node::ForLoop(ref mut for_loop) => for_loop.set_label(name),
        Node::ForOfLoop(ref mut for_of_loop) => for_of_loop.set_label(name),
        Node::ForInLoop(ref mut for_in_loop) => for_in_loop.set_label(name),
        Node::DoWhileLoop(ref mut do_while_loop) => do_while_loop.set_label(name),
        Node::WhileLoop(ref mut while_loop) => while_loop.set_label(name),
        _ => (),
    }
}
