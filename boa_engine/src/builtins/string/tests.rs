use crate::{forward, forward_val, Context};

#[test]
fn length() {
    //TEST262: https://github.com/tc39/test262/blob/master/test/built-ins/String/length.js
    let mut context = Context::default();
    let init = r#"
    const a = new String(' ');
    const b = new String('\ud834\udf06');
    const c = new String(' \b ');
    const d = new String('中文长度')
    "#;
    eprintln!("{}", forward(&mut context, init));
    let a = forward(&mut context, "a.length");
    assert_eq!(a, "1");
    let b = forward(&mut context, "b.length");
    // unicode surrogate pair length should be 1
    // utf16/usc2 length should be 2
    // utf8 length should be 4
    assert_eq!(b, "2");
    let c = forward(&mut context, "c.length");
    assert_eq!(c, "3");
    let d = forward(&mut context, "d.length");
    assert_eq!(d, "4");
}

#[test]
fn new_string_has_length() {
    let mut context = Context::default();
    let init = r#"
        let a = new String("1234");
        a
        "#;

    forward(&mut context, init);
    assert_eq!(forward(&mut context, "a.length"), "4");
}

#[test]
fn new_string_has_length_not_enumerable() {
    let mut context = Context::default();
    let init = r#"
        let a = new String("1234");
        "#;

    forward(&mut context, init);
    assert_eq!(
        forward(&mut context, "a.propertyIsEnumerable('length')"),
        "false"
    );
}

#[test]
fn new_utf8_string_has_length() {
    let mut context = Context::default();
    let init = r#"
        let a = new String("中文");
        a
        "#;

    forward(&mut context, init);
    assert_eq!(forward(&mut context, "a.length"), "2");
}

#[test]
fn concat() {
    let mut context = Context::default();
    let init = r#"
        var hello = new String('Hello, ');
        var world = new String('world! ');
        var nice = new String('Have a nice day.');
        "#;
    eprintln!("{}", forward(&mut context, init));

    let a = forward(&mut context, "hello.concat(world, nice)");
    assert_eq!(a, "\"Hello, world! Have a nice day.\"");

    let b = forward(&mut context, "hello + world + nice");
    assert_eq!(b, "\"Hello, world! Have a nice day.\"");
}

#[test]
fn generic_concat() {
    let mut context = Context::default();
    let init = r#"
        Number.prototype.concat = String.prototype.concat;
        let number = new Number(100);
        "#;
    eprintln!("{}", forward(&mut context, init));

    let a = forward(&mut context, "number.concat(' - 50', ' = 50')");
    assert_eq!(a, "\"100 - 50 = 50\"");
}

#[allow(clippy::unwrap_used)]
#[test]
/// Test the correct type is returned from call and construct
fn construct_and_call() {
    let mut context = Context::default();
    let init = r#"
        var hello = new String('Hello');
        var world = String('world');
        "#;

    forward(&mut context, init);

    let hello = forward_val(&mut context, "hello").unwrap();
    let world = forward_val(&mut context, "world").unwrap();

    assert!(hello.is_object());
    assert!(world.is_string());
}

#[test]
fn repeat() {
    let mut context = Context::default();
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "empty.repeat(0)"), "\"\"");
    assert_eq!(forward(&mut context, "empty.repeat(1)"), "\"\"");

    assert_eq!(forward(&mut context, "en.repeat(0)"), "\"\"");
    assert_eq!(forward(&mut context, "zh.repeat(0)"), "\"\"");

    assert_eq!(forward(&mut context, "en.repeat(1)"), "\"english\"");
    assert_eq!(forward(&mut context, "zh.repeat(2)"), "\"中文中文\"");
}

#[test]
fn repeat_throws_when_count_is_negative() {
    let mut context = Context::default();

    assert_eq!(
        forward(
            &mut context,
            r#"
        try {
            'x'.repeat(-1)
        } catch (e) {
            e.toString()
        }
    "#
        ),
        "\"RangeError: repeat count must be a positive finite number \
        that doesn't overflow the maximum string length (2^32 - 1)\""
    );
}

#[test]
fn repeat_throws_when_count_is_infinity() {
    let mut context = Context::default();

    assert_eq!(
        forward(
            &mut context,
            r#"
        try {
            'x'.repeat(Infinity)
        } catch (e) {
            e.toString()
        }
    "#
        ),
        "\"RangeError: repeat count must be a positive finite number \
        that doesn't overflow the maximum string length (2^32 - 1)\""
    );
}

