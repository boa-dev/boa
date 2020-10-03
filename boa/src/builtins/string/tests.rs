use crate::{forward, forward_val, Context};

///TODO: re-enable when getProperty() is finished;
#[test]
#[ignore]
fn length() {
    //TEST262: https://github.com/tc39/test262/blob/master/test/built-ins/String/length.js
    let mut engine = Context::new();
    let init = r#"
    const a = new String(' ');
    const b = new String('\ud834\udf06');
    const c = new String(' \b ');
    cosnt d = new String('中文长度')
    "#;
    eprintln!("{}", forward(&mut engine, init));
    let a = forward(&mut engine, "a.length");
    assert_eq!(a, "1");
    let b = forward(&mut engine, "b.length");
    // TODO: fix this
    // unicode surrogate pair length should be 1
    // utf16/usc2 length should be 2
    // utf8 length should be 4
    assert_eq!(b, "2");
    let c = forward(&mut engine, "c.length");
    assert_eq!(c, "3");
    let d = forward(&mut engine, "d.length");
    assert_eq!(d, "4");
}

#[test]
fn new_string_has_length() {
    let mut engine = Context::new();
    let init = r#"
        let a = new String("1234");
        a
        "#;

    forward(&mut engine, init);
    assert_eq!(forward(&mut engine, "a.length"), "4");
}

#[test]
fn new_utf8_string_has_length() {
    let mut engine = Context::new();
    let init = r#"
        let a = new String("中文");
        a
        "#;

    forward(&mut engine, init);
    assert_eq!(forward(&mut engine, "a.length"), "2");
}

#[test]
fn concat() {
    let mut engine = Context::new();
    let init = r#"
        var hello = new String('Hello, ');
        var world = new String('world! ');
        var nice = new String('Have a nice day.');
        "#;
    eprintln!("{}", forward(&mut engine, init));

    let a = forward(&mut engine, "hello.concat(world, nice)");
    assert_eq!(a, "\"Hello, world! Have a nice day.\"");

    let b = forward(&mut engine, "hello + world + nice");
    assert_eq!(b, "\"Hello, world! Have a nice day.\"");
}

#[test]
fn generic_concat() {
    let mut engine = Context::new();
    let init = r#"
        Number.prototype.concat = String.prototype.concat;
        let number = new Number(100);
        "#;
    eprintln!("{}", forward(&mut engine, init));

    let a = forward(&mut engine, "number.concat(' - 50', ' = 50')");
    assert_eq!(a, "\"100 - 50 = 50\"");
}

#[allow(clippy::unwrap_used)]
#[test]
/// Test the correct type is returned from call and construct
fn construct_and_call() {
    let mut engine = Context::new();
    let init = r#"
        var hello = new String('Hello');
        var world = String('world');
        "#;

    forward(&mut engine, init);

    let hello = forward_val(&mut engine, "hello").unwrap();
    let world = forward_val(&mut engine, "world").unwrap();

    assert_eq!(hello.is_object(), true);
    assert_eq!(world.is_string(), true);
}

#[test]
fn repeat() {
    let mut engine = Context::new();
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "empty.repeat(0)"), "\"\"");
    assert_eq!(forward(&mut engine, "empty.repeat(1)"), "\"\"");

    assert_eq!(forward(&mut engine, "en.repeat(0)"), "\"\"");
    assert_eq!(forward(&mut engine, "zh.repeat(0)"), "\"\"");

    assert_eq!(forward(&mut engine, "en.repeat(1)"), "\"english\"");
    assert_eq!(forward(&mut engine, "zh.repeat(2)"), "\"中文中文\"");
}

#[test]
fn repeat_throws_when_count_is_negative() {
    let mut engine = Context::new();

    assert_eq!(
        forward(
            &mut engine,
            r#"
        try {
            'x'.repeat(-1)
        } catch (e) {
            e.toString()
        }
    "#
        ),
        "\"RangeError: repeat count cannot be a negative number\""
    );
}

#[test]
fn repeat_throws_when_count_is_infinity() {
    let mut engine = Context::new();

    assert_eq!(
        forward(
            &mut engine,
            r#"
        try {
            'x'.repeat(Infinity)
        } catch (e) {
            e.toString()
        }
    "#
        ),
        "\"RangeError: repeat count cannot be infinity\""
    );
}

