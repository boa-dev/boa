//! Data extraction helpers for `console.table()`.
//!
//! This module converts a JS value into [`TableData`] that the [`super::Logger`]
//! backend can render however it likes (terminal box-drawing, HTML, etc.).

use boa_engine::builtins::object::OrdinaryObject;
use boa_engine::object::builtins::{JsMap, JsSet};
use boa_engine::{Context, JsError, JsResult, JsValue, js_string, object::JsObject};
use rustc_hash::{FxHashMap, FxHashSet};

/// The column name used for row indices.
const INDEX_COL: &str = "(index)";
/// The column name used for iteration indices (Map/Set).
const ITER_INDEX_COL: &str = "(iteration index)";
/// The column name used for primitive (non-object) values.
const VALUE_COL: &str = "Values";
/// The column name used for Map keys.
const KEY_COL: &str = "Key";

/// Structured data for `console.table()`, passed to the [`super::Logger`] so
/// each backend can render it in the most appropriate way.
#[derive(Debug, Clone)]
pub struct TableData {
    /// Column headers, always starting with `"(index)"` or `"(iteration index)"`.
    pub col_names: Vec<String>,
    /// Each row is a map from column name to cell value.
    pub rows: Vec<FxHashMap<String, String>>,
}

/// Try to build [`TableData`] from the first argument to `console.table()`.
///
/// Returns `Ok(None)` when the data is not tabular (primitive, or empty
/// object/array) so the caller can fall back to `console.log`.
pub(super) fn build_table_data(
    obj: &JsObject,
    properties: Option<&JsObject>,
    context: &mut Context,
) -> JsResult<Option<TableData>> {
    // Map/Set have a fixed column layout and ignore the `properties` filter,
    // matching Node.js behaviour.
    let (mut data, is_collection) = if let Ok(map) = JsMap::from_object(obj.clone()) {
        (extract_map_rows(&map)?, true)
    } else if let Ok(set) = JsSet::from_object(obj.clone()) {
        (extract_set_rows(&set)?, true)
    } else {
        (extract_rows(obj, context)?, false)
    };

    if data.rows.is_empty() {
        return Ok(None);
    }

    // Only apply the properties filter to plain objects/arrays, not Map/Set.
    if !is_collection && let Some(props) = properties {
        data.col_names = filter_columns(&data.col_names, props, context)?;
    }

    Ok(Some(data))
}

/// Extracts rows from a `Map`, using `(iteration index)`, `Key`, and `Values`
/// columns to match Node.js/Chrome behaviour.
fn extract_map_rows(map: &JsMap) -> JsResult<TableData> {
    let col_names = vec![
        ITER_INDEX_COL.to_string(),
        KEY_COL.to_string(),
        VALUE_COL.to_string(),
    ];
    let mut rows = Vec::new();
    let mut index = 0usize;

    map.for_each_native(|key, value| {
        let mut row = FxHashMap::default();
        row.insert(ITER_INDEX_COL.to_string(), index.to_string());
        row.insert(KEY_COL.to_string(), display_cell_value(&key));
        row.insert(VALUE_COL.to_string(), display_cell_value(&value));
        rows.push(row);
        index += 1;
        Ok(())
    })?;

    Ok(TableData { col_names, rows })
}

/// Extracts rows from a `Set`, using `(iteration index)` and `Values` columns.
fn extract_set_rows(set: &JsSet) -> JsResult<TableData> {
    let col_names = vec![ITER_INDEX_COL.to_string(), VALUE_COL.to_string()];
    let mut rows = Vec::new();
    let mut index = 0usize;

    set.for_each_native(|value| {
        let mut row = FxHashMap::default();
        row.insert(ITER_INDEX_COL.to_string(), index.to_string());
        row.insert(VALUE_COL.to_string(), display_cell_value(&value));
        rows.push(row);
        index += 1;
        Ok(())
    })?;

    Ok(TableData { col_names, rows })
}

