//! Tests for the Headers class

use super::TestFetcher;
use crate::test::{TestAction, run_test_actions};
use boa_engine::js_str;

#[test]
fn headers_is_iterable() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(indoc::indoc! {r#"
            const headers = new Headers([['x', 'y'], ['a', 'b']]);
            const result = [...headers];
            globalThis.testResult = result.length === 2 && result[0][0] === 'x' && result[0][1] === 'y' && result[1][0] === 'a' && result[1][1] === 'b';
        "#}),
        TestAction::inspect_context(|ctx| {
            let result = ctx.global_object().get(js_str!("testResult"), ctx).unwrap();
            assert_eq!(result.to_boolean(), true);
        }),
    ]);
}

#[test]
fn headers_can_be_used_with_map() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(indoc::indoc! {r#"
            const headers = new Headers([['x', 'y']]);
            const map = new Map(headers);
            globalThis.testResult = map.get('x') === 'y';
        "#}),
        TestAction::inspect_context(|ctx| {
            let result = ctx.global_object().get(js_str!("testResult"), ctx).unwrap();
            assert_eq!(result.to_boolean(), true);
        }),
    ]);
}

#[test]
fn headers_entries_returns_iterator() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(indoc::indoc! {r#"
            const headers = new Headers([['x', 'y']]);
            const iterator = headers.entries();
            const first = iterator.next();
            globalThis.testResult = !first.done && first.value[0] === 'x' && first.value[1] === 'y';
        "#}),
        TestAction::inspect_context(|ctx| {
            let result = ctx.global_object().get(js_str!("testResult"), ctx).unwrap();
            assert_eq!(result.to_boolean(), true);
        }),
    ]);
}

#[test]
fn headers_keys_returns_iterator() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(indoc::indoc! {r#"
            const headers = new Headers([['x', 'y'], ['a', 'b']]);
            const keys = [...headers.keys()];
            globalThis.testResult = keys.length === 2 && keys[0] === 'x' && keys[1] === 'a';
        "#}),
        TestAction::inspect_context(|ctx| {
            let result = ctx.global_object().get(js_str!("testResult"), ctx).unwrap();
            assert_eq!(result.to_boolean(), true);
        }),
    ]);
}

#[test]
fn headers_values_returns_iterator() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let fetcher = TestFetcher::default();
            crate::fetch::register(fetcher, None, ctx).expect("failed to register fetch");
        }),
        TestAction::run(indoc::indoc! {r#"
            const headers = new Headers([['x', 'y'], ['a', 'b']]);
            const values = [...headers.values()];
            globalThis.testResult = values.length === 2 && values[0] === 'y' && values[1] === 'b';
        "#}),
        TestAction::inspect_context(|ctx| {
            let result = ctx.global_object().get(js_str!("testResult"), ctx).unwrap();
            assert_eq!(result.to_boolean(), true);
        }),
    ]);
}
