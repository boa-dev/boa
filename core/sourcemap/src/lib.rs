//! Source map (ECMA-426) decoding and query utilities.
//!
//! This crate implements:
//! - Decoding regular and index source maps.
//! - Decoding base64 VLQ mapping segments.
//! - Extracting `sourceMappingURL` from JavaScript source.
//! - Looking up original positions from generated positions.

#![allow(clippy::module_name_repetitions)]

use serde_json::{Map, Value};
use thiserror::Error;
use url::Url;

const INVALID_B64: u8 = u8::MAX;
const B64_LUT: [u8; 128] = build_base64_lookup();
const SOURCE_MAPPING_URL_PREFIX: &str = "sourceMappingURL=";

const fn build_base64_lookup() -> [u8; 128] {
    let mut lut = [INVALID_B64; 128];

    let mut i = 0;
    while i < 26 {
        lut[(b'A' + i) as usize] = i;
        i += 1;
    }

    i = 0;
    while i < 26 {
        lut[(b'a' + i) as usize] = 26 + i;
        i += 1;
    }

    i = 0;
    while i < 10 {
        lut[(b'0' + i) as usize] = 52 + i;
        i += 1;
    }

    lut[b'+' as usize] = 62;
    lut[b'/' as usize] = 63;

    lut
}

/// Errors that can occur while decoding a source map document.
#[derive(Debug, Error)]
pub enum SourceMapError {
    /// The input string was not valid JSON.
    #[error("invalid source map JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    /// The parsed JSON root must be an object.
    #[error("source map root must be a JSON object")]
    RootNotObject,
    /// The source map must contain a string `mappings` field.
    #[error("source map `mappings` must be a string")]
    InvalidMappingsField,
    /// The source map must contain an array `sources` field.
    #[error("source map `sources` must be a JSON array")]
    InvalidSourcesField,
    /// The index source map must contain an array `sections` field.
    #[error("index source map `sections` must be a JSON array")]
    InvalidSectionsField,
    /// A section offset must be an object.
    #[error("index source map section `offset` must be a JSON object")]
    InvalidSectionOffset,
    /// A section map must be an object.
    #[error("index source map section `map` must be a JSON object")]
    InvalidSectionMap,
}

/// A generated or source position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// Zero-based line index.
    pub line: u32,
    /// Zero-based column index.
    pub column: u32,
}

/// A decoded source record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedSource {
    /// Resolved source URL.
    pub url: Option<Url>,
    /// Optional source file content.
    pub content: Option<Box<str>>,
    /// Whether this source is marked as ignored / third-party code.
    pub ignored: bool,
}

/// A decoded original position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalPosition {
    /// Index into [`DecodedSourceMap::sources`].
    pub source_index: usize,
    /// Zero-based line index in the original source.
    pub line: u32,
    /// Zero-based column index in the original source.
    pub column: u32,
}

/// A decoded mapping entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping {
    /// Position in generated code.
    pub generated: Position,
    /// Original position (if present and valid).
    pub original: Option<OriginalPosition>,
    /// Optional index into [`DecodedSourceMap::names`].
    pub name_index: Option<usize>,
}

/// A fully decoded source map record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedSourceMap {
    /// Optional generated file name.
    pub file: Option<Box<str>>,
    /// Decoded source list.
    pub sources: Vec<DecodedSource>,
    /// Decoded mappings.
    pub mappings: Vec<Mapping>,
    names: Vec<Box<str>>,
}

/// Resolved original position with references into a decoded source map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOriginalPosition<'a> {
    /// Source index in [`DecodedSourceMap::sources`].
    pub source_index: usize,
    /// Referenced decoded source.
    pub source: &'a DecodedSource,
    /// Zero-based line index.
    pub line: u32,
    /// Zero-based column index.
    pub column: u32,
}

/// Result of querying original positions at a generated location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginalPositionMatch<'a> {
    /// Resolved original position, if available.
    pub original: Option<ResolvedOriginalPosition<'a>>,
    /// Resolved mapped name, if available.
    pub name: Option<&'a str>,
}

impl DecodedSourceMap {
    /// Returns all decoded names.
    #[must_use]
    pub fn names(&self) -> impl ExactSizeIterator<Item = &str> {
        self.names.iter().map(Box::as_ref)
    }

