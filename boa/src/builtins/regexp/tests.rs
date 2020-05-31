use super::*;
use crate::{exec::Interpreter, forward, realm::Realm};

#[test]
fn constructors() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var constructed = new RegExp("[0-9]+(\\.[0-9]+)?");
        var literal = /[0-9]+(\.[0-9]+)?/;
        var ctor_literal = new RegExp(/[0-9]+(\.[0-9]+)?/);
        "#;

    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(forward(&mut engine, "constructed.test('1.0')"), "true");
    assert_eq!(forward(&mut engine, "literal.test('1.0')"), "true");
    assert_eq!(forward(&mut engine, "ctor_literal.test('1.0')"), "true");
}

#[test]
fn check_regexp_constructor_is_function() {
    let global = Value::new_object(None);
    let regexp_constructor = RegExp::create(&global);
    assert_eq!(regexp_constructor.is_function(), true);
}

// TODO: uncomment this test when property getters are supported

//    #[test]
//    fn flags() {
//        let mut engine = Interpreter::new();
//        let init = r#"
//                var re_gi = /test/gi;
//                var re_sm = /test/sm;
//                "#;
//
//        eprintln!("{}", forward(&mut engine, init));
//        assert_eq!(forward(&mut engine, "re_gi.global"), "true");
//        assert_eq!(forward(&mut engine, "re_gi.ignoreCase"), "true");
//        assert_eq!(forward(&mut engine, "re_gi.multiline"), "false");
//        assert_eq!(forward(&mut engine, "re_gi.dotAll"), "false");
//        assert_eq!(forward(&mut engine, "re_gi.unicode"), "false");
//        assert_eq!(forward(&mut engine, "re_gi.sticky"), "false");
//        assert_eq!(forward(&mut engine, "re_gi.flags"), "gi");
//
//        assert_eq!(forward(&mut engine, "re_sm.global"), "false");
//        assert_eq!(forward(&mut engine, "re_sm.ignoreCase"), "false");
//        assert_eq!(forward(&mut engine, "re_sm.multiline"), "true");
//        assert_eq!(forward(&mut engine, "re_sm.dotAll"), "true");
//        assert_eq!(forward(&mut engine, "re_sm.unicode"), "false");
//        assert_eq!(forward(&mut engine, "re_sm.sticky"), "false");
//        assert_eq!(forward(&mut engine, "re_sm.flags"), "ms");
//    }

#[test]
fn last_index() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var regex = /[0-9]+(\.[0-9]+)?/g;
        "#;

    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(forward(&mut engine, "regex.lastIndex"), "0");
    assert_eq!(forward(&mut engine, "regex.test('1.0foo')"), "true");
    assert_eq!(forward(&mut engine, "regex.lastIndex"), "3");
    assert_eq!(forward(&mut engine, "regex.test('1.0foo')"), "false");
    assert_eq!(forward(&mut engine, "regex.lastIndex"), "0");
}

#[test]
fn exec() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var re = /quick\s(brown).+?(jumps)/ig;
        var result = re.exec('The Quick Brown Fox Jumps Over The Lazy Dog');
        "#;

    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(forward(&mut engine, "result[0]"), "Quick Brown Fox Jumps");
    assert_eq!(forward(&mut engine, "result[1]"), "Brown");
    assert_eq!(forward(&mut engine, "result[2]"), "Jumps");
    assert_eq!(forward(&mut engine, "result.index"), "4");
    assert_eq!(
        forward(&mut engine, "result.input"),
        "The Quick Brown Fox Jumps Over The Lazy Dog"
    );
}

#[test]
fn to_string() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(
        forward(&mut engine, "(new RegExp('a+b+c')).toString()"),
        "/a+b+c/"
    );
    assert_eq!(
        forward(&mut engine, "(new RegExp('bar', 'g')).toString()"),
        "/bar/g"
    );
    assert_eq!(
        forward(&mut engine, "(new RegExp('\\\\n', 'g')).toString()"),
        "/\\n/g"
    );
    assert_eq!(forward(&mut engine, "/\\n/g.toString()"), "/\\n/g");
}