#[test]
fn repeat_throws_when_count_overflows_max_length() {
    let mut context = Context::default();

    assert_eq!(
        forward(
            &mut context,
            r#"
        try {
            'x'.repeat(2 ** 64)
        } catch (e) {
            e.toString()
        }
    "#
        ),
        "\"RangeError: repeat count must be a positive finite number \
        that doesn't overflow the maximum string length (2^32 - 1)\""
    );
}

#[test]
fn repeat_generic() {
    let mut context = Context::default();
    let init = "Number.prototype.repeat = String.prototype.repeat;";

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "(0).repeat(0)"), "\"\"");
    assert_eq!(forward(&mut context, "(1).repeat(1)"), "\"1\"");

    assert_eq!(forward(&mut context, "(1).repeat(5)"), "\"11111\"");
    assert_eq!(forward(&mut context, "(12).repeat(3)"), "\"121212\"");
}

#[test]
fn replace() {
    let mut context = Context::default();
    let init = r#"
        var a = "abc";
        a = a.replace("a", "2");
        a
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "a"), "\"2bc\"");
}

#[test]
fn replace_no_match() {
    let mut context = Context::default();
    let init = r#"
        var a = "abc";
        a = a.replace(/d/, "$&$&");
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "a"), "\"abc\"");
}

#[test]
fn replace_with_capture_groups() {
    let mut context = Context::default();
    let init = r#"
        var re = /(\w+)\s(\w+)/;
        var a = "John Smith";
        a = a.replace(re, '$2, $1');
        a
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "a"), "\"Smith, John\"");
}

#[test]
fn replace_with_tenth_capture_group() {
    let mut context = Context::default();
    let init = r#"
        var re = /(\d)(\d)(\d)(\d)(\d)(\d)(\d)(\d)(\d)(\d)/;
        var a = "0123456789";
        let res = a.replace(re, '$10');
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "res"), "\"9\"");
}

#[test]
fn replace_substitutions() {
    let mut context = Context::default();
    let init = r#"
        var re = / two /;
        var a = "one two three";
        var dollar = a.replace(re, " $$ ");
        var matched = a.replace(re, "$&$&");
        var start = a.replace(re, " $` ");
        var end = a.replace(re, " $' ");
        var no_sub = a.replace(re, " $_ ");
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "dollar"), "\"one $ three\"");
    assert_eq!(forward(&mut context, "matched"), "\"one two  two three\"");
    assert_eq!(forward(&mut context, "start"), "\"one one three\"");
    assert_eq!(forward(&mut context, "end"), "\"one three three\"");
    assert_eq!(forward(&mut context, "no_sub"), "\"one $_ three\"");
}

#[test]
fn replace_with_function() {
    let mut context = Context::default();
    let init = r#"
        var a = "ecmascript is cool";
        var p1, p2, p3, length;
        var replacer = (match, cap1, cap2, cap3, len) => {
            p1 = cap1;
            p2 = cap2;
            p3 = cap3;
            length = len;
            return "awesome!";
        };

        a = a.replace(/c(o)(o)(l)/, replacer);
        a;
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "a"), "\"ecmascript is awesome!\"");

    assert_eq!(forward(&mut context, "p1"), "\"o\"");
    assert_eq!(forward(&mut context, "p2"), "\"o\"");
    assert_eq!(forward(&mut context, "p3"), "\"l\"");
    assert_eq!(forward(&mut context, "length"), "14");
}

#[test]
fn starts_with() {
    let mut context = Context::default();
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');

        var emptyLiteral = '';
        var enLiteral = 'english';
        var zhLiteral = '中文';
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "empty.startsWith('')"), "true");
    assert_eq!(forward(&mut context, "en.startsWith('e')"), "true");
    assert_eq!(forward(&mut context, "zh.startsWith('中')"), "true");

    assert_eq!(forward(&mut context, "emptyLiteral.startsWith('')"), "true");
    assert_eq!(forward(&mut context, "enLiteral.startsWith('e')"), "true");
    assert_eq!(forward(&mut context, "zhLiteral.startsWith('中')"), "true");
}