    /// Returns one decoded name by index.
    #[must_use]
    pub fn name(&self, index: usize) -> Option<&str> {
        self.names.get(index).map(Box::as_ref)
    }

    /// Resolves original positions for the mapping group at or before `generated`.
    ///
    /// This follows ECMA-426 `GetOriginalPositions` semantics.
    #[must_use]
    pub fn get_original_positions(&self, generated: Position) -> Vec<OriginalPositionMatch<'_>> {
        let end = self
            .mappings
            .partition_point(|mapping| mapping.generated <= generated);
        if end == 0 {
            return Vec::new();
        }

        let anchor = self.mappings[end - 1].generated;
        let start = self.mappings[..end].partition_point(|mapping| mapping.generated < anchor);
        let count = self.mappings[start..].partition_point(|mapping| mapping.generated == anchor);

        let mut output = Vec::with_capacity(count);
        for mapping in &self.mappings[start..start + count] {
            let original = mapping.original.and_then(|position| {
                let source = self.sources.get(position.source_index)?;
                Some(ResolvedOriginalPosition {
                    source_index: position.source_index,
                    source,
                    line: position.line,
                    column: position.column,
                })
            });
            let name = mapping.name_index.and_then(|index| self.name(index));

            output.push(OriginalPositionMatch { original, name });
        }

        output
    }
}

/// Parses and decodes a source map document.
///
/// # Errors
///
/// Returns [`SourceMapError`] if decoding fails.
pub fn parse_source_map(input: &str, base_url: &Url) -> Result<DecodedSourceMap, SourceMapError> {
    let value: Value = serde_json::from_str(input)?;
    parse_source_map_value(&value, base_url)
}

/// Decodes a single base64 VLQ value.
#[must_use]
pub fn decode_vlq(input: &str) -> Option<i32> {
    let bytes = input.as_bytes();
    let mut index = 0;
    let value = decode_vlq_from_bytes(bytes, &mut index)?;
    if index == bytes.len() {
        Some(value)
    } else {
        None
    }
}

/// Matches `[@#]\\s*sourceMappingURL=(\\S*?)\\s*$` against one comment payload.
#[must_use]
pub fn match_source_map_url(comment: &str) -> Option<&str> {
    let first = comment.chars().next()?;
    if first != '@' && first != '#' {
        return None;
    }

    let mut index = first.len_utf8();
    while let Some(ch) = comment[index..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
    }

    if !comment[index..].starts_with(SOURCE_MAPPING_URL_PREFIX) {
        return None;
    }
    index += SOURCE_MAPPING_URL_PREFIX.len();

    let mut end = comment.len();
    while end > index {
        let Some(ch) = comment[..end].chars().next_back() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        end -= ch.len_utf8();
    }

    let value = &comment[index..end];
    if value.chars().any(char::is_whitespace) {
        return None;
    }

    Some(value)
}

/// Extracts a JavaScript `sourceMappingURL` using the non-parsing algorithm in ECMA-426.
#[must_use]
pub fn javascript_extract_source_map_url(source: &str) -> Option<String> {
    let lines = split_js_lines(source);

    for line in lines.into_iter().rev() {
        let mut position = 0;
        while position < line.len() {
            let Some(first) = line[position..].chars().next() else {
                break;
            };

            if first == '/' {
                let second_start = position + first.len_utf8();
                let second = line[second_start..].chars().next()?;
                if second != '/' {
                    return None;
                }

                let comment_start = second_start + second.len_utf8();
                let comment = &line[comment_start..];
                if comment.contains('"') || comment.contains('\'') || comment.contains('`') {
                    return None;
                }
                if comment.contains("*/") {
                    return None;
                }

                if let Some(url) = match_source_map_url(comment) {
                    return Some(url.to_owned());
                }

                position = line.len();
            } else if first.is_whitespace() {
                position += first.len_utf8();
            } else {
                return None;
            }
        }
    }

    None
}

fn parse_source_map_value(
    value: &Value,
    base_url: &Url,
) -> Result<DecodedSourceMap, SourceMapError> {
    let object = value.as_object().ok_or(SourceMapError::RootNotObject)?;
    parse_source_map_object(object, base_url)
}

