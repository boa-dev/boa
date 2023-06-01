use colored::{Color, Colorize};
use phf::{phf_set, Set};
use regex::{Captures, Regex};
use rustyline::{
    error::ReadlineError,
    highlight::Highlighter,
    validate::{MatchingBracketValidator, ValidationContext, ValidationResult, Validator},
    Completer, Helper, Hinter,
};
use std::borrow::Cow::{self, Borrowed};

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
const NUMBER_COLOR: Color = Color::TrueColor {
    r: 26,
    g: 214,
    b: 175,
};
const IDENTIFIER_COLOR: Color = Color::TrueColor {
    r: 26,
    g: 160,
    b: 214,
};

const READLINE_COLOR: Color = Color::Cyan;

#[allow(clippy::upper_case_acronyms)]
#[derive(Completer, Helper, Hinter)]
pub(crate) struct RLHelper {
    highlighter: LineHighlighter,
    validator: MatchingBracketValidator,
    colored_prompt: String,
}

impl RLHelper {
    pub(crate) fn new(prompt: &str) -> Self {
        Self {
            highlighter: LineHighlighter,
            validator: MatchingBracketValidator::new(),
            colored_prompt: prompt.color(READLINE_COLOR).bold().to_string(),
        }
    }
}

impl Validator for RLHelper {
    fn validate(
        &self,
        context: &mut ValidationContext<'_>,
    ) -> Result<ValidationResult, ReadlineError> {
        self.validator.validate(context)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Highlighter for RLHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    // Must match signature of Highlighter::highlight_prompt, can't elide lifetimes.
    #[allow(single_use_lifetimes)]
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        hint.into()
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

static KEYWORDS: Set<&'static str> = phf_set! {
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "instanceof",
    "new",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "await",
    "enum",
    "let",
};

struct LineHighlighter;

impl Highlighter for LineHighlighter {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        let mut coloured = line.to_string();

        let reg = Regex::new(
            r#"(?x)
            (?P<identifier>\b[$_\p{ID_Start}][$_\p{ID_Continue}\u{200C}\u{200D}]*\b) |
            (?P<string_double_quote>"([^"\\]|\\.)*") |
            (?P<string_single_quote>'([^'\\]|\\.)*') |
            (?P<template_literal>`([^`\\]|\\.)*`) |
            (?P<op>[+\-/*%~^!&|=<>;:]) |
            (?P<number>0[bB][01](_?[01])*n?|0[oO][0-7](_?[0-7])*n?|0[xX][0-9a-fA-F](_?[0-9a-fA-F])*n?|(([0-9](_?[0-9])*\.([0-9](_?[0-9])*)?)|(([0-9](_?[0-9])*)?\.[0-9](_?[0-9])*)|([0-9](_?[0-9])*))([eE][+-]?[0-9](_?[0-9])*)?n?)"#,
        )
        .expect("could not compile regular expression");

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
                        _ => cap.as_str().color(IDENTIFIER_COLOR).to_string(),
                    }
                } else if let Some(cap) = caps.name("string_double_quote") {
                    cap.as_str().color(STRING_COLOR).to_string()
                } else if let Some(cap) = caps.name("string_single_quote") {
                    cap.as_str().color(STRING_COLOR).to_string()
                } else if let Some(cap) = caps.name("template_literal") {
                    cap.as_str().color(STRING_COLOR).to_string()
                } else if let Some(cap) = caps.name("op") {
                    cap.as_str().color(OPERATOR_COLOR).to_string()
                } else if let Some(cap) = caps.name("number") {
                    cap.as_str().color(NUMBER_COLOR).to_string()
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();

        coloured.into()
    }
}