#[test]
fn starts_with_with_regex_arg() {
    let mut context = Context::default();

    let scenario = r#"
        try {
            'Saturday night'.startsWith(/Saturday/);
        } catch (e) {
            e.toString();
        }
    "#;

    assert_eq!(
        forward(
            &mut context, scenario
        ),
        "\"TypeError: First argument to String.prototype.startsWith must not be a regular expression\""
    );
}

#[test]
fn ends_with() {
    let mut context = Context::default();
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');

        var emptyLiteral = '';
        var enLiteral = 'english';
        var zhLiteral = '中文';
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "empty.endsWith('')"), "true");
    assert_eq!(forward(&mut context, "en.endsWith('h')"), "true");
    assert_eq!(forward(&mut context, "zh.endsWith('文')"), "true");

    assert_eq!(forward(&mut context, "emptyLiteral.endsWith('')"), "true");
    assert_eq!(forward(&mut context, "enLiteral.endsWith('h')"), "true");
    assert_eq!(forward(&mut context, "zhLiteral.endsWith('文')"), "true");
}

#[test]
fn ends_with_with_regex_arg() {
    let mut context = Context::default();

    let scenario = r#"
        try {
            'Saturday night'.endsWith(/night/);
        } catch (e) {
            e.toString();
        }
    "#;

    assert_eq!(
        forward(
            &mut context, scenario
        ),
        "\"TypeError: First argument to String.prototype.endsWith must not be a regular expression\""
    );
}

#[test]
fn includes() {
    let mut context = Context::default();
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');

        var emptyLiteral = '';
        var enLiteral = 'english';
        var zhLiteral = '中文';
        "#;

    forward(&mut context, init);

    assert_eq!(forward(&mut context, "empty.includes('')"), "true");
    assert_eq!(forward(&mut context, "en.includes('g')"), "true");
    assert_eq!(forward(&mut context, "zh.includes('文')"), "true");

    assert_eq!(forward(&mut context, "emptyLiteral.includes('')"), "true");
    assert_eq!(forward(&mut context, "enLiteral.includes('g')"), "true");
    assert_eq!(forward(&mut context, "zhLiteral.includes('文')"), "true");
}

#[test]
fn includes_with_regex_arg() {
    let mut context = Context::default();

    let scenario = r#"
        try {
            'Saturday night'.includes(/day/);
        } catch (e) {
            e.toString();
        }
    "#;

    assert_eq!(
        forward(
            &mut context, scenario
        ),
        "\"TypeError: First argument to String.prototype.includes must not be a regular expression\""
    );
}

#[test]
fn match_all() {
    let mut context = Context::default();

    forward(
        &mut context,
        r#"
        var groupMatches = 'test1test2'.matchAll(/t(e)(st(\d?))/g);
        var m1 = groupMatches.next();
        var m2 = groupMatches.next();
        var m3 = groupMatches.next();
        "#,
    );

    assert_eq!(forward(&mut context, "m1.done"), "false");
    assert_eq!(forward(&mut context, "m2.done"), "false");
    assert_eq!(forward(&mut context, "m3.done"), "true");

    assert_eq!(forward(&mut context, "m1.value[0]"), "\"test1\"");
    assert_eq!(forward(&mut context, "m1.value[1]"), "\"e\"");
    assert_eq!(forward(&mut context, "m1.value[2]"), "\"st1\"");
    assert_eq!(forward(&mut context, "m1.value[3]"), "\"1\"");
    assert_eq!(forward(&mut context, "m1.value[4]"), "undefined");
    assert_eq!(forward(&mut context, "m1.value.index"), "0");
    assert_eq!(forward(&mut context, "m1.value.input"), "\"test1test2\"");
    assert_eq!(forward(&mut context, "m1.value.groups"), "undefined");

    assert_eq!(forward(&mut context, "m2.value[0]"), "\"test2\"");
    assert_eq!(forward(&mut context, "m2.value[1]"), "\"e\"");
    assert_eq!(forward(&mut context, "m2.value[2]"), "\"st2\"");
    assert_eq!(forward(&mut context, "m2.value[3]"), "\"2\"");
    assert_eq!(forward(&mut context, "m2.value[4]"), "undefined");
    assert_eq!(forward(&mut context, "m2.value.index"), "5");
    assert_eq!(forward(&mut context, "m2.value.input"), "\"test1test2\"");
    assert_eq!(forward(&mut context, "m2.value.groups"), "undefined");

    assert_eq!(forward(&mut context, "m3.value"), "undefined");

    forward(
        &mut context,
        r#"
        var regexp = RegExp('foo[a-z]*','g');
        var str = 'table football, foosball';
        var matches = str.matchAll(regexp);
        var m1 = matches.next();
        var m2 = matches.next();
        var m3 = matches.next();
        "#,
    );

    assert_eq!(forward(&mut context, "m1.done"), "false");
    assert_eq!(forward(&mut context, "m2.done"), "false");
    assert_eq!(forward(&mut context, "m3.done"), "true");

    assert_eq!(forward(&mut context, "m1.value[0]"), "\"football\"");
    assert_eq!(forward(&mut context, "m1.value[1]"), "undefined");
    assert_eq!(forward(&mut context, "m1.value.index"), "6");
    assert_eq!(
        forward(&mut context, "m1.value.input"),
        "\"table football, foosball\""
    );
    assert_eq!(forward(&mut context, "m1.value.groups"), "undefined");

    assert_eq!(forward(&mut context, "m2.value[0]"), "\"foosball\"");
    assert_eq!(forward(&mut context, "m2.value[1]"), "undefined");
    assert_eq!(forward(&mut context, "m2.value.index"), "16");
    assert_eq!(
        forward(&mut context, "m1.value.input"),
        "\"table football, foosball\""
    );
    assert_eq!(forward(&mut context, "m2.value.groups"), "undefined");

    assert_eq!(forward(&mut context, "m3.value"), "undefined");
}