fn parse_source_map_object(
    object: &Map<String, Value>,
    base_url: &Url,
) -> Result<DecodedSourceMap, SourceMapError> {
    if object.contains_key("sections") {
        decode_index_source_map(object, base_url)
    } else {
        decode_regular_source_map(object, base_url)
    }
}

fn decode_regular_source_map(
    object: &Map<String, Value>,
    base_url: &Url,
) -> Result<DecodedSourceMap, SourceMapError> {
    let mappings_field = object
        .get("mappings")
        .and_then(Value::as_str)
        .ok_or(SourceMapError::InvalidMappingsField)?;

    if !matches!(object.get("sources"), Some(Value::Array(_))) {
        return Err(SourceMapError::InvalidSourcesField);
    }

    let file = get_optional_string(object, "file");
    let source_root = get_optional_string(object, "sourceRoot");
    let sources_field = get_optional_list_of_optional_strings(object, "sources");
    let sources_content_field = get_optional_list_of_optional_strings(object, "sourcesContent");

    let ignore_list = if object.contains_key("ignoreList") {
        get_optional_list_of_array_indexes(object, "ignoreList")
    } else {
        get_optional_list_of_array_indexes(object, "x_google_ignoreList")
    };

    let sources = decode_source_map_sources(
        base_url,
        source_root.as_deref(),
        &sources_field,
        &sources_content_field,
        &ignore_list,
    );
    let names = get_optional_list_of_strings(object, "names");
    let mappings = decode_mappings(mappings_field, names.len(), sources.len());

    Ok(DecodedSourceMap {
        file,
        sources,
        mappings,
        names,
    })
}

fn decode_index_source_map(
    object: &Map<String, Value>,
    base_url: &Url,
) -> Result<DecodedSourceMap, SourceMapError> {
    let sections = object
        .get("sections")
        .and_then(Value::as_array)
        .ok_or(SourceMapError::InvalidSectionsField)?;

    let file = get_optional_string(object, "file");
    let mut source_map = DecodedSourceMap {
        file,
        sources: Vec::new(),
        mappings: Vec::new(),
        names: Vec::new(),
    };

    let mut previous_offset: Option<Position> = None;
    let mut previous_last_mapping: Option<Position> = None;

    for section in sections {
        let Some(section_object) = section.as_object() else {
            continue;
        };

        let offset = section_object
            .get("offset")
            .and_then(Value::as_object)
            .ok_or(SourceMapError::InvalidSectionOffset)?;

        let offset_line = get_integral_u32(offset.get("line")).unwrap_or(0);
        let offset_column = get_integral_u32(offset.get("column")).unwrap_or(0);
        let offset_position = Position {
            line: offset_line,
            column: offset_column,
        };

        if previous_offset.is_some_and(|previous| offset_position < previous) {
            // optional error
        }
        if previous_last_mapping.is_some_and(|last| offset_position < last) {
            // optional error
        }

        let map_field = section_object
            .get("map")
            .and_then(Value::as_object)
            .ok_or(SourceMapError::InvalidSectionMap)?;

        let Ok(mut decoded_section) = parse_source_map_object(map_field, base_url) else {
            continue; // optional error
        };

        let mut source_remap = Vec::with_capacity(decoded_section.sources.len());
        for source in decoded_section.sources.drain(..) {
            source_remap.push(remap_or_insert(&mut source_map.sources, source));
        }

        let mut name_remap = Vec::with_capacity(decoded_section.names.len());
        for name in decoded_section.names.drain(..) {
            name_remap.push(remap_or_insert(&mut source_map.names, name));
        }

        for mut mapping in decoded_section.mappings {
            if mapping.generated.line == 0 {
                mapping.generated.column = mapping.generated.column.saturating_add(offset_column);
            }
            mapping.generated.line = mapping.generated.line.saturating_add(offset_line);

            if let Some(mut original) = mapping.original {
                if let Some(&remapped_index) = source_remap.get(original.source_index) {
                    original.source_index = remapped_index;
                    mapping.original = Some(original);
                } else {
                    mapping.original = None;
                }
            }

            if let Some(name_index) = mapping.name_index {
                mapping.name_index = name_remap.get(name_index).copied();
            }

            source_map.mappings.push(mapping);
        }

        previous_offset = Some(offset_position);
        previous_last_mapping = source_map.mappings.last().map(|mapping| mapping.generated);
    }

    sort_mappings_if_needed(&mut source_map.mappings);
    Ok(source_map)
}

