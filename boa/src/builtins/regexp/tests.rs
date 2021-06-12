use crate::{forward, Context};

#[test]
fn constructors() {
    let mut context = Context::new();
    let init = r#"
        var constructed = new RegExp("[0-9]+(\\.[0-9]+)?");
        var literal = /[0-9]+(\.[0-9]+)?/;
        var ctor_literal = new RegExp(/[0-9]+(\.[0-9]+)?/);
        "#;

    eprintln!("{}", forward(&mut context, init));
    assert_eq!(forward(&mut context, "constructed.test('1.0')"), "true");
    assert_eq!(forward(&mut context, "literal.test('1.0')"), "true");
    assert_eq!(forward(&mut context, "ctor_literal.test('1.0')"), "true");
}

// TODO: uncomment this test when property getters are supported

//    #[test]
//    fn flags() {
//        let mut context = Context::new();
//        let init = r#"
//                var re_gi = /test/gi;
//                var re_sm = /test/sm;
//                "#;
//
//        eprintln!("{}", forward(&mut context, init));
//        assert_eq!(forward(&mut context, "re_gi.global"), "true");
//        assert_eq!(forward(&mut context, "re_gi.ignoreCase"), "true");
//        assert_eq!(forward(&mut context, "re_gi.multiline"), "false");
//        assert_eq!(forward(&mut context, "re_gi.dotAll"), "false");
//        assert_eq!(forward(&mut context, "re_gi.unicode"), "false");
//        assert_eq!(forward(&mut context, "re_gi.sticky"), "false");
//        assert_eq!(forward(&mut context, "re_gi.flags"), "gi");
//
//        assert_eq!(forward(&mut context, "re_sm.global"), "false");
//        assert_eq!(forward(&mut context, "re_sm.ignoreCase"), "false");
//        assert_eq!(forward(&mut context, "re_sm.multiline"), "true");
//        assert_eq!(forward(&mut context, "re_sm.dotAll"), "true");
//        assert_eq!(forward(&mut context, "re_sm.unicode"), "false");
//        assert_eq!(forward(&mut context, "re_sm.sticky"), "false");
//        assert_eq!(forward(&mut context, "re_sm.flags"), "ms");
//    }

#[test]
fn last_index() {
    let mut context = Context::new();
    let init = r#"
        var regex = /[0-9]+(\.[0-9]+)?/g;
        "#;

    eprintln!("{}", forward(&mut context, init));
    assert_eq!(forward(&mut context, "regex.lastIndex"), "0");
    assert_eq!(forward(&mut context, "regex.test('1.0foo')"), "true");
    assert_eq!(forward(&mut context, "regex.lastIndex"), "3");
    assert_eq!(forward(&mut context, "regex.test('1.0foo')"), "false");
    assert_eq!(forward(&mut context, "regex.lastIndex"), "0");
}

#[test]
fn exec() {
    let mut context = Context::new();
    let init = r#"
        var re = /quick\s(brown).+?(jumps)/ig;
        var result = re.exec('The Quick Brown Fox Jumps Over The Lazy Dog');
        "#;

    eprintln!("{}", forward(&mut context, init));
    assert_eq!(
        forward(&mut context, "result[0]"),
        "\"Quick Brown Fox Jumps\""
    );
    assert_eq!(forward(&mut context, "result[1]"), "\"Brown\"");
    assert_eq!(forward(&mut context, "result[2]"), "\"Jumps\"");
    assert_eq!(forward(&mut context, "result.index"), "4");
    assert_eq!(
        forward(&mut context, "result.input"),
        "\"The Quick Brown Fox Jumps Over The Lazy Dog\""
    );
}

#[test]
fn to_string() {
    let mut context = Context::new();

    assert_eq!(
        forward(&mut context, "(new RegExp('a+b+c')).toString()"),
        "\"/a+b+c/\""
    );
    assert_eq!(
        forward(&mut context, "(new RegExp('bar', 'g')).toString()"),
        "\"/bar/g\""
    );
    assert_eq!(
        forward(&mut context, "(new RegExp('\\\\n', 'g')).toString()"),
        "\"/\\n/g\""
    );
    assert_eq!(forward(&mut context, "/\\n/g.toString()"), "\"/\\n/g\"");
}

#[test]
fn no_panic_on_invalid_character_escape() {
    let mut context = Context::new();

    // This used to panic, we now return an error
    // The line below should not cause Boa to panic
    forward(&mut context, r"const a = /,\;/");
}

#[test]
fn search() {
    let mut context = Context::new();

    assert_eq!(forward(&mut context, "/a/[Symbol.search](\"a\")"), "0");
    assert_eq!(forward(&mut context, "/a/[Symbol.search](\"ba\")"), "1");
    assert_eq!(forward(&mut context, "/a/[Symbol.search](\"bb\")"), "-1");
    assert_eq!(forward(&mut context, "/u/[Symbol.search](null)"), "1");
    assert_eq!(forward(&mut context, "/d/[Symbol.search](undefined)"), "2");
}