#[test]
fn test_match() {
    let mut context = Context::default();
    let init = r#"
        var str = new String('The Quick Brown Fox Jumps Over The Lazy Dog');
        var result1 = str.match(/quick\s(brown).+?(jumps)/i);
        var result2 = str.match(/[A-Z]/g);
        var result3 = str.match("T");
        var result4 = str.match(RegExp("B", 'g'));
        "#;

    forward(&mut context, init);

    assert_eq!(
        forward(&mut context, "result1[0]"),
        "\"Quick Brown Fox Jumps\""
    );
    assert_eq!(forward(&mut context, "result1[1]"), "\"Brown\"");
    assert_eq!(forward(&mut context, "result1[2]"), "\"Jumps\"");
    assert_eq!(forward(&mut context, "result1.index"), "4");
    assert_eq!(
        forward(&mut context, "result1.input"),
        "\"The Quick Brown Fox Jumps Over The Lazy Dog\""
    );

    assert_eq!(forward(&mut context, "result2[0]"), "\"T\"");
    assert_eq!(forward(&mut context, "result2[1]"), "\"Q\"");
    assert_eq!(forward(&mut context, "result2[2]"), "\"B\"");
    assert_eq!(forward(&mut context, "result2[3]"), "\"F\"");
    assert_eq!(forward(&mut context, "result2[4]"), "\"J\"");
    assert_eq!(forward(&mut context, "result2[5]"), "\"O\"");
    assert_eq!(forward(&mut context, "result2[6]"), "\"T\"");
    assert_eq!(forward(&mut context, "result2[7]"), "\"L\"");
    assert_eq!(forward(&mut context, "result2[8]"), "\"D\"");

    assert_eq!(forward(&mut context, "result3[0]"), "\"T\"");
    assert_eq!(forward(&mut context, "result3.index"), "0");
    assert_eq!(
        forward(&mut context, "result3.input"),
        "\"The Quick Brown Fox Jumps Over The Lazy Dog\""
    );
    assert_eq!(forward(&mut context, "result4[0]"), "\"B\"");
}