fn get_optional_string(object: &Map<String, Value>, key: &str) -> Option<Box<str>> {
    object.get(key).and_then(Value::as_str).map(Into::into)
}

fn get_optional_list_of_strings(object: &Map<String, Value>, key: &str) -> Vec<Box<str>> {
    let Some(values) = object.get(key).and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut list = Vec::with_capacity(values.len());
    for item in values {
        if let Some(string) = item.as_str() {
            list.push(string.into());
        } else {
            list.push("".into());
        }
    }

    list
}

fn get_optional_list_of_optional_strings(
    object: &Map<String, Value>,
    key: &str,
) -> Vec<Option<Box<str>>> {
    let Some(values) = object.get(key).and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut list = Vec::with_capacity(values.len());
    for item in values {
        if let Some(string) = item.as_str() {
            list.push(Some(string.into()));
        } else {
            list.push(None);
        }
    }

    list
}

fn get_optional_list_of_array_indexes(object: &Map<String, Value>, key: &str) -> Vec<usize> {
    let Some(values) = object.get(key).and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut list = Vec::with_capacity(values.len());
    for item in values {
        let Some(number) = item.as_u64() else {
            continue;
        };
        let Ok(index) = usize::try_from(number) else {
            continue;
        };
        list.push(index);
    }

    list
}

fn get_integral_u32(value: Option<&Value>) -> Option<u32> {
    let number = value?.as_u64()?;
    u32::try_from(number).ok()
}

fn decode_source_map_sources(
    base_url: &Url,
    source_root: Option<&str>,
    sources: &[Option<Box<str>>],
    sources_content: &[Option<Box<str>>],
    ignore_list: &[usize],
) -> Vec<DecodedSource> {
    let mut ignored = vec![false; sources.len()];
    for &index in ignore_list {
        if let Some(flag) = ignored.get_mut(index) {
            *flag = true;
        }
    }

    let source_url_prefix = source_root.map(|root| {
        if root.ends_with('/') {
            root.to_owned()
        } else {
            let mut prefixed = String::with_capacity(root.len() + 1);
            prefixed.push_str(root);
            prefixed.push('/');
            prefixed
        }
    });

    let mut decoded_sources = Vec::with_capacity(sources.len());
    for (index, source) in sources.iter().enumerate() {
        let url = source.as_deref().and_then(|source_name| {
            if let Some(prefix) = &source_url_prefix {
                let mut candidate = String::with_capacity(prefix.len() + source_name.len());
                candidate.push_str(prefix);
                candidate.push_str(source_name);
                base_url.join(&candidate).ok()
            } else {
                base_url.join(source_name).ok()
            }
        });

        let content = sources_content.get(index).cloned().flatten();
        let ignored = ignored[index];

        decoded_sources.push(DecodedSource {
            url,
            content,
            ignored,
        });
    }

    decoded_sources
}

#[derive(Debug, Clone, Copy, Default)]
struct DecodeMappingState {
    generated_line: u32,
    generated_column: i64,
    source_index: i64,
    original_line: i64,
    original_column: i64,
    name_index: i64,
}

#[derive(Debug, Clone, Copy)]
struct SegmentValues {
    count: usize,
    values: [i32; 5],
}

fn decode_mappings(raw_mappings: &str, names_len: usize, sources_len: usize) -> Vec<Mapping> {
    let mut mappings = Vec::with_capacity(estimate_mapping_capacity(raw_mappings));
    let mut state = DecodeMappingState::default();
    let mut monotonic = true;
    let mut last_generated = None;

    let mut lines = raw_mappings.split(';').peekable();
    while let Some(line) = lines.next() {
        if !decode_line(
            line,
            &mut state,
            names_len,
            sources_len,
            &mut mappings,
            &mut monotonic,
            &mut last_generated,
        ) {
            // Parsing failed: optional error, return empty list.
            return Vec::new();
        }

        if lines.peek().is_some() {
            state.generated_line = state.generated_line.saturating_add(1);
            state.generated_column = 0;
        }
    }

    if !monotonic {
        mappings.sort_unstable_by_key(|mapping| mapping.generated);
    }

    mappings
}