#[test]
fn repeat_throws_when_count_overflows_max_length() {
    let mut engine = Context::new();

    assert_eq!(
        forward(
            &mut engine,
            r#"
        try {
            'x'.repeat(2 ** 64)
        } catch (e) {
            e.toString()
        }
    "#
        ),
        "\"RangeError: repeat count must not overflow maximum string length\""
    );
}

#[test]
fn repeat_generic() {
    let mut engine = Context::new();
    let init = "Number.prototype.repeat = String.prototype.repeat;";

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "(0).repeat(0)"), "\"\"");
    assert_eq!(forward(&mut engine, "(1).repeat(1)"), "\"1\"");

    assert_eq!(forward(&mut engine, "(1).repeat(5)"), "\"11111\"");
    assert_eq!(forward(&mut engine, "(12).repeat(3)"), "\"121212\"");
}

#[test]
fn replace() {
    let mut engine = Context::new();
    let init = r#"
        var a = "abc";
        a = a.replace("a", "2");
        a
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "a"), "\"2bc\"");
}

#[test]
fn replace_no_match() {
    let mut engine = Context::new();
    let init = r#"
        var a = "abc";
        a = a.replace(/d/, "$&$&");
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "a"), "\"abc\"");
}

#[test]
fn replace_with_capture_groups() {
    let mut engine = Context::new();
    let init = r#"
        var re = /(\w+)\s(\w+)/;
        var a = "John Smith";
        a = a.replace(re, '$2, $1');
        a
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "a"), "\"Smith, John\"");
}

#[test]
fn replace_with_tenth_capture_group() {
    let mut engine = Context::new();
    let init = r#"
        var re = /(\d)(\d)(\d)(\d)(\d)(\d)(\d)(\d)(\d)(\d)/;
        var a = "0123456789";
        let res = a.replace(re, '$10');
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "res"), "\"9\"");
}

#[test]
fn replace_substitutions() {
    let mut engine = Context::new();
    let init = r#"
        var re = / two /;
        var a = "one two three";
        var dollar = a.replace(re, " $$ ");
        var matched = a.replace(re, "$&$&");
        var start = a.replace(re, " $` ");
        var end = a.replace(re, " $' ");
        var no_sub = a.replace(re, " $_ ");
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "dollar"), "\"one $ three\"");
    assert_eq!(forward(&mut engine, "matched"), "\"one two  two three\"");
    assert_eq!(forward(&mut engine, "start"), "\"one one three\"");
    assert_eq!(forward(&mut engine, "end"), "\"one three three\"");
    assert_eq!(forward(&mut engine, "no_sub"), "\"one $_ three\"");
}

#[test]
fn replace_with_function() {
    let mut engine = Context::new();
    let init = r#"
        var a = "ecmascript is cool";
        var p1, p2, p3;
        var replacer = (match, cap1, cap2, cap3) => {
            p1 = cap1;
            p2 = cap2;
            p3 = cap3;
            return "awesome!";
        };

        a = a.replace(/c(o)(o)(l)/, replacer);
        a;
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "a"), "\"ecmascript is awesome!\"");

    assert_eq!(forward(&mut engine, "p1"), "\"o\"");
    assert_eq!(forward(&mut engine, "p2"), "\"o\"");
    assert_eq!(forward(&mut engine, "p3"), "\"l\"");
}

#[test]
fn starts_with() {
    let mut engine = Context::new();
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');

        var emptyLiteral = '';
        var enLiteral = 'english';
        var zhLiteral = '中文';
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "empty.startsWith('')"), "true");
    assert_eq!(forward(&mut engine, "en.startsWith('e')"), "true");
    assert_eq!(forward(&mut engine, "zh.startsWith('中')"), "true");

    assert_eq!(forward(&mut engine, "emptyLiteral.startsWith('')"), "true");
    assert_eq!(forward(&mut engine, "enLiteral.startsWith('e')"), "true");
    assert_eq!(forward(&mut engine, "zhLiteral.startsWith('中')"), "true");
}

