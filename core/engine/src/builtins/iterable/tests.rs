//! Tests for the Iterator Helpers proposal implementation (#4444).

use crate::{JsNativeErrorKind, JsValue, TestAction, run_test_actions};
use boa_macros::js_str;

// ── Iterator Constructor ──────────────────────────────────────────────────────

#[test]
fn iterator_constructor_requires_new() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator()",
        JsNativeErrorKind::Type,
        "Iterator constructor requires 'new'",
    )]);
}

#[test]
fn iterator_constructor_abstract_class() {
    run_test_actions([TestAction::assert_native_error(
        "new Iterator()",
        JsNativeErrorKind::Type,
        "Abstract class Iterator not directly constructable",
    )]);
}

#[test]
fn iterator_subclass_can_be_constructed() {
    run_test_actions([TestAction::assert(
        "class MyIter extends Iterator {
           next() { return {value: 42, done: false}; }
         }
         new MyIter() instanceof Iterator",
    )]);
}

#[test]
fn iterator_to_string_tag() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.prototype[Symbol.toStringTag]",
        js_str!("Iterator"),
    )]);
}

// ── Iterator.from() ───────────────────────────────────────────────────────────

#[test]
fn iterator_from_array() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1, 2, 3]).toArray().join(',')",
        js_str!("1,2,3"),
    )]);
}

#[test]
fn iterator_from_string() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from('abc').toArray().join(',')",
        js_str!("a,b,c"),
    )]);
}

#[test]
fn iterator_from_returns_object() {
    run_test_actions([TestAction::assert_eq(
        "typeof Iterator.from([1, 2, 3])",
        js_str!("object"),
    )]);
}

// ── Lazy — map ────────────────────────────────────────────────────────────────

#[test]
fn iterator_map_basic() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1, 2, 3]).map(x => x * 2).toArray().join(',')",
        js_str!("2,4,6"),
    )]);
}

#[test]
fn iterator_map_counter() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from(['a','b','c']).map((v, i) => i).toArray().join(',')",
        js_str!("0,1,2"),
    )]);
}

#[test]
fn iterator_map_mapper_not_callable_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.from([1]).map(42)",
        JsNativeErrorKind::Type,
        "Iterator.prototype.map: mapper is not callable",
    )]);
}

// ── Lazy — filter ─────────────────────────────────────────────────────────────

#[test]
fn iterator_filter_basic() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3,4,5]).filter(x => x % 2 === 0).toArray().join(',')",
        js_str!("2,4"),
    )]);
}

#[test]
fn iterator_filter_none_match() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).filter(x => x > 99).toArray().length",
        0,
    )]);
}

// ── Lazy — take ───────────────────────────────────────────────────────────────

#[test]
fn iterator_take_basic() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3,4,5]).take(3).toArray().join(',')",
        js_str!("1,2,3"),
    )]);
}

#[test]
fn iterator_take_zero() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).take(0).toArray().length",
        0,
    )]);
}

#[test]
fn iterator_take_negative_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.from([1]).take(-1)",
        JsNativeErrorKind::Range,
        "Iterator.prototype.take: limit cannot be negative",
    )]);
}

#[test]
fn iterator_take_nan_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.from([1]).take(NaN)",
        JsNativeErrorKind::Range,
        "Iterator.prototype.take: limit cannot be NaN",
    )]);
}

#[test]
fn iterator_take_more_than_length() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).take(100).toArray().join(',')",
        js_str!("1,2,3"),
    )]);
}

// ── Lazy — drop ───────────────────────────────────────────────────────────────

#[test]
fn iterator_drop_basic() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3,4,5]).drop(2).toArray().join(',')",
        js_str!("3,4,5"),
    )]);
}

#[test]
fn iterator_drop_more_than_length() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).drop(10).toArray().length",
        0,
    )]);
}

#[test]
fn iterator_drop_zero() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).drop(0).toArray().join(',')",
        js_str!("1,2,3"),
    )]);
}

// ── Lazy — flatMap ────────────────────────────────────────────────────────────

