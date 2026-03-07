use crate::{TestAction, run_test_actions};

#[test]
fn iterator_map() {
    run_test_actions([TestAction::assert(
        "[1,2,3].values().map(x => x * 2).toArray().join(',') === '2,4,6'",
    )]);
}

#[test]
fn iterator_filter() {
    run_test_actions([TestAction::assert(
        "[1,2,3,4].values().filter(x => x % 2 === 0).toArray().join(',') === '2,4'",
    )]);
}

#[test]
fn iterator_take() {
    run_test_actions([TestAction::assert(
        "[1,2,3,4,5].values().take(3).toArray().join(',') === '1,2,3'",
    )]);
}

#[test]
fn iterator_drop() {
    run_test_actions([TestAction::assert(
        "[1,2,3,4,5].values().drop(2).toArray().join(',') === '3,4,5'",
    )]);
}

#[test]
fn iterator_to_array() {
    run_test_actions([TestAction::assert(
        "[10,20,30].values().toArray().join(',') === '10,20,30'",
    )]);
}

#[test]
fn iterator_for_each() {
    run_test_actions([TestAction::assert(
        "let s = ''; [1,2,3].values().forEach(x => { s += x; }); s === '123'",
    )]);
}

#[test]
fn iterator_some() {
    run_test_actions([
        TestAction::assert("[1,2,3].values().some(x => x > 2)"),
        TestAction::assert("![1,2,3].values().some(x => x > 10)"),
    ]);
}

#[test]
fn iterator_every() {
    run_test_actions([
        TestAction::assert("[1,2,3].values().every(x => x > 0)"),
        TestAction::assert("![1,2,3].values().every(x => x > 1)"),
    ]);
}

#[test]
fn iterator_find() {
    run_test_actions([
        TestAction::assert("[1,2,3].values().find(x => x === 2) === 2"),
        TestAction::assert("[1,2,3].values().find(x => x === 99) === undefined"),
    ]);
}

#[test]
fn iterator_reduce() {
    run_test_actions([TestAction::assert(
        "[1,2,3].values().reduce((a, x) => a + x, 0) === 6",
    )]);
}

#[test]
fn iterator_flat_map() {
    run_test_actions([TestAction::assert(
        "[1,2].values().flatMap(x => [x, x * 10].values()).toArray().join(',') === '1,10,2,20'",
    )]);
}

#[test]
fn iterator_from() {
    run_test_actions([TestAction::assert(
        "Iterator.from([1,2,3][Symbol.iterator]()).toArray().join(',') === '1,2,3'",
    )]);
}

#[test]
fn iterator_chaining() {
    run_test_actions([TestAction::assert(
        "[1,2,3,4,5].values().filter(x => x % 2 !== 0).map(x => x * 10).toArray().join(',') === '10,30,50'",
    )]);
}