#[test]
fn starts_with_with_regex_arg() {
    let mut engine = Context::new();

    let scenario = r#"
        try {
            'Saturday night'.startsWith(/Saturday/);
        } catch (e) {
            e.toString();
        }
    "#;

    assert_eq!(
        forward(
            &mut engine, scenario
        ),
        "\"TypeError: First argument to String.prototype.startsWith must not be a regular expression\""
    )
}

#[test]
fn ends_with() {
    let mut engine = Context::new();
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');

        var emptyLiteral = '';
        var enLiteral = 'english';
        var zhLiteral = '中文';
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "empty.endsWith('')"), "true");
    assert_eq!(forward(&mut engine, "en.endsWith('h')"), "true");
    assert_eq!(forward(&mut engine, "zh.endsWith('文')"), "true");

    assert_eq!(forward(&mut engine, "emptyLiteral.endsWith('')"), "true");
    assert_eq!(forward(&mut engine, "enLiteral.endsWith('h')"), "true");
    assert_eq!(forward(&mut engine, "zhLiteral.endsWith('文')"), "true");
}

#[test]
fn ends_with_with_regex_arg() {
    let mut engine = Context::new();

    let scenario = r#"
        try {
            'Saturday night'.endsWith(/night/);
        } catch (e) {
            e.toString();
        }
    "#;

    assert_eq!(
        forward(
            &mut engine, scenario
        ),
        "\"TypeError: First argument to String.prototype.endsWith must not be a regular expression\""
    )
}

#[test]
fn includes() {
    let mut engine = Context::new();
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');

        var emptyLiteral = '';
        var enLiteral = 'english';
        var zhLiteral = '中文';
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "empty.includes('')"), "true");
    assert_eq!(forward(&mut engine, "en.includes('g')"), "true");
    assert_eq!(forward(&mut engine, "zh.includes('文')"), "true");

    assert_eq!(forward(&mut engine, "emptyLiteral.includes('')"), "true");
    assert_eq!(forward(&mut engine, "enLiteral.includes('g')"), "true");
    assert_eq!(forward(&mut engine, "zhLiteral.includes('文')"), "true");
}

#[test]
fn includes_with_regex_arg() {
    let mut engine = Context::new();

    let scenario = r#"
        try {
            'Saturday night'.includes(/day/);
        } catch (e) {
            e.toString();
        }
    "#;

    assert_eq!(
        forward(
            &mut engine, scenario
        ),
        "\"TypeError: First argument to String.prototype.includes must not be a regular expression\""
    )
}

#[test]
fn match_all() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "'aa'.matchAll(null).length"), "0");
    assert_eq!(forward(&mut engine, "'aa'.matchAll(/b/).length"), "0");
    assert_eq!(forward(&mut engine, "'aa'.matchAll(/a/).length"), "1");
    assert_eq!(forward(&mut engine, "'aa'.matchAll(/a/g).length"), "2");

    forward(
        &mut engine,
        "var groupMatches = 'test1test2'.matchAll(/t(e)(st(\\d?))/g)",
    );

    assert_eq!(forward(&mut engine, "groupMatches.length"), "2");
    assert_eq!(forward(&mut engine, "groupMatches[0][1]"), "\"e\"");
    assert_eq!(forward(&mut engine, "groupMatches[0][2]"), "\"st1\"");
    assert_eq!(forward(&mut engine, "groupMatches[0][3]"), "\"1\"");
    assert_eq!(forward(&mut engine, "groupMatches[1][3]"), "\"2\"");

    assert_eq!(
        forward(
            &mut engine,
            "'test1test2'.matchAll(/t(e)(st(\\d?))/).length"
        ),
        "1"
    );

    let init = r#"
        var regexp = RegExp('foo[a-z]*','g');
        var str = 'table football, foosball';
        var matches = str.matchAll(regexp);
        "#;

    forward(&mut engine, init);

    assert_eq!(forward(&mut engine, "matches[0][0]"), "\"football\"");
    assert_eq!(forward(&mut engine, "matches[0].index"), "6");
    assert_eq!(forward(&mut engine, "matches[1][0]"), "\"foosball\"");
    assert_eq!(forward(&mut engine, "matches[1].index"), "16");
}