fn decode_line(
    line: &str,
    state: &mut DecodeMappingState,
    names_len: usize,
    sources_len: usize,
    mappings: &mut Vec<Mapping>,
    monotonic: &mut bool,
    last_generated: &mut Option<Position>,
) -> bool {
    if line.is_empty() {
        return true;
    }

    let bytes = line.as_bytes();
    let mut segment_start = 0;

    for index in 0..=bytes.len() {
        if index != bytes.len() && bytes[index] != b',' {
            continue;
        }

        if index == segment_start {
            return false;
        }

        let segment = &line[segment_start..index];
        let Some(decoded) = decode_segment(segment) else {
            return false;
        };

        apply_segment(
            decoded,
            state,
            names_len,
            sources_len,
            mappings,
            monotonic,
            last_generated,
        );

        segment_start = index + 1;
    }

    true
}

fn decode_segment(segment: &str) -> Option<SegmentValues> {
    let bytes = segment.as_bytes();
    let mut index = 0;
    let mut count = 0;
    let mut values = [0; 5];

    while index < bytes.len() {
        if count == values.len() {
            return None;
        }

        values[count] = decode_vlq_from_bytes(bytes, &mut index)?;
        count += 1;
    }

    if matches!(count, 1 | 4 | 5) {
        Some(SegmentValues { count, values })
    } else {
        None
    }
}

fn apply_segment(
    segment: SegmentValues,
    state: &mut DecodeMappingState,
    names_len: usize,
    sources_len: usize,
    mappings: &mut Vec<Mapping>,
    monotonic: &mut bool,
    last_generated: &mut Option<Position>,
) {
    state.generated_column += i64::from(segment.values[0]);
    if !(0..=i64::from(u32::MAX)).contains(&state.generated_column) {
        return;
    }

    let generated = Position {
        line: state.generated_line,
        column: match u32::try_from(state.generated_column) {
            Ok(value) => value,
            Err(_) => return,
        },
    };

    if segment.count == 1 {
        push_mapping(
            Mapping {
                generated,
                original: None,
                name_index: None,
            },
            mappings,
            monotonic,
            last_generated,
        );
        return;
    }

    state.source_index += i64::from(segment.values[1]);
    state.original_line += i64::from(segment.values[2]);
    state.original_column += i64::from(segment.values[3]);

    let original = if state.source_index < 0
        || state.original_line < 0
        || state.original_column < 0
        || state.original_line > i64::from(u32::MAX)
        || state.original_column > i64::from(u32::MAX)
    {
        None
    } else if let Ok(source_index) = usize::try_from(state.source_index) {
        if source_index < sources_len {
            Some(OriginalPosition {
                source_index,
                line: match u32::try_from(state.original_line) {
                    Ok(value) => value,
                    Err(_) => return,
                },
                column: match u32::try_from(state.original_column) {
                    Ok(value) => value,
                    Err(_) => return,
                },
            })
        } else {
            None
        }
    } else {
        None
    };

    let mut name_index = None;
    if segment.count == 5 {
        state.name_index += i64::from(segment.values[4]);
        if let Ok(candidate) = usize::try_from(state.name_index)
            && candidate < names_len
        {
            name_index = Some(candidate);
        }
    }

    push_mapping(
        Mapping {
            generated,
            original,
            name_index,
        },
        mappings,
        monotonic,
        last_generated,
    );
}

fn push_mapping(
    mapping: Mapping,
    mappings: &mut Vec<Mapping>,
    monotonic: &mut bool,
    last_generated: &mut Option<Position>,
) {
    if last_generated.is_some_and(|last| mapping.generated < last) {
        *monotonic = false;
    }
    *last_generated = Some(mapping.generated);
    mappings.push(mapping);
}

fn sort_mappings_if_needed(mappings: &mut [Mapping]) {
    if mappings
        .windows(2)
        .any(|window| window[0].generated > window[1].generated)
    {
        mappings.sort_unstable_by_key(|mapping| mapping.generated);
    }
}

