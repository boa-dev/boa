mod miri {

    use indoc::indoc;

    use crate::{JsNativeErrorKind, TestAction, run_test_actions};

    #[test]
    fn finalization_registry_simple() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;
            const registry = new FinalizationRegistry(() => {
                counter++;
            });

            registry.register(["foo"]);
        "#}),
            TestAction::assert_eq("counter", 0),
            TestAction::inspect_context(|_| boa_gc::force_collect()),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Callback should run at least once
            TestAction::assert_eq("counter", 1),
        ]);
    }

    #[test]
    fn finalization_registry_unregister() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;
            const registry = new FinalizationRegistry(() => {
                counter++;
            });

            {
                let array = ["foo"];
                registry.register(array, undefined, array);
                registry.unregister(array);
            }

        "#}),
            TestAction::assert_eq("counter", 0),
            TestAction::inspect_context(|_| boa_gc::force_collect()),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Callback shouldn't run
            TestAction::assert_eq("counter", 0),
        ]);
    }

    #[test]
    fn finalization_registry_held_value_handover() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;
            const registry = new FinalizationRegistry((value) => {
                counter += value.increment;
            });

            registry.register(["foo"], { increment: 5 });
        "#}),
            TestAction::assert_eq("counter", 0),
            TestAction::inspect_context(|_| boa_gc::force_collect()),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Registry should handover the held value as argument
            TestAction::assert_eq("counter", 5),
        ]);
    }

    #[test]
    fn finalization_registry_symbol_unregister_token() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;
            const registry = new FinalizationRegistry(() => {
                counter++;
            });
            const token = Symbol("token");

            {
                let array = ["foo"];
                registry.register(array, undefined, token);
            }
        "#}),
            // An unrelated symbol should not unregister the cell
            TestAction::assert_eq("registry.unregister(Symbol('token'))", false),
            TestAction::assert_eq("registry.unregister(token)", true),
            TestAction::inspect_context(|_| boa_gc::force_collect()),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Callback shouldn't run
            TestAction::assert_eq("counter", 0),
        ]);
    }

    #[test]
    fn finalization_registry_registered_symbol_unregister_token_throws() {
        run_test_actions([
            TestAction::run("const registry = new FinalizationRegistry(() => {});"),
            TestAction::assert_native_error(
                "registry.register({}, undefined, Symbol.for('token'))",
                JsNativeErrorKind::Type,
                "FinalizationRegistry.prototype.register: `unregisterToken` must be \
                an Object, a non-registered Symbol, or undefined",
            ),
            TestAction::assert_native_error(
                "registry.unregister(Symbol.for('token'))",
                JsNativeErrorKind::Type,
                "FinalizationRegistry.prototype.unregister: `unregisterToken` must be \
                an Object or a non-registered Symbol.",
            ),
        ]);
    }

    #[test]
    fn finalization_registry_unrelated_unregister_token() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;

            const registry = new FinalizationRegistry((value) => {
                counter += 1;
            });

            registry.register(["foo"], undefined, {});
            registry.unregister({});
        "#}),
            TestAction::assert_eq("counter", 0),
            TestAction::inspect_context(|_| boa_gc::force_collect()),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Object should not have been unregistered if the token is not the correct one
            TestAction::assert_eq("counter", 1),
        ]);
    }
}