/// Extracts rows and column names from a JS object/array.
///
/// Only considers enumerable own string-keyed properties, matching
/// browser behaviour (equivalent to `Object.keys()`, e.g. excludes `length` on arrays).
fn extract_rows(obj: &JsObject, context: &mut Context) -> JsResult<TableData> {
    let keys = enumerable_keys(obj, context)?;
    let mut col_names = vec![INDEX_COL.to_string()];
    let mut seen_cols: FxHashSet<String> = FxHashSet::default();
    seen_cols.insert(INDEX_COL.to_string());
    let mut rows = Vec::new();

    for index_str in &keys {
        let val = obj.get(js_string!(index_str.as_str()), context)?;
        let mut row = FxHashMap::default();
        row.insert(INDEX_COL.to_string(), index_str.clone());

        if let Some(val_obj) = val.as_object() {
            let inner_keys = enumerable_keys(&val_obj, context)?;
            for col in &inner_keys {
                if seen_cols.insert(col.clone()) {
                    col_names.push(col.clone());
                }
                let cell = val_obj.get(js_string!(col.as_str()), context)?;
                row.insert(col.clone(), display_cell_value(&cell));
            }
        } else {
            if seen_cols.insert(VALUE_COL.to_string()) {
                col_names.push(VALUE_COL.to_string());
            }
            row.insert(VALUE_COL.to_string(), display_cell_value(&val));
        }

        rows.push(row);
    }

    Ok(TableData { col_names, rows })
}

/// Formats a JS value for display inside a table cell.
///
/// Objects and arrays are rendered on a single line (e.g. `{ nested: true }`
/// instead of multi-line pretty-print), matching Node.js/Chrome behaviour
/// for nested values in `console.table`.
fn display_cell_value(val: &JsValue) -> String {
    let raw = val.display().to_string();
    // If the display spans multiple lines, collapse to single-line.
    if raw.contains('\n') {
        raw.split('\n').map(str::trim).collect::<Vec<_>>().join(" ")
    } else {
        raw
    }
}

/// Returns the enumerable own string-keyed property names of `obj`,
/// equivalent to `Object.keys(obj)`.
fn enumerable_keys(obj: &JsObject, context: &mut Context) -> JsResult<Vec<String>> {
    let keys_val = OrdinaryObject::keys(
        &JsValue::undefined(),
        &[JsValue::from(obj.clone())],
        context,
    )?;
    let Some(keys_obj) = keys_val.as_object() else {
        return Err(JsError::from_native(
            boa_engine::JsNativeError::typ().with_message("Object.keys did not return an object"),
        ));
    };
    let length = keys_obj
        .get(js_string!("length"), context)?
        .to_length(context)?;
    let mut result = Vec::with_capacity(usize::try_from(length).unwrap_or(0));
    for i in 0..length {
        let val = keys_obj.get(i, context)?;
        result.push(val.to_string(context)?.to_std_string_escaped());
    }
    Ok(result)
}

/// Builds a column list from the `properties` array.
///
/// The returned list uses the **filter's order** (not discovery order),
/// and includes properties that don't exist in the data (they render as
/// empty cells). The index column is always first. Duplicates are ignored.
/// This matches Node.js behaviour.
fn filter_columns(
    all_cols: &[String],
    properties: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<String>> {
    let length = properties
        .get(js_string!("length"), context)?
        .to_length(context)?;

    let mut result = Vec::new();
    let mut seen = FxHashSet::default();

    // Always include the index column first.
    if let Some(idx_col) = all_cols.first() {
        result.push(idx_col.clone());
        seen.insert(idx_col.clone());
    }

    // Add columns in the order specified by the properties array.
    for i in 0..length {
        let val = properties.get(i, context)?;
        let col = val.to_string(context)?.to_std_string_escaped();
        if seen.insert(col.clone()) {
            result.push(col);
        }
    }

    Ok(result)
}
