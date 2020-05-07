use super::*;
use crate::exec::Executor;
use crate::realm::Realm;
use crate::{forward, forward_val};

#[test]
fn check_string_constructor_is_function() {
    let global = ValueData::new_obj(None);
    let string_constructor = create(&global);
    assert_eq!(string_constructor.is_function(), true);
}

// #[test]
// TODO: re-enable when getProperty() is finished;
// fn length() {
//     //TEST262: https://github.com/tc39/test262/blob/master/test/built-ins/String/length.js
//     let mut engine = Executor::new();
//     let init = r#"
//     const a = new String(' ');
//     const b = new String('\ud834\udf06');
//     const c = new String(' \b ');
//     cosnt d = new String('中文长度')
//     "#;
//     eprintln!("{}", forward(&mut engine, init));
//     let a = forward(&mut engine, "a.length");
//     assert_eq!(a, String::from("1"));
//     let b = forward(&mut engine, "b.length");
//     // TODO: fix this
//     // unicode surrogate pair length should be 1
//     // utf16/usc2 length should be 2
//     // utf8 length should be 4
//     //assert_eq!(b, String::from("2"));
//     let c = forward(&mut engine, "c.length");
//     assert_eq!(c, String::from("3"));
//     let d = forward(&mut engine, "d.length");
//     assert_eq!(d, String::from("4"));
// }

#[test]
fn concat() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var hello = new String('Hello, ');
        var world = new String('world! ');
        var nice = new String('Have a nice day.');
        "#;
    eprintln!("{}", forward(&mut engine, init));

    // Todo: fix this
    let _a = forward(&mut engine, "hello.concat(world, nice)");
    let _b = forward(&mut engine, "hello + world + nice");
    // assert_eq!(a, String::from("Hello, world! Have a nice day."));
    // assert_eq!(b, String::from("Hello, world! Have a nice day."));
}

#[allow(clippy::result_unwrap_used)]
#[test]
/// Test the correct type is returned from call and construct
fn construct_and_call() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var hello = new String('Hello');
        var world = String('world');
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let hello = forward_val(&mut engine, "hello").unwrap();
    let world = forward_val(&mut engine, "world").unwrap();

    assert_eq!(hello.is_object(), true);
    assert_eq!(world.is_string(), true);
}

#[test]
fn repeat() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');
        "#;
    eprintln!("{}", forward(&mut engine, init));

    let empty = String::from("");
    assert_eq!(forward(&mut engine, "empty.repeat(0)"), empty);
    assert_eq!(forward(&mut engine, "empty.repeat(1)"), empty);

    assert_eq!(forward(&mut engine, "en.repeat(0)"), empty);
    assert_eq!(forward(&mut engine, "zh.repeat(0)"), empty);

    assert_eq!(
        forward(&mut engine, "en.repeat(1)"),
        String::from("english")
    );
    assert_eq!(
        forward(&mut engine, "zh.repeat(2)"),
        String::from("中文中文")
    );
}

#[test]
fn replace() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var a = "abc";
        a = a.replace("a", "2");
        a
        "#;
    eprintln!("{}", forward(&mut engine, init));

    let empty = String::from("2bc");
    assert_eq!(forward(&mut engine, "a"), empty);
}

#[test]
fn replace_with_function() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
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
    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(
        forward(&mut engine, "a"),
        String::from("ecmascript is awesome!")
    );

    assert_eq!(forward(&mut engine, "p1"), String::from("o"));
    assert_eq!(forward(&mut engine, "p2"), String::from("o"));
    assert_eq!(forward(&mut engine, "p3"), String::from("l"));
}

#[test]
fn starts_with() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');

        var emptyLiteral = '';
        var enLiteral = 'english';
        var zhLiteral = '中文';
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let pass = String::from("true");
    assert_eq!(forward(&mut engine, "empty.startsWith('')"), pass);
    assert_eq!(forward(&mut engine, "en.startsWith('e')"), pass);
    assert_eq!(forward(&mut engine, "zh.startsWith('中')"), pass);

    assert_eq!(forward(&mut engine, "emptyLiteral.startsWith('')"), pass);
    assert_eq!(forward(&mut engine, "enLiteral.startsWith('e')"), pass);
    assert_eq!(forward(&mut engine, "zhLiteral.startsWith('中')"), pass);
}

