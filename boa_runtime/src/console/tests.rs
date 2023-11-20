use super::Console;

use boa_engine::{js_string, property::Attribute, Context, Source};

fn test_console_output(code: &str, expect_output: &str) {
    let mut context = Context::default();
    let console = Console::init(&mut context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("register_global_property console error");

    let hack_code = Source::from_bytes(include_bytes!("./deno-scripts/02_hack_print.js"));
    context.eval(hack_code).expect("eval hack script error");

    context
        .eval(Source::from_bytes(code.as_bytes()))
        .expect("eval test code error");
    let output_value = context
        .global_object()
        .get(js_string!("__hack_print_output"), &mut context)
        .expect("get output JsValue error");
    let output_string: String = output_value
        .as_string()
        .expect("get output JsString error")
        .to_std_string()
        .expect("get output string error");
    assert_eq!(output_string, expect_output);
}

#[test]
fn test_console() {
    test_console_output("console.log([''])", "[ \"\" ]\n");
    test_console_output("console.log([])", "[]\n");
    test_console_output("console.log(\"\")", "\n");
    test_console_output("console.log(\"1\")", "1\n");
    test_console_output("console.log(0.1+0.2)", "0.30000000000000004\n");

    test_console_output(
        "let a = [1]; a[1]=a; console.log(a)",
        "<ref *1> [ 1, [Circular *1] ]\n",
    );
    test_console_output(
        "let a = new Set([1,2,1,4]); console.log(a)",
        "Set(3) { 1, 2, 4 }\n",
    );
    test_console_output(
        "let a = new Map([[1, 1],[2,2],[1, 3],[4, 4]]); console.log(a)",
        "Map(3) { [ 1, 3 ], [ 2, 2 ], [ 4, 4 ] }\n",
    );

    test_console_output(
        "console.log(['', 'to powinno zostać', 'połączone'].join(' '))",
        " to powinno zostać połączone\n",
    );
    test_console_output("console.log(['你', '好'].join())", "你,好\n");
    test_console_output(
        "console.log('Są takie chwile %dą %są tu%sów %привет%ź', 123,1.23,'ł')",
        "Są takie chwile 123ą 1.23ą tułów %привет%ź\n",
    );
    // FIXME: directly log a string array containing Chinese or other languages without using the 'join' method causes error
    // https://github.com/ridiculousfish/regress/pull/74 need update regress version
    // test_console_output("console.log(['你', '好'])", "[ \"你\", \"好\" ]\n");
    // test_console_output("console.log(['to powinno zostać', 'połączone'])", "[ \"to powinno zostać\", \"połączone\" ]\n");

    test_console_output("console.log('%%%%%', '|')", "%%% |\n");
    test_console_output("console.log('a%fb', 3.141500)", "a3.1415b\n");
    test_console_output("console.log('%d %s %% %f')", "%d %s %% %f\n");
}
