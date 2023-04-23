use super::{formatter, Console};
use crate::test::{run_test_actions, run_test_actions_with, TestAction};
use boa_engine::{property::Attribute, Context, JsValue};
use indoc::indoc;

#[test]
fn formatter_no_args_is_empty_string() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(formatter(&[], ctx).unwrap(), "");
    })]);
}

#[test]
fn formatter_empty_format_string_is_empty_string() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(formatter(&[JsValue::new("")], ctx).unwrap(), "");
    })]);
}

#[test]
fn formatter_format_without_args_renders_verbatim() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(&[JsValue::new("%d %s %% %f")], ctx).unwrap(),
            "%d %s %% %f"
        );
    })]);
}

#[test]
fn formatter_empty_format_string_concatenates_rest_of_args() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(
                &[
                    JsValue::new(""),
                    JsValue::new("to powinno zostać"),
                    JsValue::new("połączone"),
                ],
                ctx
            )
            .unwrap(),
            " to powinno zostać połączone"
        );
    })]);
}

#[test]
fn formatter_utf_8_checks() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(
                &[
                    JsValue::new("Są takie chwile %dą %są tu%sów %привет%ź".to_string()),
                    JsValue::new(123),
                    JsValue::new(1.23),
                    JsValue::new("ł"),
                ],
                ctx
            )
            .unwrap(),
            "Są takie chwile 123ą 1.23ą tułów %привет%ź"
        );
    })]);
}

#[test]
fn formatter_trailing_format_leader_renders() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(&[JsValue::new("%%%%%"), JsValue::new("|")], ctx).unwrap(),
            "%%% |"
        );
    })]);
}

#[test]
#[allow(clippy::approx_constant)]
fn formatter_float_format_works() {
    run_test_actions([TestAction::inspect_context(|ctx| {
        assert_eq!(
            formatter(&[JsValue::new("%f"), JsValue::new(3.1415)], ctx).unwrap(),
            "3.141500"
        );
    })]);
}

#[test]
fn console_log_cyclic() {
    let mut context = Context::default();
    let console = Console::init(&mut context);
    context
        .register_global_property(Console::NAME, console, Attribute::all())
        .unwrap();

    run_test_actions_with(
        [TestAction::run(indoc! {r#"
                let a = [1];
                a[1] = a;
                console.log(a);
            "#})],
        &mut context,
    );
    // Should not stack overflow
}