#[test]
fn test_match() {
    let mut engine = Context::new();
    let init = r#"
        var str = new String('The Quick Brown Fox Jumps Over The Lazy Dog');
        var result1 = str.match(/quick\s(brown).+?(jumps)/i);
        var result2 = str.match(/[A-Z]/g);
        var result3 = str.match("T");
        var result4 = str.match(RegExp("B", 'g'));
        "#;

    forward(&mut engine, init);

    assert_eq!(
        forward(&mut engine, "result1[0]"),
        "\"Quick Brown Fox Jumps\""
    );
    assert_eq!(forward(&mut engine, "result1[1]"), "\"Brown\"");
    assert_eq!(forward(&mut engine, "result1[2]"), "\"Jumps\"");
    assert_eq!(forward(&mut engine, "result1.index"), "4");
    assert_eq!(
        forward(&mut engine, "result1.input"),
        "\"The Quick Brown Fox Jumps Over The Lazy Dog\""
    );

    assert_eq!(forward(&mut engine, "result2[0]"), "\"T\"");
    assert_eq!(forward(&mut engine, "result2[1]"), "\"Q\"");
    assert_eq!(forward(&mut engine, "result2[2]"), "\"B\"");
    assert_eq!(forward(&mut engine, "result2[3]"), "\"F\"");
    assert_eq!(forward(&mut engine, "result2[4]"), "\"J\"");
    assert_eq!(forward(&mut engine, "result2[5]"), "\"O\"");
    assert_eq!(forward(&mut engine, "result2[6]"), "\"T\"");
    assert_eq!(forward(&mut engine, "result2[7]"), "\"L\"");
    assert_eq!(forward(&mut engine, "result2[8]"), "\"D\"");

    assert_eq!(forward(&mut engine, "result3[0]"), "\"T\"");
    assert_eq!(forward(&mut engine, "result3.index"), "0");
    assert_eq!(
        forward(&mut engine, "result3.input"),
        "\"The Quick Brown Fox Jumps Over The Lazy Dog\""
    );
    assert_eq!(forward(&mut engine, "result4[0]"), "\"B\"");
}

#[test]
fn trim() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "'Hello'.trim()"), "\"Hello\"");
    assert_eq!(forward(&mut engine, "' \nHello'.trim()"), "\"Hello\"");
    assert_eq!(forward(&mut engine, "'Hello \n\r'.trim()"), "\"Hello\"");
    assert_eq!(forward(&mut engine, "' Hello '.trim()"), "\"Hello\"");
}

#[test]
fn trim_start() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "'Hello'.trimStart()"), "\"Hello\"");
    assert_eq!(forward(&mut engine, "' \nHello'.trimStart()"), "\"Hello\"");
    assert_eq!(
        forward(&mut engine, "'Hello \n'.trimStart()"),
        "\"Hello \n\""
    );
    assert_eq!(forward(&mut engine, "' Hello '.trimStart()"), "\"Hello \"");
}

#[test]
fn trim_end() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "'Hello'.trimEnd()"), "\"Hello\"");
    assert_eq!(forward(&mut engine, "' \nHello'.trimEnd()"), "\" \nHello\"");
    assert_eq!(forward(&mut engine, "'Hello \n'.trimEnd()"), "\"Hello\"");
    assert_eq!(forward(&mut engine, "' Hello '.trimEnd()"), "\" Hello\"");
}

#[test]
fn index_of_with_no_arguments() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.indexOf()"), "-1");
    assert_eq!(forward(&mut engine, "'undefined'.indexOf()"), "0");
    assert_eq!(forward(&mut engine, "'a1undefined'.indexOf()"), "2");
    assert_eq!(forward(&mut engine, "'a1undefined1a'.indexOf()"), "2");
    assert_eq!(forward(&mut engine, "'µµµundefined'.indexOf()"), "3");
    assert_eq!(forward(&mut engine, "'µµµundefinedµµµ'.indexOf()"), "3");
}

#[test]
fn index_of_with_string_search_string_argument() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.indexOf('hello')"), "-1");
    assert_eq!(
        forward(&mut engine, "'undefined'.indexOf('undefined')"),
        "0"
    );
    assert_eq!(
        forward(&mut engine, "'a1undefined'.indexOf('undefined')"),
        "2"
    );
    assert_eq!(
        forward(&mut engine, "'a1undefined1a'.indexOf('undefined')"),
        "2"
    );
    assert_eq!(
        forward(&mut engine, "'µµµundefined'.indexOf('undefined')"),
        "3"
    );
    assert_eq!(
        forward(&mut engine, "'µµµundefinedµµµ'.indexOf('undefined')"),
        "3"
    );
}