fn decode_vlq_from_bytes(bytes: &[u8], index: &mut usize) -> Option<i32> {
    let mut shift = 0u32;
    let mut unsigned = 0u64;

    loop {
        let byte = *bytes.get(*index)?;
        *index += 1;

        let decoded = u64::from(decode_base64(byte)?);
        let continuation = (decoded & 0b10_0000) != 0;
        let payload = decoded & 0b01_1111;

        unsigned = unsigned.checked_add(payload.checked_shl(shift)?)?;
        if unsigned >= (1u64 << 32) {
            return None;
        }

        if !continuation {
            break;
        }

        shift = shift.checked_add(5)?;
        if shift > 35 {
            return None;
        }
    }

    decode_vlq_signed(unsigned)
}

fn decode_base64(byte: u8) -> Option<u8> {
    if byte >= 128 {
        return None;
    }

    let decoded = B64_LUT[byte as usize];
    if decoded == INVALID_B64 {
        None
    } else {
        Some(decoded)
    }
}

fn decode_vlq_signed(unsigned: u64) -> Option<i32> {
    let negative = (unsigned & 1) == 1;
    let value = unsigned >> 1;

    if negative && value == 0 {
        return Some(i32::MIN);
    }
    if value >= (1u64 << 31) {
        return None;
    }

    let value = i32::try_from(value).ok()?;
    if negative { Some(-value) } else { Some(value) }
}

fn estimate_mapping_capacity(raw_mappings: &str) -> usize {
    raw_mappings
        .as_bytes()
        .iter()
        .filter(|&&byte| byte == b',' || byte == b';')
        .count()
        + 1
}

fn split_js_lines(source: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let bytes = source.as_bytes();

    let mut start = 0;
    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];

        if byte == b'\r' {
            parts.push(&source[start..index]);
            if bytes.get(index + 1) == Some(&b'\n') {
                index += 2;
            } else {
                index += 1;
            }
            start = index;
            continue;
        }

        if byte == b'\n' {
            parts.push(&source[start..index]);
            index += 1;
            start = index;
            continue;
        }

        // U+2028 and U+2029 in UTF-8.
        if byte == 0xE2
            && bytes.get(index + 1) == Some(&0x80)
            && matches!(bytes.get(index + 2), Some(0xA8 | 0xA9))
        {
            parts.push(&source[start..index]);
            index += 3;
            start = index;
            continue;
        }

        index += 1;
    }

    parts.push(&source[start..]);
    parts
}

