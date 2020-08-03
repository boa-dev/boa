use colored::*;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use rustyline::{
    error::ReadlineError,
    highlight::Highlighter,
    validate::{MatchingBracketValidator, ValidationContext, ValidationResult, Validator},
};
use rustyline_derive::{Completer, Helper, Hinter};
use std::borrow::Cow;
use std::collections::HashSet;

const STRING_COLOR: Color = Color::Green;
const KEYWORD_COLOR: Color = Color::Yellow;
const PROPERTY_COLOR: Color = Color::Magenta;
const OPERATOR_COLOR: Color = Color::TrueColor {
    r: 214,
    g: 95,
    b: 26,
};
const UNDEFINED_COLOR: Color = Color::TrueColor {
    r: 100,
    g: 100,
    b: 100,
};

#[derive(Completer, Helper, Hinter)]
pub(crate) struct RLHelper {
    highlighter: LineHighlighter,
    validator: MatchingBracketValidator,
}

impl RLHelper {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            highlighter: LineHighlighter,
            validator: MatchingBracketValidator::new(),
        }
    }
}

impl Validator for RLHelper {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> Result<ValidationResult, ReadlineError> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Highlighter for RLHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        hint.into()
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        self.highlighter.highlight(candidate, 0)
    }

    fn highlight_char(&self, line: &str, _: usize) -> bool {
        !line.is_empty()
    }
}

lazy_static! {
    static ref KEYWORDS: HashSet<&'static str> = {
        let mut keywords = HashSet::new();
        keywords.insert("break");
        keywords.insert("case");
        keywords.insert("catch");
        keywords.insert("class");
        keywords.insert("const");
        keywords.insert("continue");
        keywords.insert("default");
        keywords.insert("delete");
        keywords.insert("do");
        keywords.insert("else");
        keywords.insert("export");
        keywords.insert("extends");
        keywords.insert("finally");
        keywords.insert("for");
        keywords.insert("function");
        keywords.insert("if");
        keywords.insert("import");
        keywords.insert("instanceof");
        keywords.insert("new");
        keywords.insert("return");
        keywords.insert("super");
        keywords.insert("switch");
        keywords.insert("this");
        keywords.insert("throw");
        keywords.insert("try");
        keywords.insert("typeof");
        keywords.insert("var");
        keywords.insert("void");
        keywords.insert("while");
        keywords.insert("with");
        keywords.insert("yield");
        keywords.insert("await");
        keywords.insert("enum");
        keywords.insert("let");
        keywords
    };
}

struct LineHighlighter;

impl Highlighter for LineHighlighter {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        let mut coloured = line.to_string();

        let reg = Regex::new(
            r#"(?x)
            (?P<identifier>[$A-z_]+[$A-z_0-9]*) |
            (?P<string_double_quote>"([^"\\]|\\.)*") |
            (?P<string_single_quote>'([^'\\]|\\.)*') |
            (?P<op>[+\-/*%~^!&|=<>;:])"#,
        )
        .unwrap();

        coloured = reg
            .replace_all(&coloured, |caps: &Captures<'_>| {
                if let Some(cap) = caps.name("identifier") {
                    match cap.as_str() {
                        "true" | "false" | "null" | "Infinity" | "globalThis" => {
                            cap.as_str().color(PROPERTY_COLOR).to_string()
                        }
                        "undefined" => cap.as_str().color(UNDEFINED_COLOR).to_string(),
                        identifier if KEYWORDS.contains(identifier) => {
                            cap.as_str().color(KEYWORD_COLOR).bold().to_string()
                        }
                        _ => cap.as_str().to_string(),
                    }
                } else if let Some(cap) = caps.name("string_double_quote") {
                    cap.as_str().color(STRING_COLOR).to_string()
                } else if let Some(cap) = caps.name("string_single_quote") {
                    cap.as_str().color(STRING_COLOR).to_string()
                } else if let Some(cap) = caps.name("op") {
                    cap.as_str().color(OPERATOR_COLOR).to_string()
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();

        coloured.into()
    }
}