#[test]
fn trim() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, r#"'Hello'.trim()"#), "\"Hello\"");
    assert_eq!(forward(&mut context, r#"' \nHello'.trim()"#), "\"Hello\"");
    assert_eq!(forward(&mut context, r#"'Hello \n\r'.trim()"#), "\"Hello\"");
    assert_eq!(forward(&mut context, r#"' Hello '.trim()"#), "\"Hello\"");
}

#[test]
fn trim_start() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, r#"'Hello'.trimStart()"#), "\"Hello\"");
    assert_eq!(
        forward(&mut context, r#"' \nHello'.trimStart()"#),
        "\"Hello\""
    );
    assert_eq!(
        forward(&mut context, r#"'Hello \n'.trimStart()"#),
        "\"Hello \n\""
    );
    assert_eq!(
        forward(&mut context, r#"' Hello '.trimStart()"#),
        "\"Hello \""
    );
}

#[test]
fn trim_end() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, r#"'Hello'.trimEnd()"#), "\"Hello\"");
    assert_eq!(
        forward(&mut context, r#"' \nHello'.trimEnd()"#),
        "\" \nHello\""
    );
    assert_eq!(
        forward(&mut context, r#"'Hello \n'.trimEnd()"#),
        "\"Hello\""
    );
    assert_eq!(
        forward(&mut context, r#"' Hello '.trimEnd()"#),
        "\" Hello\""
    );
}

#[test]
fn split() {
    let mut context = Context::default();
    assert_eq!(
        forward(&mut context, "'Hello'.split()"),
        forward(&mut context, "['Hello']")
    );
    assert_eq!(
        forward(&mut context, "'Hello'.split(null)"),
        forward(&mut context, "['Hello']")
    );
    assert_eq!(
        forward(&mut context, "'Hello'.split(undefined)"),
        forward(&mut context, "['Hello']")
    );
    assert_eq!(
        forward(&mut context, "'Hello'.split('')"),
        forward(&mut context, "['H','e','l','l','o']")
    );

    assert_eq!(
        forward(&mut context, "'x1x2'.split('x')"),
        forward(&mut context, "['','1','2']")
    );
    assert_eq!(
        forward(&mut context, "'x1x2x'.split('x')"),
        forward(&mut context, "['','1','2','']")
    );

    assert_eq!(
        forward(&mut context, "'x1x2x'.split('x', 0)"),
        forward(&mut context, "[]")
    );
    assert_eq!(
        forward(&mut context, "'x1x2x'.split('x', 2)"),
        forward(&mut context, "['','1']")
    );
    assert_eq!(
        forward(&mut context, "'x1x2x'.split('x', 10)"),
        forward(&mut context, "['','1','2','']")
    );

    assert_eq!(
        forward(&mut context, "'x1x2x'.split(1)"),
        forward(&mut context, "['x','x2x']")
    );

    assert_eq!(
        forward(&mut context, "'Hello'.split(null, 0)"),
        forward(&mut context, "[]")
    );
    assert_eq!(
        forward(&mut context, "'Hello'.split(undefined, 0)"),
        forward(&mut context, "[]")
    );

    assert_eq!(
        forward(&mut context, "''.split()"),
        forward(&mut context, "['']")
    );
    assert_eq!(
        forward(&mut context, "''.split(undefined)"),
        forward(&mut context, "['']")
    );
    assert_eq!(
        forward(&mut context, "''.split('')"),
        forward(&mut context, "[]")
    );
    assert_eq!(
        forward(&mut context, "''.split('1')"),
        forward(&mut context, "['']")
    );

    assert_eq!(
        forward(
            &mut context,
            "\'\u{1d7d8}\u{1d7d9}\u{1d7da}\u{1d7db}\'.split(\'\')"
        ),
        // TODO: modify interner to store UTF-16 surrogates from string literals
        // forward(&mut context, "['�','�','�','�','�','�','�','�']")
        "[ \"\\uD835\", \"\\uDFD8\", \"\\uD835\", \"\\uDFD9\", \"\\uD835\", \"\\uDFDA\", \"\\uD835\", \"\\uDFDB\" ]"
    );
}

#[test]
fn split_with_symbol_split_method() {
    assert_eq!(
        forward(
            &mut Context::default(),
            r#"
            let sep = {};
            sep[Symbol.split] = function(s, limit) { return s + limit.toString(); };
            'hello'.split(sep, 10)
        "#
        ),
        "\"hello10\""
    );

    assert_eq!(
        forward(
            &mut Context::default(),
            r#"
            let sep = {};
            sep[Symbol.split] = undefined;
            'hello'.split(sep)
        "#
        ),
        "[ \"hello\" ]"
    );

    assert_eq!(
        forward(
            &mut Context::default(),
            r#"
            try {
                let sep = {};
                sep[Symbol.split] = 10;
                'hello'.split(sep, 10);
            } catch(e) {
                e.toString()
            }
        "#
        ),
        "\"TypeError: value returned for property of object is not a function\""
    );
}

#[test]
fn index_of_with_no_arguments() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.indexOf()"), "-1");
    assert_eq!(forward(&mut context, "'undefined'.indexOf()"), "0");
    assert_eq!(forward(&mut context, "'a1undefined'.indexOf()"), "2");
    assert_eq!(forward(&mut context, "'a1undefined1a'.indexOf()"), "2");
    assert_eq!(forward(&mut context, "'µµµundefined'.indexOf()"), "3");
    assert_eq!(forward(&mut context, "'µµµundefinedµµµ'.indexOf()"), "3");
}

#[test]
fn index_of_with_string_search_string_argument() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.indexOf('hello')"), "-1");
    assert_eq!(
        forward(&mut context, "'undefined'.indexOf('undefined')"),
        "0"
    );
    assert_eq!(
        forward(&mut context, "'a1undefined'.indexOf('undefined')"),
        "2"
    );
    assert_eq!(
        forward(&mut context, "'a1undefined1a'.indexOf('undefined')"),
        "2"
    );
    assert_eq!(
        forward(&mut context, "'µµµundefined'.indexOf('undefined')"),
        "3"
    );
    assert_eq!(
        forward(&mut context, "'µµµundefinedµµµ'.indexOf('undefined')"),
        "3"
    );
}

#[test]
fn index_of_with_non_string_search_string_argument() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.indexOf(1)"), "-1");
    assert_eq!(forward(&mut context, "'1'.indexOf(1)"), "0");
    assert_eq!(forward(&mut context, "'true'.indexOf(true)"), "0");
    assert_eq!(forward(&mut context, "'ab100ba'.indexOf(100)"), "2");
    assert_eq!(forward(&mut context, "'µµµfalse'.indexOf(true)"), "-1");
    assert_eq!(forward(&mut context, "'µµµ5µµµ'.indexOf(5)"), "3");
}

#[test]
fn index_of_with_from_index_argument() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.indexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut context, "'x'.indexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut context, "'abcx'.indexOf('x', 2)"), "3");
    assert_eq!(forward(&mut context, "'x'.indexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut context, "'µµµxµµµ'.indexOf('x', 2)"), "3");

    assert_eq!(
        forward(&mut context, "'µµµxµµµ'.indexOf('x', 10000000)"),
        "-1"
    );
}

#[test]
fn generic_index_of() {
    let mut context = Context::default();
    forward_val(
        &mut context,
        "Number.prototype.indexOf = String.prototype.indexOf",
    )
    .unwrap();

    assert_eq!(forward(&mut context, "(10).indexOf(9)"), "-1");
    assert_eq!(forward(&mut context, "(10).indexOf(0)"), "1");
    assert_eq!(forward(&mut context, "(10).indexOf('0')"), "1");
}

#[test]
fn index_of_empty_search_string() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "''.indexOf('')"), "0");
    assert_eq!(forward(&mut context, "''.indexOf('', 10)"), "0");
    assert_eq!(forward(&mut context, "'ABC'.indexOf('', 1)"), "1");
    assert_eq!(forward(&mut context, "'ABC'.indexOf('', 2)"), "2");
    assert_eq!(forward(&mut context, "'ABC'.indexOf('', 10)"), "3");
}

#[test]
fn last_index_of_with_no_arguments() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.lastIndexOf()"), "-1");
    assert_eq!(forward(&mut context, "'undefined'.lastIndexOf()"), "0");
    assert_eq!(forward(&mut context, "'a1undefined'.lastIndexOf()"), "2");
    assert_eq!(
        forward(&mut context, "'a1undefined1aundefined'.lastIndexOf()"),
        "13"
    );
    assert_eq!(
        forward(&mut context, "'µµµundefinedundefined'.lastIndexOf()"),
        "12"
    );
    assert_eq!(
        forward(&mut context, "'µµµundefinedµµµundefined'.lastIndexOf()"),
        "15"
    );
}

#[test]
fn last_index_of_with_string_search_string_argument() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.lastIndexOf('hello')"), "-1");
    assert_eq!(
        forward(&mut context, "'undefined'.lastIndexOf('undefined')"),
        "0"
    );
    assert_eq!(
        forward(&mut context, "'a1undefined'.lastIndexOf('undefined')"),
        "2"
    );
    assert_eq!(
        forward(
            &mut context,
            "'a1undefined1aundefined'.lastIndexOf('undefined')"
        ),
        "13"
    );
    assert_eq!(
        forward(
            &mut context,
            "'µµµundefinedundefined'.lastIndexOf('undefined')"
        ),
        "12"
    );
    assert_eq!(
        forward(
            &mut context,
            "'µµµundefinedµµµundefined'.lastIndexOf('undefined')"
        ),
        "15"
    );
}

#[test]
fn last_index_of_with_non_string_search_string_argument() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.lastIndexOf(1)"), "-1");
    assert_eq!(forward(&mut context, "'1'.lastIndexOf(1)"), "0");
    assert_eq!(forward(&mut context, "'11'.lastIndexOf(1)"), "1");
    assert_eq!(
        forward(&mut context, "'truefalsetrue'.lastIndexOf(true)"),
        "9"
    );
    assert_eq!(forward(&mut context, "'ab100ba'.lastIndexOf(100)"), "2");
    assert_eq!(forward(&mut context, "'µµµfalse'.lastIndexOf(true)"), "-1");
    assert_eq!(forward(&mut context, "'µµµ5µµµ65µ'.lastIndexOf(5)"), "8");
}

#[test]
fn last_index_of_with_from_index_argument() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.lastIndexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut context, "'x'.lastIndexOf('x', 2)"), "0");
    assert_eq!(forward(&mut context, "'abcxx'.lastIndexOf('x', 2)"), "-1");
    assert_eq!(forward(&mut context, "'x'.lastIndexOf('x', 2)"), "0");
    assert_eq!(forward(&mut context, "'µµµxµµµ'.lastIndexOf('x', 2)"), "-1");

    assert_eq!(
        forward(&mut context, "'µµµxµµµ'.lastIndexOf('x', 10000000)"),
        "3"
    );
}

