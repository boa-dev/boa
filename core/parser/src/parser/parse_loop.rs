use boa_ast::Position;
use boa_interner::Interner;

use boa_ast as ast;
use crate::error::{ParseResult, Error};
use crate::lexer::Token;
use crate::source::ReadChar;
use crate::parser::Cursor;
use crate::parser::statement::{StatementList, StatementListLocal, StatementListNode};

#[macro_export]
macro_rules! parse_cmd {
    // or move into `pop_local_state!`
    [[POP LOCAL]: $state:ident => $variant:ident] => {{
        let Ok($crate::parser::SavedState::$variant(ret)) = $state.pop_local_state() else {
            return Err($state.general_error(concat!("expect `", stringify!($variant) ,"Local` saved state")))
        };
        ret
    }};

    // or move into `pop_last_node!`
    [[POP NODE]: $state:ident => $variant:ident] => {{
        let Ok($crate::parser::ParsedNode::$variant(ret)) = $state.pop_node() else {
            return Err($state.general_error(concat!("expect `", stringify!($variant) ,"Local` saved state")))
        };
        ret
    }};

    // or move into `sub_parse!`
    [[SUB PARSE]: $item:expr; $state:ident <= $local:ident as $variant:ident ($point:literal)] => {{
        $state.push_local($crate::parser::SavedState::$variant($local));
        return ParseResult::Ok($crate::parser::ControlFlow::SubParse { node: Box::new($item), point: $point });
    }};

    // or move into `parse_done!`
    [[DONE]: $state:ident <= $variant:ident($node:expr)] => {{
        $state.push_node($crate::parser::ParsedNode::$variant($node));
        return Ok($crate::parser::ControlFlow::Done)
    }};
}

pub(super) struct ParseLoop;

impl ParseLoop {
    pub(super) fn parse_loop<R: ReadChar>(
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
        entry: StatementList
    ) -> ParseResult<StatementListNode> {
        let mut state: ParseState<'_, R> = ParseState::new(cursor, interner);

        let mut parse_stack: Vec<Box<dyn TokenLoopParser<R>>> = vec![Box::new(entry)];
        let mut continue_points = vec![0];

        loop {
            debug_assert!(!parse_stack.is_empty());
            debug_assert_eq!(continue_points.len(), parse_stack.len());

            let continue_point = continue_points.pop().unwrap();
            let parser = parse_stack.last_mut().unwrap();

            match parser.parse_loop(&mut state, continue_point)? {
                ControlFlow::SubParse { node, point } => {
                    continue_points.push(point); // reinsert current updated `continue_point`
                    continue_points.push(0); // insert continue point for new sub parsing node

                    parse_stack.push(node);
                }
                ControlFlow::Done => {
                    // remove parsing node from stack (`continue_point` already removed)
                    parse_stack.pop();

                    if parse_stack.is_empty() {
                        let stmt_list_node = parse_cmd![[POP NODE]: state => StatementList];
                        assert!(state.nodes.is_empty());
                        return Ok(stmt_list_node)
                    }
                }
            }

        }
    }
}

/// Trait implemented by parsers.
///
/// This makes it possible to abstract over the underlying implementation of a parser.
pub(super) trait TokenLoopParser<R>
where
    R: ReadChar,
{
    /// Parses the token stream using the current parser.
    ///
    /// This method needs to be provided by the implementor type.
    ///
    /// # Errors
    ///
    /// It will fail if the cursor is not placed at the beginning of the expected non-terminal.
    fn parse_loop(&mut self, state: &mut ParseState<'_, R>, continue_point: usize) -> ParseResult<ControlFlow<R>>;
}

pub(super) enum ControlFlow<R>
where R: ReadChar,
{
    SubParse{node: Box<dyn TokenLoopParser<R>>, point: usize},
    Done,
}

pub(super) struct ParseState<'a, R> {
    nodes: Vec<ParsedNode>,
    saved_state: Vec<SavedState>,
    cursor: &'a mut Cursor<R>,
    interner: &'a mut Interner,
}
impl<'a, R: ReadChar> ParseState<'a, R> {
    pub(super) fn new(cursor: &'a mut Cursor<R>, interner: &'a mut Interner) -> Self {
        Self {
            nodes: Vec::new(),
            saved_state: Vec::new(),
            cursor,
            interner,
        }
    }
    pub(super) fn mut_inner(&mut self) -> (&mut Cursor<R>, &mut Interner) {
        (&mut self.cursor, &mut self.interner)
    }

    pub(super) fn cursor(&mut self) -> &Cursor<R> {
        &self.cursor
    }
    pub(super) fn cursor_mut(&mut self) -> &mut Cursor<R> {
        &mut self.cursor
    }
    pub(super) fn interner(&self) -> &Interner {
        &self.interner
    }
    pub(super) fn interner_mut(&mut self) -> &mut Interner {
        &mut self.interner
    }

    pub(super) fn push_node(&mut self, node: ParsedNode) {
        self.nodes.push(node);
    }
    pub(super) fn push_local(&mut self, local: SavedState) {
        self.saved_state.push(local);
    }

    pub(super) fn pop_node(&mut self) -> ParseResult<ParsedNode> {
        self.nodes.pop().ok_or_else(||self.general_error("expect parsed node"))
    }

    pub(super) fn pop_local_state(&mut self) -> ParseResult<SavedState> {
        self.saved_state.pop().ok_or_else(||self.general_error("expect saved state"))
    }

    pub(super) fn continue_point_error<T>(&self, continue_point: usize) -> ParseResult<T> {
        Err(self.general_error(format!("unexpected continue point ({continue_point})")))
    }

    pub(super) fn general_error<S: AsRef<str>>(&self, msg: S) -> Error {
        Error::general(
            format!("{}; linear position: {}", msg.as_ref(), self.cursor.linear_pos()),
            Position::new(1, 1) // TODO: something to take last position see `self.cursor.linear_pos()`
        )
    }

    ///Peeks a future token, without consuming it or advancing the cursor. This peeking skips line terminators.
    ///
    /// You can skip some tokens with the `skip_n` option.
    pub(super) fn peek(&mut self, skip_n: usize) -> ParseResult<Option<&Token>> {
        self.cursor.peek(skip_n, &mut self.interner)
    }
}

pub(super) enum ParsedNode {
    Empty,
    StatementListItem(ast::StatementListItem),
    StatementList(StatementListNode),
}

pub(super) enum SavedState {
    StatementList(StatementListLocal),
}