#[test]
fn index_of_with_non_string_search_string_argument() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.indexOf(1)"), "-1");
    assert_eq!(forward(&mut engine, "'1'.indexOf(1)"), "0");
    assert_eq!(forward(&mut engine, "'true'.indexOf(true)"), "0");
    assert_eq!(forward(&mut engine, "'ab100ba'.indexOf(100)"), "2");
    assert_eq!(forward(&mut engine, "'µµµfalse'.indexOf(true)"), "-1");
    assert_eq!(forward(&mut engine, "'µµµ5µµµ'.indexOf(5)"), "3");
}

#[test]
fn index_of_with_from_index_argument() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.indexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut engine, "'x'.indexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut engine, "'abcx'.indexOf('x', 2)"), "3");
    assert_eq!(forward(&mut engine, "'x'.indexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut engine, "'µµµxµµµ'.indexOf('x', 2)"), "3");

    assert_eq!(
        forward(&mut engine, "'µµµxµµµ'.indexOf('x', 10000000)"),
        "-1"
    );
}

#[test]
fn generic_index_of() {
    let mut engine = Context::new();
    forward_val(
        &mut engine,
        "Number.prototype.indexOf = String.prototype.indexOf",
    )
    .unwrap();

    assert_eq!(forward(&mut engine, "(10).indexOf(9)"), "-1");
    assert_eq!(forward(&mut engine, "(10).indexOf(0)"), "1");
    assert_eq!(forward(&mut engine, "(10).indexOf('0')"), "1");
}

#[test]
fn index_of_empty_search_string() {
    let mut engine = Context::new();

    assert_eq!(forward(&mut engine, "''.indexOf('')"), "0");
    assert_eq!(forward(&mut engine, "''.indexOf('', 10)"), "0");
    assert_eq!(forward(&mut engine, "'ABC'.indexOf('', 1)"), "1");
    assert_eq!(forward(&mut engine, "'ABC'.indexOf('', 2)"), "2");
    assert_eq!(forward(&mut engine, "'ABC'.indexOf('', 10)"), "3");
}

#[test]
fn last_index_of_with_no_arguments() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.lastIndexOf()"), "-1");
    assert_eq!(forward(&mut engine, "'undefined'.lastIndexOf()"), "0");
    assert_eq!(forward(&mut engine, "'a1undefined'.lastIndexOf()"), "2");
    assert_eq!(
        forward(&mut engine, "'a1undefined1aundefined'.lastIndexOf()"),
        "13"
    );
    assert_eq!(
        forward(&mut engine, "'µµµundefinedundefined'.lastIndexOf()"),
        "12"
    );
    assert_eq!(
        forward(&mut engine, "'µµµundefinedµµµundefined'.lastIndexOf()"),
        "15"
    );
}

#[test]
fn last_index_of_with_string_search_string_argument() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.lastIndexOf('hello')"), "-1");
    assert_eq!(
        forward(&mut engine, "'undefined'.lastIndexOf('undefined')"),
        "0"
    );
    assert_eq!(
        forward(&mut engine, "'a1undefined'.lastIndexOf('undefined')"),
        "2"
    );
    assert_eq!(
        forward(
            &mut engine,
            "'a1undefined1aundefined'.lastIndexOf('undefined')"
        ),
        "13"
    );
    assert_eq!(
        forward(
            &mut engine,
            "'µµµundefinedundefined'.lastIndexOf('undefined')"
        ),
        "12"
    );
    assert_eq!(
        forward(
            &mut engine,
            "'µµµundefinedµµµundefined'.lastIndexOf('undefined')"
        ),
        "15"
    );
}

#[test]
fn last_index_of_with_non_string_search_string_argument() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.lastIndexOf(1)"), "-1");
    assert_eq!(forward(&mut engine, "'1'.lastIndexOf(1)"), "0");
    assert_eq!(forward(&mut engine, "'11'.lastIndexOf(1)"), "1");
    assert_eq!(
        forward(&mut engine, "'truefalsetrue'.lastIndexOf(true)"),
        "9"
    );
    assert_eq!(forward(&mut engine, "'ab100ba'.lastIndexOf(100)"), "2");
    assert_eq!(forward(&mut engine, "'µµµfalse'.lastIndexOf(true)"), "-1");
    assert_eq!(forward(&mut engine, "'µµµ5µµµ65µ'.lastIndexOf(5)"), "8");
}