#[test]
fn iterator_flat_map_basic() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).flatMap(x => [x, x*10]).toArray().join(',')",
        js_str!("1,10,2,20,3,30"),
    )]);
}

#[test]
fn iterator_flat_map_inner_empty() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).flatMap(x => []).toArray().length",
        0,
    )]);
}

#[test]
fn iterator_flat_map_not_callable_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.from([1]).flatMap(42)",
        JsNativeErrorKind::Type,
        "Iterator.prototype.flatMap: mapper is not callable",
    )]);
}

// ── Eager — reduce ────────────────────────────────────────────────────────────

#[test]
fn iterator_reduce_with_initial_value() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3,4]).reduce((acc, x) => acc + x, 0)",
        10,
    )]);
}

#[test]
fn iterator_reduce_no_initial_value() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).reduce((acc, x) => acc + x)",
        6,
    )]);
}

#[test]
fn iterator_reduce_empty_no_initial_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.from([]).reduce((acc, x) => acc + x)",
        JsNativeErrorKind::Type,
        "Iterator.prototype.reduce: cannot reduce empty iterator with no initial value",
    )]);
}

// ── Eager — toArray ───────────────────────────────────────────────────────────

#[test]
fn iterator_to_array_basic() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(Iterator.from([1,2,3]).toArray())",
        js_str!("[1,2,3]"),
    )]);
}

#[test]
fn iterator_to_array_empty() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([]).toArray().length",
        0,
    )]);
}

// ── Eager — forEach ───────────────────────────────────────────────────────────

#[test]
fn iterator_for_each_basic() {
    run_test_actions([
        TestAction::run("let sum = 0; Iterator.from([1,2,3]).forEach(x => { sum += x; });"),
        TestAction::assert_eq("sum", 6),
    ]);
}

// ── Eager — some / every / find ───────────────────────────────────────────────

#[test]
fn iterator_some_true() {
    run_test_actions([TestAction::assert(
        "Iterator.from([1,2,3]).some(x => x === 2)",
    )]);
}

#[test]
fn iterator_some_false() {
    run_test_actions([TestAction::assert(
        "!Iterator.from([1,2,3]).some(x => x === 99)",
    )]);
}

#[test]
fn iterator_every_true() {
    run_test_actions([TestAction::assert(
        "Iterator.from([2,4,6]).every(x => x % 2 === 0)",
    )]);
}

#[test]
fn iterator_every_false() {
    run_test_actions([TestAction::assert(
        "!Iterator.from([2,4,5]).every(x => x % 2 === 0)",
    )]);
}

#[test]
fn iterator_find_found() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).find(x => x > 1)",
        2,
    )]);
}

#[test]
fn iterator_find_not_found() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3]).find(x => x > 99)",
        JsValue::undefined(),
    )]);
}

// ── IteratorHelper protocol ───────────────────────────────────────────────────

#[test]
fn iterator_helper_to_string_tag() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1]).map(x => x)[Symbol.toStringTag]",
        js_str!("Iterator Helper"),
    )]);
}

#[test]
fn iterator_helper_return_closes_underlying() {
    run_test_actions([
        TestAction::run(
            "let closed = false;
             const iter = {
               [Symbol.iterator]() { return this; },
               next() { return {value: 1, done: false}; },
               return() { closed = true; return {value: undefined, done: true}; }
             };
             const helper = Iterator.from(iter).map(x => x);
             helper.return();",
        ),
        TestAction::assert("closed"),
    ]);
}

// ── Chaining ─────────────────────────────────────────────────────────────────

#[test]
fn iterator_chaining() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.from([1,2,3,4,5,6,7,8,9,10])
           .filter(x => x % 2 === 0)
           .map(x => x * 3)
           .take(3)
           .toArray()
           .join(',')",
        js_str!("6,12,18"),
    )]);
}

#[test]
fn iterator_prototype_iterator_self() {
    run_test_actions([TestAction::assert(
        "Iterator.prototype[Symbol.iterator]() === Iterator.prototype",
    )]);
}