#[test]
fn last_index_with_empty_search_string() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "''.lastIndexOf('')"), "0");
    assert_eq!(forward(&mut context, "'x'.lastIndexOf('', 2)"), "1");
    assert_eq!(forward(&mut context, "'abcxx'.lastIndexOf('', 4)"), "4");
    assert_eq!(forward(&mut context, "'µµµxµµµ'.lastIndexOf('', 2)"), "2");

    assert_eq!(
        forward(&mut context, "'abc'.lastIndexOf('', 10000000)"),
        "3"
    );
}

#[test]
fn generic_last_index_of() {
    let mut context = Context::default();
    forward_val(
        &mut context,
        "Number.prototype.lastIndexOf = String.prototype.lastIndexOf",
    )
    .unwrap();

    assert_eq!(forward(&mut context, "(1001).lastIndexOf(9)"), "-1");
    assert_eq!(forward(&mut context, "(1001).lastIndexOf(0)"), "2");
    assert_eq!(forward(&mut context, "(1001).lastIndexOf('0')"), "2");
}

#[test]
fn last_index_non_integer_position_argument() {
    let mut context = Context::default();
    assert_eq!(
        forward(&mut context, "''.lastIndexOf('x', new Number(4))"),
        "-1"
    );
    assert_eq!(
        forward(&mut context, "'abc'.lastIndexOf('b', new Number(1))"),
        "1"
    );
    assert_eq!(
        forward(&mut context, "'abcx'.lastIndexOf('x', new String('1'))"),
        "-1"
    );
    assert_eq!(
        forward(&mut context, "'abcx'.lastIndexOf('x', new String('100'))"),
        "3"
    );
    assert_eq!(forward(&mut context, "'abcx'.lastIndexOf('x', null)"), "-1");
}