#[test]
fn ends_with() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var empty = new String('');
        var en = new String('english');
        var zh = new String('中文');

        var emptyLiteral = '';
        var enLiteral = 'english';
        var zhLiteral = '中文';
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let pass = String::from("true");
    assert_eq!(forward(&mut engine, "empty.endsWith('')"), pass);
    assert_eq!(forward(&mut engine, "en.endsWith('h')"), pass);
    assert_eq!(forward(&mut engine, "zh.endsWith('文')"), pass);

    assert_eq!(forward(&mut engine, "emptyLiteral.endsWith('')"), pass);
    assert_eq!(forward(&mut engine, "enLiteral.endsWith('h')"), pass);
    assert_eq!(forward(&mut engine, "zhLiteral.endsWith('文')"), pass);
}

#[test]
fn match_all() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);

    assert_eq!(
        forward(&mut engine, "'aa'.matchAll(null).length"),
        String::from("0")
    );
    assert_eq!(
        forward(&mut engine, "'aa'.matchAll(/b/).length"),
        String::from("0")
    );
    assert_eq!(
        forward(&mut engine, "'aa'.matchAll(/a/).length"),
        String::from("1")
    );
    assert_eq!(
        forward(&mut engine, "'aa'.matchAll(/a/g).length"),
        String::from("2")
    );

    forward(
        &mut engine,
        "var groupMatches = 'test1test2'.matchAll(/t(e)(st(\\d?))/g)",
    );
    assert_eq!(
        forward(&mut engine, "groupMatches.length"),
        String::from("2")
    );
    assert_eq!(
        forward(&mut engine, "groupMatches[0][1]"),
        String::from("e")
    );
    assert_eq!(
        forward(&mut engine, "groupMatches[0][2]"),
        String::from("st1")
    );
    assert_eq!(
        forward(&mut engine, "groupMatches[0][3]"),
        String::from("1")
    );
    assert_eq!(
        forward(&mut engine, "groupMatches[1][3]"),
        String::from("2")
    );

    assert_eq!(
        forward(
            &mut engine,
            "'test1test2'.matchAll(/t(e)(st(\\d?))/).length"
        ),
        String::from("1")
    );

    let init = r#"
        var regexp = RegExp('foo[a-z]*','g');
        var str = 'table football, foosball';
        var matches = str.matchAll(regexp);
        "#;
    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(
        forward(&mut engine, "matches[0][0]"),
        String::from("football")
    );
    assert_eq!(forward(&mut engine, "matches[0].index"), String::from("6"));
    assert_eq!(
        forward(&mut engine, "matches[1][0]"),
        String::from("foosball")
    );
    assert_eq!(forward(&mut engine, "matches[1].index"), String::from("16"));
}

#[test]
fn test_match() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var str = new String('The Quick Brown Fox Jumps Over The Lazy Dog');
        var result1 = str.match(/quick\s(brown).+?(jumps)/i);
        var result2 = str.match(/[A-Z]/g);
        var result3 = str.match("T");
        var result4 = str.match(RegExp("B", 'g'));
        "#;

    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(forward(&mut engine, "result1[0]"), "Quick Brown Fox Jumps");
    assert_eq!(forward(&mut engine, "result1[1]"), "Brown");
    assert_eq!(forward(&mut engine, "result1[2]"), "Jumps");
    assert_eq!(forward(&mut engine, "result1.index"), "4");
    assert_eq!(
        forward(&mut engine, "result1.input"),
        "The Quick Brown Fox Jumps Over The Lazy Dog"
    );

    assert_eq!(forward(&mut engine, "result2[0]"), "T");
    assert_eq!(forward(&mut engine, "result2[1]"), "Q");
    assert_eq!(forward(&mut engine, "result2[2]"), "B");
    assert_eq!(forward(&mut engine, "result2[3]"), "F");
    assert_eq!(forward(&mut engine, "result2[4]"), "J");
    assert_eq!(forward(&mut engine, "result2[5]"), "O");
    assert_eq!(forward(&mut engine, "result2[6]"), "T");
    assert_eq!(forward(&mut engine, "result2[7]"), "L");
    assert_eq!(forward(&mut engine, "result2[8]"), "D");

    assert_eq!(forward(&mut engine, "result3[0]"), "T");
    assert_eq!(forward(&mut engine, "result3.index"), "0");
    assert_eq!(
        forward(&mut engine, "result3.input"),
        "The Quick Brown Fox Jumps Over The Lazy Dog"
    );
    assert_eq!(forward(&mut engine, "result4[0]"), "B");
}
