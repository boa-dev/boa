use crate::{TestAction, run_test_actions};
use indoc::indoc;

#[test]
fn promise() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                    let count = 0;
                    const promise = new Promise((resolve, reject) => {
                        count += 1;
                        resolve(undefined);
                    }).then((_) => (count += 1));
                    count += 1;
                "#}),
        TestAction::assert_eq("count", 2),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("count", 3),
    ]);
}

#[test]
fn promise_all_resolves_values() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var values = [];
            var p = Promise.all([Promise.resolve(1), Promise.resolve(2)]);
            p.then(v => { values = v; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("values.length", 2),
        TestAction::assert_eq("values[0]", 1),
        TestAction::assert_eq("values[1]", 2),
    ]);
}

#[test]
fn promise_all_rejects() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var err = null;
            var p = Promise.all([Promise.resolve(1), Promise.reject(2)]);
            p.catch(e => { err = e; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("err", 2),
    ]);
}

#[test]
fn promise_any_resolves_first_success() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var val = null;
            var p = Promise.any([Promise.reject(1), Promise.resolve(2)]);
            p.then(v => { val = v; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("val", 2),
    ]);
}