fn remap_or_insert<T: PartialEq>(table: &mut Vec<T>, value: T) -> usize {
    if let Some(index) = table.iter().position(|entry| *entry == value) {
        index
    } else {
        let index = table.len();
        table.push(value);
        index
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Position, decode_vlq, javascript_extract_source_map_url, match_source_map_url,
        parse_source_map,
    };
    use url::Url;

    #[test]
    fn vlq_decoding_examples() {
        assert_eq!(decode_vlq("iB"), Some(17));
        assert_eq!(decode_vlq("V"), Some(-10));
        assert_eq!(decode_vlq("A"), Some(0));
        assert_eq!(decode_vlq("===="), None);
    }

    #[test]
    fn decode_regular_map_and_lookup() {
        let base_url = Url::parse("https://example.com/dist/out.js.map").expect("valid base URL");
        let map = parse_source_map(
            r#"{
                "version":3,
                "file":"out.js",
                "sources":["foo.js"],
                "sourcesContent":["export const x = 1;"],
                "names":["x"],
                "mappings":"AAAAA,AAAAA",
                "ignoreList":[0]
            }"#,
            &base_url,
        )
        .expect("map should decode");

        assert_eq!(map.file.as_deref(), Some("out.js"));
        assert_eq!(map.sources.len(), 1);
        assert!(map.sources[0].ignored);
        assert_eq!(
            map.sources[0]
                .url
                .as_ref()
                .expect("source should resolve")
                .as_str(),
            "https://example.com/dist/foo.js"
        );
        assert_eq!(map.mappings.len(), 2);
        assert_eq!(map.name(0), Some("x"));

        let results = map.get_original_positions(Position { line: 0, column: 0 });
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|result| result.name == Some("x")));
        assert!(results.iter().all(|result| {
            result
                .original
                .as_ref()
                .is_some_and(|p| p.line == 0 && p.column == 0)
        }));
    }

    #[test]
    fn decode_invalid_mappings_returns_empty_list() {
        let base_url = Url::parse("https://example.com/app.js.map").expect("valid base URL");
        let map = parse_source_map(
            r#"{
                "version":3,
                "sources":["app.ts"],
                "mappings":"A,"
            }"#,
            &base_url,
        )
        .expect("top-level decode should still succeed");

        assert!(map.mappings.is_empty());
    }

    #[test]
    fn decode_unsorted_mappings_are_sorted() {
        let base_url = Url::parse("https://example.com/app.js.map").expect("valid base URL");
        let map = parse_source_map(
            r#"{
                "version":3,
                "sources":["app.ts"],
                "mappings":"C,D"
            }"#,
            &base_url,
        )
        .expect("map should decode");

        assert_eq!(map.mappings.len(), 2);
        assert_eq!(map.mappings[0].generated.column, 0);
        assert_eq!(map.mappings[1].generated.column, 1);
    }

    #[test]
    fn decode_index_source_map_with_offsets() {
        let base_url = Url::parse("https://example.com/build/bundle.js.map").expect("valid URL");
        let map = parse_source_map(
            r#"{
                "version":3,
                "file":"bundle.js",
                "sections":[
                    {
                        "offset":{"line":0,"column":0},
                        "map":{
                            "version":3,
                            "sources":["a.ts"],
                            "names":["fn"],
                            "mappings":"AAAAA"
                        }
                    },
                    {
                        "offset":{"line":10,"column":5},
                        "map":{
                            "version":3,
                            "sources":["b.ts"],
                            "names":["fn"],
                            "mappings":"AAAAA"
                        }
                    }
                ]
            }"#,
            &base_url,
        )
        .expect("index map should decode");

        assert_eq!(map.sources.len(), 2);
        assert_eq!(map.mappings.len(), 2);
        assert_eq!(map.mappings[0].generated, Position { line: 0, column: 0 });
        assert_eq!(
            map.mappings[1].generated,
            Position {
                line: 10,
                column: 5
            }
        );

        let results = map.get_original_positions(Position {
            line: 10,
            column: 5,
        });
        assert_eq!(results.len(), 1);
        let resolved = results[0]
            .original
            .as_ref()
            .expect("should resolve original position");
        assert!(
            resolved
                .source
                .url
                .as_ref()
                .expect("resolved source should have URL")
                .as_str()
                .ends_with("/build/b.ts")
        );
    }

    #[test]
    fn deprecated_ignore_list_is_used_when_ignore_list_missing() {
        let base_url = Url::parse("https://example.com/app.js.map").expect("valid base URL");
        let map = parse_source_map(
            r#"{
                "version":3,
                "sources":["a.js","b.js"],
                "x_google_ignoreList":[1],
                "mappings":"AAAA;AACA"
            }"#,
            &base_url,
        )
        .expect("map should decode");

        assert!(!map.sources[0].ignored);
        assert!(map.sources[1].ignored);
    }

    #[test]
    fn source_mapping_url_matching() {
        assert_eq!(
            match_source_map_url("# sourceMappingURL=foo.js.map"),
            Some("foo.js.map")
        );
        assert_eq!(
            match_source_map_url("@sourceMappingURL=data:application/json,{}"),
            Some("data:application/json,{}")
        );
        assert_eq!(match_source_map_url(" # sourceMappingURL=a.map"), None);
        assert_eq!(match_source_map_url("# sourceMappingURL=a map"), None);
    }

    #[test]
    fn javascript_source_mapping_url_extraction() {
        let source = "let x = 1;\n//# sourceMappingURL=foo.js.map\n";
        assert_eq!(
            javascript_extract_source_map_url(source),
            Some("foo.js.map".to_string())
        );
    }

    #[test]
    fn javascript_source_mapping_url_extraction_ambiguous_returns_none() {
        let source = "let a = `\n//# sourceMappingURL=foo.js.map\n// `";
        assert_eq!(javascript_extract_source_map_url(source), None);
    }

    #[test]
    fn javascript_source_mapping_url_requires_comment_only_line() {
        let source = "const x = 1; //# sourceMappingURL=foo.js.map";
        assert_eq!(javascript_extract_source_map_url(source), None);
    }
}