#[test]
fn iterator_concat_basic() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.concat([1,2],[3,4]).toArray().join(',')",
        js_str!("1,2,3,4"),
    )]);
}

#[test]
fn iterator_concat_zero_arguments() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.concat().toArray().length",
        0,
    )]);
}

// ── Iterator.zip — shortest mode (default) ──────────────────────────────────

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_basic_two_arrays() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(Iterator.zip([[1,2,3], ['a','b','c']]).toArray())",
        js_str!("[[1,\"a\"],[2,\"b\"],[3,\"c\"]]"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_basic_three_arrays() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(Iterator.zip([[1,2], ['a','b'], [true, false]]).toArray())",
        js_str!("[[1,\"a\",true],[2,\"b\",false]]"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_stops_at_shortest() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(Iterator.zip([[1,2,3], ['a']]).toArray())",
        js_str!("[[1,\"a\"]]"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_empty_iterables() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.zip([]).toArray().length",
        0,
    )]);
}

#[test]
fn iterator_concat_single_argument() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.concat([1,2]).toArray().join(',')",
        js_str!("1,2"),
    )]);
}

#[test]
fn iterator_concat_three_arguments() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.concat([1],[2],[3]).toArray().join(',')",
        js_str!("1,2,3"),
    )]);
}

#[test]
fn iterator_concat_lazy_next() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.concat([1,2],[3,4]).next().value",
        1,
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_single_iterable() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(Iterator.zip([[1,2,3]]).toArray())",
        js_str!("[[1],[2],[3]]"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_shortest_mode_explicit() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(Iterator.zip([[1,2,3], ['a','b']], { mode: 'shortest' }).toArray())",
        js_str!("[[1,\"a\"],[2,\"b\"]]"),
    )]);
}

