// TODO: implement `Segmenter` when https://github.com/unicode-org/icu4x/issues/2259 closes.

use std::{fmt::Display, str::FromStr};

use boa_profiler::Profiler;
use tap::{Conv, Pipe};

use crate::{builtins::BuiltIn, object::ConstructorBuilder, Context, JsResult, JsValue};

#[derive(Debug, Clone)]
pub(crate) struct Segmenter;

#[derive(Debug, Clone, Copy, Default)]
enum Granularity {
    #[default]
    Grapheme,
    Word,
    Sentence,
}

#[derive(Debug)]
pub(super) struct ParseGranularityError;

impl Display for ParseGranularityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "provided string was not `grapheme`, `word` or `sentence`".fmt(f)
    }
}

impl FromStr for Granularity {
    type Err = ParseGranularityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grapheme" => Ok(Self::Grapheme),
            "word" => Ok(Self::Word),
            "sentence" => Ok(Self::Sentence),
            _ => Err(ParseGranularityError),
        }
    }
}

impl Display for Granularity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Granularity::Grapheme => "grapheme",
            Granularity::Word => "word",
            Granularity::Sentence => "sentence",
        }
        .fmt(f)
    }
}

impl BuiltIn for Segmenter {
    const NAME: &'static str = "Segmenter";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().segmenter().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Segmenter {
    pub(crate) const LENGTH: usize = 0;

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }
}
