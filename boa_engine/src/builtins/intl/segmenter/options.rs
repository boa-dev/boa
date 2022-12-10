#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum Granularity {
    #[default]
    Grapheme,
    Word,
    Sentence,
}

#[derive(Debug)]
pub(crate) struct ParseGranularityError;

impl std::fmt::Display for ParseGranularityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "provided string was not `grapheme`, `word` or `sentence`".fmt(f)
    }
}

impl std::str::FromStr for Granularity {
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