// ── Iterator.zip — longest mode ─────────────────────────────────────────────

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_longest_pads_with_undefined() {
    run_test_actions([TestAction::assert_eq(
        r#"
        const result = Iterator.zip([[1,2,3], ['a']], { mode: 'longest' }).toArray();
        JSON.stringify(result)
        "#,
        js_str!("[[1,\"a\"],[2,null],[3,null]]"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_longest_custom_padding() {
    run_test_actions([TestAction::assert_eq(
        r#"
        const result = Iterator.zip(
            [[1,2,3], ['a']],
            { mode: 'longest', padding: ['?', '!'] }
        ).toArray();
        JSON.stringify(result)
        "#,
        js_str!("[[1,\"a\"],[2,\"!\"],[3,\"!\"]]"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_longest_same_length() {
    run_test_actions([TestAction::assert_eq(
        r#"
        JSON.stringify(Iterator.zip([[1,2], ['a','b']], { mode: 'longest' }).toArray())
        "#,
        js_str!("[[1,\"a\"],[2,\"b\"]]"),
    )]);
}

// ── Iterator.zip — strict mode ──────────────────────────────────────────────

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_strict_same_length() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(Iterator.zip([[1,2], ['a','b']], { mode: 'strict' }).toArray())",
        js_str!("[[1,\"a\"],[2,\"b\"]]"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_strict_different_length_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.zip([[1,2,3], ['a','b']], { mode: 'strict' }).toArray()",
        JsNativeErrorKind::Type,
        "iterators have different lengths in strict mode",
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_strict_first_shorter_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.zip([[1], ['a','b','c']], { mode: 'strict' }).toArray()",
        JsNativeErrorKind::Type,
        "iterators have different lengths in strict mode",
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_strict_empty_iterators() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.zip([[], []], { mode: 'strict' }).toArray().length",
        0,
    )]);
}

// ── Iterator.zipKeyed ───────────────────────────────────────────────────────

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_keyed_basic() {
    run_test_actions([TestAction::assert_eq(
        r#"
        const result = Iterator.zipKeyed({ a: [1,2,3], b: ['x','y','z'] }).toArray();
        result.map(o => o.a + ':' + o.b).join(',')
        "#,
        js_str!("1:x,2:y,3:z"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_keyed_shortest_default() {
    run_test_actions([TestAction::assert_eq(
        r#"
        const result = Iterator.zipKeyed({ x: [1,2,3], y: ['a'] }).toArray();
        result.length
        "#,
        1,
    )]);
}

#[test]
fn iterator_concat_non_object_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.concat(42)",
        JsNativeErrorKind::Type,
        "Iterator.concat requires iterable objects",
    )]);
}

#[test]
fn iterator_concat_missing_iterator_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.concat({})",
        JsNativeErrorKind::Type,
        "Iterator.concat requires objects with @@iterator",
    )]);
}

#[test]
fn iterator_concat_return_closes_inner() {
    run_test_actions([
        TestAction::run(
            "let closed = false;
             const iter = {
                 [Symbol.iterator]() {
                     return {
                         next() { return { value: 1, done: false }; },
                         return() { closed = true; return { done: true }; }
                     };
                 }
             };
             const it = Iterator.concat(iter);
             it.next();
             it.return();",
        ),
        TestAction::assert("closed"),
    ]);
}

#[test]
fn iterator_concat_return_result_shape() {
    run_test_actions([TestAction::assert(
        "const it = Iterator.concat([1,2]); it.next();
         const r = it.return(); r.done === true && r.value === undefined",
    )]);
}
#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_keyed_longest_mode() {
    run_test_actions([TestAction::assert_eq(
        r#"
        const result = Iterator.zipKeyed(
            { x: [1,2,3], y: ['a'] },
            { mode: 'longest' }
        ).toArray();
        result.length
        "#,
        3,
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_keyed_longest_with_padding() {
    run_test_actions([TestAction::assert_eq(
        r#"
        const result = Iterator.zipKeyed(
            { x: [1,2,3], y: ['a'] },
            { mode: 'longest', padding: { y: 'default' } }
        ).toArray();
        result.map(o => o.x + ':' + o.y).join(',')
        "#,
        js_str!("1:a,2:default,3:default"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_keyed_strict_same_length() {
    run_test_actions([TestAction::assert_eq(
        r#"
        const result = Iterator.zipKeyed(
            { a: [1,2], b: ['x','y'] },
            { mode: 'strict' }
        ).toArray();
        result.length
        "#,
        2,
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_keyed_strict_different_length_throws() {
    run_test_actions([TestAction::assert_native_error(
        r#"
        Iterator.zipKeyed(
            { a: [1,2,3], b: ['x','y'] },
            { mode: 'strict' }
        ).toArray()
        "#,
        JsNativeErrorKind::Type,
        "iterators have different lengths in strict mode",
    )]);
}

// ── Error handling ──────────────────────────────────────────────────────────

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_non_object_iterables_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.zip(42)",
        JsNativeErrorKind::Type,
        "Iterator.zip requires an iterable object",
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_invalid_mode_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.zip([[1]], { mode: 'invalid' })",
        JsNativeErrorKind::Type,
        "mode must be \"shortest\", \"longest\", or \"strict\"",
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_non_object_padding_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.zip([[1]], { mode: 'longest', padding: 42 })",
        JsNativeErrorKind::Type,
        "padding must be an object",
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_options_must_be_object() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.zip([[1]], 'notAnObject')",
        JsNativeErrorKind::Type,
        "options must be an object",
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_keyed_non_object_iterables_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Iterator.zipKeyed(42)",
        JsNativeErrorKind::Type,
        "Iterator.zipKeyed requires an object",
    )]);
}

// ── ZipIterator protocol ────────────────────────────────────────────────────

#[cfg(feature = "experimental")]
#[test]
fn zip_iterator_return_closes_iterators() {
    run_test_actions([
        TestAction::run(
            r#"
            let closed1 = false;
            let closed2 = false;
            const iter1 = {
                [Symbol.iterator]() { return this; },
                next() { return { value: 1, done: false }; },
                return() { closed1 = true; return { value: undefined, done: true }; }
            };
            const iter2 = {
                [Symbol.iterator]() { return this; },
                next() { return { value: 2, done: false }; },
                return() { closed2 = true; return { value: undefined, done: true }; }
            };
            const zipped = Iterator.zip([iter1, iter2]);
            zipped.return();
            "#,
        ),
        TestAction::assert("closed1"),
        TestAction::assert("closed2"),
    ]);
}

#[cfg(feature = "experimental")]
#[test]
fn zip_iterator_next_after_done() {
    run_test_actions([
        TestAction::run(
            r#"
            const zipped = Iterator.zip([[]]);
            "#,
        ),
        TestAction::assert_eq("JSON.stringify(zipped.next())", js_str!("{\"done\":true}")),
        TestAction::assert_eq("JSON.stringify(zipped.next())", js_str!("{\"done\":true}")),
    ]);
}

#[cfg(feature = "experimental")]
#[test]
fn zip_iterator_to_string_tag() {
    run_test_actions([TestAction::assert_eq(
        "Iterator.zip([[1]]).next(); Iterator.zip([[1]])[Symbol.toStringTag]",
        js_str!("Iterator Helper"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_with_generators() {
    run_test_actions([TestAction::assert_eq(
        r#"
        function* nums() { yield 1; yield 2; yield 3; }
        function* letters() { yield 'a'; yield 'b'; yield 'c'; }
        JSON.stringify(Iterator.zip([nums(), letters()]).toArray())
        "#,
        js_str!("[[1,\"a\"],[2,\"b\"],[3,\"c\"]]"),
    )]);
}

#[cfg(feature = "experimental")]
#[test]
fn iterator_zip_longest_with_three_iterators() {
    run_test_actions([TestAction::assert_eq(
        r#"
        const result = Iterator.zip([[1], [10, 20], [100, 200, 300]], { mode: 'longest' }).toArray();
        JSON.stringify(result)
        "#,
        js_str!("[[1,10,100],[null,20,200],[null,null,300]]"),
    )]);
}

#[test]
fn iterator_includes_basic() {
    run_test_actions([
        TestAction::run("const gen = () => Iterator.from([1, 3]);"),
        TestAction::assert_eq("gen().includes(1)", true),
        TestAction::assert_eq("gen().includes(2)", false),
        TestAction::assert_eq("gen().includes(3)", true),
        TestAction::assert_eq("gen().drop(1).includes(1)", false),
        TestAction::assert_eq("gen().drop(1).includes(3)", true),
        TestAction::assert_eq("gen().drop(2).includes(3)", false),
        TestAction::assert_eq("gen().includes(1, 1)", false),
        TestAction::assert_eq("gen().includes(3, 1)", true),
        TestAction::assert_eq("gen().includes(3, 2)", false),
    ]);
}

#[test]
fn iterator_includes_generator() {
    run_test_actions([
        TestAction::run("function* gen() { yield 1; yield 3; }"),
        TestAction::assert_eq("gen().includes(1)", true),
        TestAction::assert_eq("gen().includes(2)", false),
        TestAction::assert_eq("gen().includes(3)", true),
        TestAction::assert_eq("gen().drop(1).includes(1)", false),
        TestAction::assert_eq("gen().drop(1).includes(3)", true),
        TestAction::assert_eq("gen().drop(2).includes(3)", false),
        TestAction::assert_eq("gen().includes(1, 1)", false),
        TestAction::assert_eq("gen().includes(3, 1)", true),
        TestAction::assert_eq("gen().includes(3, 2)", false),
    ]);
}

#[test]
fn iterator_includes_errors() {
    run_test_actions([
        TestAction::run("const gen = () => Iterator.from([1, 3]);"),
        TestAction::assert_native_error(
            "gen().includes(1, NaN)",
            JsNativeErrorKind::Type,
            "skippedElements must be a number",
        ),
        TestAction::assert_native_error(
            "gen().includes(1, 'a string')",
            JsNativeErrorKind::Type,
            "skippedElements must be a number",
        ),
        TestAction::assert_native_error(
            "gen().includes(1, -1)",
            JsNativeErrorKind::Range,
            "skippedElements must be a positive number",
        ),
    ]);
}