#[test]
fn char_at() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "'abc'.charAt(-1)"), "\"\"");
    assert_eq!(forward(&mut context, "'abc'.charAt(1)"), "\"b\"");
    assert_eq!(forward(&mut context, "'abc'.charAt(9)"), "\"\"");
    assert_eq!(forward(&mut context, "'abc'.charAt()"), "\"a\"");
    assert_eq!(forward(&mut context, "'abc'.charAt(null)"), "\"a\"");
    assert_eq!(forward(&mut context, "'\\uDBFF'.charAt(0)"), r#""\uDBFF""#);
}

#[test]
fn char_code_at() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "'abc'.charCodeAt(-1)"), "NaN");
    assert_eq!(forward(&mut context, "'abc'.charCodeAt(1)"), "98");
    assert_eq!(forward(&mut context, "'abc'.charCodeAt(9)"), "NaN");
    assert_eq!(forward(&mut context, "'abc'.charCodeAt()"), "97");
    assert_eq!(forward(&mut context, "'abc'.charCodeAt(null)"), "97");
    assert_eq!(forward(&mut context, "'\\uFFFF'.charCodeAt(0)"), "65535");
}

#[test]
fn code_point_at() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "'abc'.codePointAt(-1)"), "undefined");
    assert_eq!(forward(&mut context, "'abc'.codePointAt(1)"), "98");
    assert_eq!(forward(&mut context, "'abc'.codePointAt(9)"), "undefined");
    assert_eq!(forward(&mut context, "'abc'.codePointAt()"), "97");
    assert_eq!(forward(&mut context, "'abc'.codePointAt(null)"), "97");
    assert_eq!(
        forward(&mut context, "'\\uD800\\uDC00'.codePointAt(0)"),
        "65536"
    );
    assert_eq!(
        forward(&mut context, "'\\uD800\\uDFFF'.codePointAt(0)"),
        "66559"
    );
    assert_eq!(
        forward(&mut context, "'\\uDBFF\\uDC00'.codePointAt(0)"),
        "1113088"
    );
    assert_eq!(
        forward(&mut context, "'\\uDBFF\\uDFFF'.codePointAt(0)"),
        "1114111"
    );
    assert_eq!(
        forward(&mut context, "'\\uD800\\uDC00'.codePointAt(1)"),
        "56320"
    );
    assert_eq!(
        forward(&mut context, "'\\uD800\\uDFFF'.codePointAt(1)"),
        "57343"
    );
    assert_eq!(
        forward(&mut context, "'\\uDBFF\\uDC00'.codePointAt(1)"),
        "56320"
    );
    assert_eq!(
        forward(&mut context, "'\\uDBFF\\uDFFF'.codePointAt(1)"),
        "57343"
    );
}

