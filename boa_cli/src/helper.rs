use colored::{Color, Colorize};
use phf::{phf_set, Set};
use regex::{Captures, Regex, Replacer};
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

#[allow(clippy::upper_case_acronyms, clippy::redundant_pub_crate)]
#[derive(Completer, Helper, Hinter)]
pub(crate) struct RLHelper {
    highlighter: LineHighlighter,
    validator: MatchingBracketValidator,
    colored_prompt: String,
}

impl RLHelper {
    pub(crate) fn new(prompt: &str) -> Self {
        Self {
            highlighter: LineHighlighter::new(),
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

struct LineHighlighter {
    regex: Regex,
}

impl LineHighlighter {
    fn new() -> Self {
        // Precompiles the regex to avoid creating it again after every highlight
        let regex = Regex::new(
            r#"(?x)
            (?P<identifier>\b[$_\p{ID_Start}][$_\p{ID_Continue}\u{200C}\u{200D}]*\b) |
            (?P<string_double_quote>"([^"\\]|\\.)*") |
            (?P<string_single_quote>'([^'\\]|\\.)*') |
            (?P<template_literal>`([^`\\]|\\.)*`) |
            (?P<op>[+\-/*%~^!&|=<>;:]) |
            (?P<number>0[bB][01](_?[01])*n?|0[oO][0-7](_?[0-7])*n?|0[xX][0-9a-fA-F](_?[0-9a-fA-F])*n?|(([0-9](_?[0-9])*\.([0-9](_?[0-9])*)?)|(([0-9](_?[0-9])*)?\.[0-9](_?[0-9])*)|([0-9](_?[0-9])*))([eE][+-]?[0-9](_?[0-9])*)?n?)"#,
        ).expect("could not compile regular expression");

        Self { regex }
    }
}

impl Highlighter for LineHighlighter {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        use std::fmt::Write;

        struct Colorizer;

        impl Replacer for Colorizer {
            // Changing to map_or_else moves the handling of "identifier" after all other kinds,
            // which reads worse than this version.
            #[allow(clippy::option_if_let_else)]
            fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
                let colored = if let Some(cap) = caps.name("identifier") {
                    let cap = cap.as_str();

                    let colored = match cap {
                        "true" | "false" | "null" | "Infinity" | "globalThis" => {
                            cap.color(PROPERTY_COLOR)
                        }
                        "undefined" => cap.color(UNDEFINED_COLOR),
                        identifier if KEYWORDS.contains(identifier) => {
                            cap.color(KEYWORD_COLOR).bold()
                        }
                        _ => cap.color(IDENTIFIER_COLOR),
                    };

                    Some(colored)
                } else if let Some(cap) = caps
                    .name("string_double_quote")
                    .or_else(|| caps.name("string_single_quote"))
                    .or_else(|| caps.name("template_literal"))
                {
                    Some(cap.as_str().color(STRING_COLOR))
                } else if let Some(cap) = caps.name("op") {
                    Some(cap.as_str().color(OPERATOR_COLOR))
                } else {
                    caps.name("number")
                        .map(|cap| cap.as_str().color(NUMBER_COLOR))
                };

                if let Some(colored) = colored {
                    write!(dst, "{colored}").expect("could not append data to dst");
                } else {
                    dst.push_str(&caps[0]);
                }
            }
        }
        self.regex.replace_all(line, Colorizer)
    }
}