#[test]
fn last_index_of_with_from_index_argument() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.lastIndexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut engine, "'x'.lastIndexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut engine, "'abcxx'.lastIndexOf('x', 2)"), "4");
    assert_eq!(forward(&mut engine, "'x'.lastIndexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut engine, "'µµµxµµµ'.lastIndexOf('x', 2)"), "3");

    assert_eq!(
        forward(&mut engine, "'µµµxµµµ'.lastIndexOf('x', 10000000)"),
        "-1"
    );
}

#[test]
fn last_index_with_empty_search_string() {
    let mut engine = Context::new();
    assert_eq!(forward(&mut engine, "''.lastIndexOf('')"), "0");
    assert_eq!(forward(&mut engine, "'x'.lastIndexOf('', 2)"), "1");
    assert_eq!(forward(&mut engine, "'abcxx'.lastIndexOf('', 4)"), "4");
    assert_eq!(forward(&mut engine, "'µµµxµµµ'.lastIndexOf('', 2)"), "2");

    assert_eq!(forward(&mut engine, "'abc'.lastIndexOf('', 10000000)"), "3");
}

#[test]
fn generic_last_index_of() {
    let mut engine = Context::new();
    forward_val(
        &mut engine,
        "Number.prototype.lastIndexOf = String.prototype.lastIndexOf",
    )
    .unwrap();

    assert_eq!(forward(&mut engine, "(1001).lastIndexOf(9)"), "-1");
    assert_eq!(forward(&mut engine, "(1001).lastIndexOf(0)"), "2");
    assert_eq!(forward(&mut engine, "(1001).lastIndexOf('0')"), "2");
}

#[test]
fn last_index_non_integer_position_argument() {
    let mut engine = Context::new();
    assert_eq!(
        forward(&mut engine, "''.lastIndexOf('x', new Number(4))"),
        "-1"
    );
    assert_eq!(
        forward(&mut engine, "'abc'.lastIndexOf('b', new Number(1))"),
        "1"
    );
    assert_eq!(
        forward(&mut engine, "'abcx'.lastIndexOf('x', new String('1'))"),
        "3"
    );
    assert_eq!(
        forward(&mut engine, "'abcx'.lastIndexOf('x', new String('100'))"),
        "-1"
    );
    assert_eq!(forward(&mut engine, "'abcx'.lastIndexOf('x', null)"), "3");
}

#[test]
fn empty_iter() {
    let mut engine = Context::new();
    let init = r#"
        let iter = new String()[Symbol.iterator]();
        let next = iter.next();
    "#;
    forward(&mut engine, init);
    assert_eq!(forward(&mut engine, "next.value"), "undefined");
    assert_eq!(forward(&mut engine, "next.done"), "true");
}

#[test]
fn ascii_iter() {
    let mut engine = Context::new();
    let init = r#"
        let iter = new String("Hello World")[Symbol.iterator]();
        let next = iter.next();
    "#;
    forward(&mut engine, init);
    assert_eq!(forward(&mut engine, "next.value"), "\"H\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"e\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"l\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"l\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"o\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\" \"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"W\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"o\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"r\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"l\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"d\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "undefined");
    assert_eq!(forward(&mut engine, "next.done"), "true");
}

#[test]
fn unicode_iter() {
    let mut engine = Context::new();
    let init = r#"
        let iter = new String("C🙂🙂l W🙂rld")[Symbol.iterator]();
        let next = iter.next();
    "#;
    forward(&mut engine, init);
    assert_eq!(forward(&mut engine, "next.value"), "\"C\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"🙂\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"🙂\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"l\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\" \"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"W\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"🙂\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"r\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"l\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "\"d\"");
    assert_eq!(forward(&mut engine, "next.done"), "false");
    forward(&mut engine, "next = iter.next()");
    assert_eq!(forward(&mut engine, "next.value"), "undefined");
    assert_eq!(forward(&mut engine, "next.done"), "true");
}