#[test]
fn slice() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "'abc'.slice()"), "\"abc\"");
    assert_eq!(forward(&mut context, "'abc'.slice(1)"), "\"bc\"");
    assert_eq!(forward(&mut context, "'abc'.slice(-1)"), "\"c\"");
    assert_eq!(forward(&mut context, "'abc'.slice(0, 9)"), "\"abc\"");
    assert_eq!(forward(&mut context, "'abc'.slice(9, 10)"), "\"\"");
}

#[test]
fn empty_iter() {
    let mut context = Context::default();
    let init = r#"
        let iter = new String()[Symbol.iterator]();
        let next = iter.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn ascii_iter() {
    let mut context = Context::default();
    let init = r#"
        let iter = new String("Hello World")[Symbol.iterator]();
        let next = iter.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "\"H\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"e\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"l\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"l\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"o\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\" \"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"W\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"o\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"r\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"l\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"d\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn unicode_iter() {
    let mut context = Context::default();
    let init = r#"
        let iter = new String("C🙂🙂l W🙂rld")[Symbol.iterator]();
        let next = iter.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "\"C\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"🙂\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"🙂\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"l\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\" \"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"W\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"🙂\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"r\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"l\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "\"d\"");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iter.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn string_get_property() {
    let mut context = Context::default();
    assert_eq!(forward(&mut context, "'abc'[-1]"), "undefined");
    assert_eq!(forward(&mut context, "'abc'[1]"), "\"b\"");
    assert_eq!(forward(&mut context, "'abc'[2]"), "\"c\"");
    assert_eq!(forward(&mut context, "'abc'[3]"), "undefined");
    assert_eq!(forward(&mut context, "'abc'['foo']"), "undefined");
    assert_eq!(forward(&mut context, "'😀'[0]"), "\"\\uD83D\"");
}

#[test]
fn search() {
    let mut context = Context::default();

    assert_eq!(forward(&mut context, "'aa'.search(/b/)"), "-1");
    assert_eq!(forward(&mut context, "'aa'.search(/a/)"), "0");
    assert_eq!(forward(&mut context, "'aa'.search(/a/g)"), "0");
    assert_eq!(forward(&mut context, "'ba'.search(/a/)"), "1");
}
