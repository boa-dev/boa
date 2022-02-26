use super::Array;
use crate::builtins::Number;
use crate::{forward, Context, JsValue};

#[test]
fn is_array() {
    let mut context = Context::default();
    let init = r#"
        var empty = [];
        var new_arr = new Array();
        var many = ["a", "b", "c"];
        "#;
    context.eval(init).unwrap();
    assert_eq!(
        context.eval("Array.isArray(empty)").unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        context.eval("Array.isArray(new_arr)").unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        context.eval("Array.isArray(many)").unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        context.eval("Array.isArray([1, 2, 3])").unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        context.eval("Array.isArray([])").unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        context.eval("Array.isArray({})").unwrap(),
        JsValue::new(false)
    );
    // assert_eq!(context.eval("Array.isArray(new Array)"), "true");
    assert_eq!(
        context.eval("Array.isArray()").unwrap(),
        JsValue::new(false)
    );
    assert_eq!(
        context
            .eval("Array.isArray({ constructor: Array })")
            .unwrap(),
        JsValue::new(false)
    );
    assert_eq!(
        context
            .eval("Array.isArray({ push: Array.prototype.push, concat: Array.prototype.concat })")
            .unwrap(),
        JsValue::new(false)
    );
    assert_eq!(
        context.eval("Array.isArray(17)").unwrap(),
        JsValue::new(false)
    );
    assert_eq!(
        context
            .eval("Array.isArray({ __proto__: Array.prototype })")
            .unwrap(),
        JsValue::new(false)
    );
    assert_eq!(
        context.eval("Array.isArray({ length: 0 })").unwrap(),
        JsValue::new(false)
    );
}

#[test]
fn of() {
    let mut context = Context::default();
    assert_eq!(
        context
            .eval("Array.of(1, 2, 3)")
            .unwrap()
            .to_string(&mut context)
            .unwrap(),
        context
            .eval("[1, 2, 3]")
            .unwrap()
            .to_string(&mut context)
            .unwrap()
    );
    assert_eq!(
        context
            .eval("Array.of(1, 'a', [], undefined, null)")
            .unwrap()
            .to_string(&mut context)
            .unwrap(),
        context
            .eval("[1, 'a', [], undefined, null]")
            .unwrap()
            .to_string(&mut context)
            .unwrap()
    );
    assert_eq!(
        context
            .eval("Array.of()")
            .unwrap()
            .to_string(&mut context)
            .unwrap(),
        context.eval("[]").unwrap().to_string(&mut context).unwrap()
    );

    context
        .eval(r#"let a = Array.of.call(Date, "a", undefined, 3);"#)
        .unwrap();
    assert_eq!(
        context.eval("a instanceof Date").unwrap(),
        JsValue::new(true)
    );
    assert_eq!(context.eval("a[0]").unwrap(), JsValue::new("a"));
    assert_eq!(context.eval("a[1]").unwrap(), JsValue::undefined());
    assert_eq!(context.eval("a[2]").unwrap(), JsValue::new(3));
    assert_eq!(context.eval("a.length").unwrap(), JsValue::new(3));
}

#[test]
fn concat() {
    let mut context = Context::default();
    let init = r#"
    var empty = [];
    var one = [1];
    "#;
    context.eval(init).unwrap();
    // Empty ++ Empty
    let ee = context
        .eval("empty.concat(empty)")
        .unwrap()
        .display()
        .to_string();
    assert_eq!(ee, "[]");
    // Empty ++ NonEmpty
    let en = context
        .eval("empty.concat(one)")
        .unwrap()
        .display()
        .to_string();
    assert_eq!(en, "[ 1 ]");
    // NonEmpty ++ Empty
    let ne = context
        .eval("one.concat(empty)")
        .unwrap()
        .display()
        .to_string();
    assert_eq!(ne, "[ 1 ]");
    // NonEmpty ++ NonEmpty
    let nn = context
        .eval("one.concat(one)")
        .unwrap()
        .display()
        .to_string();
    assert_eq!(nn, "[ 1, 1 ]");
}

#[test]
fn copy_within() {
    let mut context = Context::default();

    let target = forward(&mut context, "[1,2,3,4,5].copyWithin(-2).join('.')");
    assert_eq!(target, String::from("\"1.2.3.1.2\""));

    let start = forward(&mut context, "[1,2,3,4,5].copyWithin(0, 3).join('.')");
    assert_eq!(start, String::from("\"4.5.3.4.5\""));

    let end = forward(&mut context, "[1,2,3,4,5].copyWithin(0, 3, 4).join('.')");
    assert_eq!(end, String::from("\"4.2.3.4.5\""));

    let negatives = forward(&mut context, "[1,2,3,4,5].copyWithin(-2, -3, -1).join('.')");
    assert_eq!(negatives, String::from("\"1.2.3.3.4\""));
}

#[test]
fn join() {
    let mut context = Context::default();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut context, init));
    // Empty
    let empty = forward(&mut context, "empty.join('.')");
    assert_eq!(empty, String::from("\"\""));
    // One
    let one = forward(&mut context, "one.join('.')");
    assert_eq!(one, String::from("\"a\""));
    // Many
    let many = forward(&mut context, "many.join('.')");
    assert_eq!(many, String::from("\"a.b.c\""));
}

#[test]
fn to_string() {
    let mut context = Context::default();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut context, init));
    // Empty
    let empty = forward(&mut context, "empty.toString()");
    assert_eq!(empty, String::from("\"\""));
    // One
    let one = forward(&mut context, "one.toString()");
    assert_eq!(one, String::from("\"a\""));
    // Many
    let many = forward(&mut context, "many.toString()");
    assert_eq!(many, String::from("\"a,b,c\""));
}

#[test]
fn every() {
    let mut context = Context::default();
    // taken from https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/every
    let init = r#"
        var empty = [];

        var array = [11, 23, 45];
        function callback(element) {
            return element > 10;
        }
        function callback2(element) {
            return element < 10;
        }

        var appendArray = [1,2,3,4];
        function appendingCallback(elem,index,arr) {
          arr.push('new');
          return elem !== "new";
        }

        var delArray = [1,2,3,4];
        function deletingCallback(elem,index,arr) {
          arr.pop()
          return elem < 3;
        }
        "#;
    eprintln!("{}", forward(&mut context, init));
    let result = forward(&mut context, "array.every(callback);");
    assert_eq!(result, "true");

    let result = forward(&mut context, "empty.every(callback);");
    assert_eq!(result, "true");

    let result = forward(&mut context, "array.every(callback2);");
    assert_eq!(result, "false");

    let result = forward(&mut context, "appendArray.every(appendingCallback);");
    assert_eq!(result, "true");

    let result = forward(&mut context, "delArray.every(deletingCallback);");
    assert_eq!(result, "true");
}

#[test]
fn find() {
    let mut context = Context::default();
    let init = r#"
        function comp(a) {
            return a == "a";
        }
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut context, init));
    let found = forward(&mut context, "many.find(comp)");
    assert_eq!(found, String::from("\"a\""));
}

#[test]
fn find_index() {
    let mut context = Context::default();

    let code = r#"
        function comp(item) {
            return item == 2;
        }
        var many = [1, 2, 3];
        var empty = [];
        var missing = [4, 5, 6];
        "#;

    forward(&mut context, code);

    let many = forward(&mut context, "many.findIndex(comp)");
    assert_eq!(many, String::from("1"));

    let empty = forward(&mut context, "empty.findIndex(comp)");
    assert_eq!(empty, String::from("-1"));

    let missing = forward(&mut context, "missing.findIndex(comp)");
    assert_eq!(missing, String::from("-1"));
}

#[test]
fn flat() {
    let mut context = Context::default();

    let code = r#"
        var depth1 = ['a', ['b', 'c']];
        var flat_depth1 = depth1.flat();

        var depth2 = ['a', ['b', ['c'], 'd']];
        var flat_depth2 = depth2.flat(2);
        "#;
    forward(&mut context, code);

    assert_eq!(forward(&mut context, "flat_depth1[0]"), "\"a\"");
    assert_eq!(forward(&mut context, "flat_depth1[1]"), "\"b\"");
    assert_eq!(forward(&mut context, "flat_depth1[2]"), "\"c\"");
    assert_eq!(forward(&mut context, "flat_depth1.length"), "3");

    assert_eq!(forward(&mut context, "flat_depth2[0]"), "\"a\"");
    assert_eq!(forward(&mut context, "flat_depth2[1]"), "\"b\"");
    assert_eq!(forward(&mut context, "flat_depth2[2]"), "\"c\"");
    assert_eq!(forward(&mut context, "flat_depth2[3]"), "\"d\"");
    assert_eq!(forward(&mut context, "flat_depth2.length"), "4");
}

#[test]
fn flat_empty() {
    let mut context = Context::default();

    let code = r#"
        var empty = [[]];
        var flat_empty = empty.flat();
        "#;
    forward(&mut context, code);

    assert_eq!(forward(&mut context, "flat_empty.length"), "0");
}

#[test]
fn flat_infinity() {
    let mut context = Context::default();

    let code = r#"
        var arr = [[[[[['a']]]]]];
        var flat_arr = arr.flat(Infinity)
        "#;
    forward(&mut context, code);

    assert_eq!(forward(&mut context, "flat_arr[0]"), "\"a\"");
    assert_eq!(forward(&mut context, "flat_arr.length"), "1");
}

#[test]
fn flat_map() {
    let mut context = Context::default();

    let code = r#"
        var double = [1, 2, 3];
        var double_flatmap = double.flatMap(i => [i * 2]);

        var sentence = ["it's Sunny", "in Cali"];
        var flat_split_sentence = sentence.flatMap(x => x.split(" "));
    "#;
    forward(&mut context, code);

    assert_eq!(forward(&mut context, "double_flatmap[0]"), "2");
    assert_eq!(forward(&mut context, "double_flatmap[1]"), "4");
    assert_eq!(forward(&mut context, "double_flatmap[2]"), "6");
    assert_eq!(forward(&mut context, "double_flatmap.length"), "3");

    assert_eq!(forward(&mut context, "flat_split_sentence[0]"), "\"it's\"");
    assert_eq!(forward(&mut context, "flat_split_sentence[1]"), "\"Sunny\"");
    assert_eq!(forward(&mut context, "flat_split_sentence[2]"), "\"in\"");
    assert_eq!(forward(&mut context, "flat_split_sentence[3]"), "\"Cali\"");
    assert_eq!(forward(&mut context, "flat_split_sentence.length"), "4");
}

#[test]
fn flat_map_with_hole() {
    let mut context = Context::default();

    let code = r#"
        var arr = [0, 1, 2];
        delete arr[1];
        var arr_flattened = arr.flatMap(i => [i * 2]);
    "#;
    forward(&mut context, code);

    assert_eq!(forward(&mut context, "arr_flattened[0]"), "0");
    assert_eq!(forward(&mut context, "arr_flattened[1]"), "4");
    assert_eq!(forward(&mut context, "arr_flattened.length"), "2");
}

#[test]
fn flat_map_not_callable() {
    let mut context = Context::default();

    let code = r#"
        try {
            var array = [1,2,3];
            array.flatMap("not a function");
        } catch (err) {
            err.name === "TypeError"
        }
    "#;

    assert_eq!(forward(&mut context, code), "true");
}

#[test]
fn push() {
    let mut context = Context::default();
    let init = r#"
        var arr = [1, 2];
        "#;
    eprintln!("{}", forward(&mut context, init));

    assert_eq!(forward(&mut context, "arr.push()"), "2");
    assert_eq!(forward(&mut context, "arr.push(3, 4)"), "4");
    assert_eq!(forward(&mut context, "arr[2]"), "3");
    assert_eq!(forward(&mut context, "arr[3]"), "4");
}

#[test]
fn pop() {
    let mut context = Context::default();
    let init = r#"
        var empty = [ ];
        var one = [1];
        var many = [1, 2, 3, 4];
        "#;
    eprintln!("{}", forward(&mut context, init));

    assert_eq!(
        forward(&mut context, "empty.pop()"),
        String::from("undefined")
    );
    assert_eq!(forward(&mut context, "one.pop()"), "1");
    assert_eq!(forward(&mut context, "one.length"), "0");
    assert_eq!(forward(&mut context, "many.pop()"), "4");
    assert_eq!(forward(&mut context, "many[0]"), "1");
    assert_eq!(forward(&mut context, "many.length"), "3");
}

#[test]
fn shift() {
    let mut context = Context::default();
    let init = r#"
        var empty = [ ];
        var one = [1];
        var many = [1, 2, 3, 4];
        "#;
    eprintln!("{}", forward(&mut context, init));

    assert_eq!(
        forward(&mut context, "empty.shift()"),
        String::from("undefined")
    );
    assert_eq!(forward(&mut context, "one.shift()"), "1");
    assert_eq!(forward(&mut context, "one.length"), "0");
    assert_eq!(forward(&mut context, "many.shift()"), "1");
    assert_eq!(forward(&mut context, "many[0]"), "2");
    assert_eq!(forward(&mut context, "many.length"), "3");
}

#[test]
fn unshift() {
    let mut context = Context::default();
    let init = r#"
        var arr = [3, 4];
        "#;
    eprintln!("{}", forward(&mut context, init));

    assert_eq!(forward(&mut context, "arr.unshift()"), "2");
    assert_eq!(forward(&mut context, "arr.unshift(1, 2)"), "4");
    assert_eq!(forward(&mut context, "arr[0]"), "1");
    assert_eq!(forward(&mut context, "arr[1]"), "2");
}

#[test]
fn reverse() {
    let mut context = Context::default();
    let init = r#"
        var arr = [1, 2];
        var reversed = arr.reverse();
        "#;
    eprintln!("{}", forward(&mut context, init));
    assert_eq!(forward(&mut context, "reversed[0]"), "2");
    assert_eq!(forward(&mut context, "reversed[1]"), "1");
    assert_eq!(forward(&mut context, "arr[0]"), "2");
    assert_eq!(forward(&mut context, "arr[1]"), "1");
}

#[test]
fn index_of() {
    let mut context = Context::default();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        "#;
    eprintln!("{}", forward(&mut context, init));

    // Empty
    let empty = forward(&mut context, "empty.indexOf('a')");
    assert_eq!(empty, String::from("-1"));

    // One
    let one = forward(&mut context, "one.indexOf('a')");
    assert_eq!(one, String::from("0"));
    // Missing from one
    let missing_from_one = forward(&mut context, "one.indexOf('b')");
    assert_eq!(missing_from_one, String::from("-1"));

    // First in many
    let first_in_many = forward(&mut context, "many.indexOf('a')");
    assert_eq!(first_in_many, String::from("0"));
    // Second in many
    let second_in_many = forward(&mut context, "many.indexOf('b')");
    assert_eq!(second_in_many, String::from("1"));

    // First in duplicates
    let first_in_many = forward(&mut context, "duplicates.indexOf('a')");
    assert_eq!(first_in_many, String::from("0"));
    // Second in duplicates
    let second_in_many = forward(&mut context, "duplicates.indexOf('b')");
    assert_eq!(second_in_many, String::from("1"));

    // Positive fromIndex greater than array length
    let fromindex_greater_than_length = forward(&mut context, "one.indexOf('a', 2)");
    assert_eq!(fromindex_greater_than_length, String::from("-1"));
    // Positive fromIndex missed match
    let fromindex_misses_match = forward(&mut context, "many.indexOf('a', 1)");
    assert_eq!(fromindex_misses_match, String::from("-1"));
    // Positive fromIndex matched
    let fromindex_matches = forward(&mut context, "many.indexOf('b', 1)");
    assert_eq!(fromindex_matches, String::from("1"));
    // Positive fromIndex with duplicates
    let first_in_many = forward(&mut context, "duplicates.indexOf('a', 1)");
    assert_eq!(first_in_many, String::from("3"));

    // Negative fromIndex greater than array length
    let fromindex_greater_than_length = forward(&mut context, "one.indexOf('a', -2)");
    assert_eq!(fromindex_greater_than_length, String::from("0"));
    // Negative fromIndex missed match
    let fromindex_misses_match = forward(&mut context, "many.indexOf('b', -1)");
    assert_eq!(fromindex_misses_match, String::from("-1"));
    // Negative fromIndex matched
    let fromindex_matches = forward(&mut context, "many.indexOf('c', -1)");
    assert_eq!(fromindex_matches, String::from("2"));
    // Negative fromIndex with duplicates
    let second_in_many = forward(&mut context, "duplicates.indexOf('b', -2)");
    assert_eq!(second_in_many, String::from("4"));
}

#[test]
fn last_index_of() {
    let mut context = Context::default();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        "#;
    eprintln!("{}", forward(&mut context, init));

    // Empty
    let empty = forward(&mut context, "empty.lastIndexOf('a')");
    assert_eq!(empty, String::from("-1"));

    // One
    let one = forward(&mut context, "one.lastIndexOf('a')");
    assert_eq!(one, String::from("0"));
    // Missing from one
    let missing_from_one = forward(&mut context, "one.lastIndexOf('b')");
    assert_eq!(missing_from_one, String::from("-1"));

    // First in many
    let first_in_many = forward(&mut context, "many.lastIndexOf('a')");
    assert_eq!(first_in_many, String::from("0"));
    // Second in many
    let second_in_many = forward(&mut context, "many.lastIndexOf('b')");
    assert_eq!(second_in_many, String::from("1"));

    // 4th in duplicates
    let first_in_many = forward(&mut context, "duplicates.lastIndexOf('a')");
    assert_eq!(first_in_many, String::from("3"));
    // 5th in duplicates
    let second_in_many = forward(&mut context, "duplicates.lastIndexOf('b')");
    assert_eq!(second_in_many, String::from("4"));

    // Positive fromIndex greater than array length
    let fromindex_greater_than_length = forward(&mut context, "one.lastIndexOf('a', 2)");
    assert_eq!(fromindex_greater_than_length, String::from("0"));
    // Positive fromIndex missed match
    let fromindex_misses_match = forward(&mut context, "many.lastIndexOf('c', 1)");
    assert_eq!(fromindex_misses_match, String::from("-1"));
    // Positive fromIndex matched
    let fromindex_matches = forward(&mut context, "many.lastIndexOf('b', 1)");
    assert_eq!(fromindex_matches, String::from("1"));
    // Positive fromIndex with duplicates
    let first_in_many = forward(&mut context, "duplicates.lastIndexOf('a', 1)");
    assert_eq!(first_in_many, String::from("0"));

    // Negative fromIndex greater than array length
    let fromindex_greater_than_length = forward(&mut context, "one.lastIndexOf('a', -2)");
    assert_eq!(fromindex_greater_than_length, String::from("-1"));
    // Negative fromIndex missed match
    let fromindex_misses_match = forward(&mut context, "many.lastIndexOf('c', -2)");
    assert_eq!(fromindex_misses_match, String::from("-1"));
    // Negative fromIndex matched
    let fromindex_matches = forward(&mut context, "many.lastIndexOf('c', -1)");
    assert_eq!(fromindex_matches, String::from("2"));
    // Negative fromIndex with duplicates
    let second_in_many = forward(&mut context, "duplicates.lastIndexOf('b', -2)");
    assert_eq!(second_in_many, String::from("1"));
}

#[test]
fn fill_obj_ref() {
    let mut context = Context::default();

    // test object reference
    forward(&mut context, "a = (new Array(3)).fill({});");
    forward(&mut context, "a[0].hi = 'hi';");
    assert_eq!(forward(&mut context, "a[0].hi"), "\"hi\"");
}

#[test]
fn fill() {
    let mut context = Context::default();

    forward(&mut context, "var a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4).join()"),
        String::from("\"4,4,4\"")
    );
    // make sure the array is modified
    assert_eq!(forward(&mut context, "a.join()"), String::from("\"4,4,4\""));

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, '1').join()"),
        String::from("\"1,4,4\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, 1, 2).join()"),
        String::from("\"1,4,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, 1, 1).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, 3, 3).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, -3, -2).join()"),
        String::from("\"4,2,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, NaN, NaN).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, 3, 5).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, '1.2', '2.5').join()"),
        String::from("\"1,4,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, 'str').join()"),
        String::from("\"4,4,4\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, 'str', 'str').join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, undefined, null).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut context, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut context, "a.fill(4, undefined, undefined).join()"),
        String::from("\"4,4,4\"")
    );

    assert_eq!(
        forward(&mut context, "a.fill().join()"),
        String::from("\",,\"")
    );

    // test object reference
    forward(&mut context, "a = (new Array(3)).fill({});");
    forward(&mut context, "a[0].hi = 'hi';");
    assert_eq!(forward(&mut context, "a[0].hi"), String::from("\"hi\""));
}

#[test]
fn includes_value() {
    let mut context = Context::default();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        var undefined = [undefined];
        "#;
    eprintln!("{}", forward(&mut context, init));

    // Empty
    let empty = forward(&mut context, "empty.includes('a')");
    assert_eq!(empty, String::from("false"));

    // One
    let one = forward(&mut context, "one.includes('a')");
    assert_eq!(one, String::from("true"));
    // Missing from one
    let missing_from_one = forward(&mut context, "one.includes('b')");
    assert_eq!(missing_from_one, String::from("false"));

    // In many
    let first_in_many = forward(&mut context, "many.includes('c')");
    assert_eq!(first_in_many, String::from("true"));
    // Missing from many
    let second_in_many = forward(&mut context, "many.includes('d')");
    assert_eq!(second_in_many, String::from("false"));

    // In duplicates
    let first_in_many = forward(&mut context, "duplicates.includes('a')");
    assert_eq!(first_in_many, String::from("true"));
    // Missing from duplicates
    let second_in_many = forward(&mut context, "duplicates.includes('d')");
    assert_eq!(second_in_many, String::from("false"));
}

#[test]
fn map() {
    let mut context = Context::default();

    let js = r#"
        var empty = [];
        var one = ["x"];
        var many = ["x", "y", "z"];

        var _this = { answer: 42 };

        function callbackThatUsesThis() {
             return 'The answer to life is: ' + this.answer;
        }

        var empty_mapped = empty.map(v => v + '_');
        var one_mapped = one.map(v => '_' + v);
        var many_mapped = many.map(v => '_' + v + '_');
        "#;

    forward(&mut context, js);

    // assert the old arrays have not been modified
    assert_eq!(forward(&mut context, "one[0]"), String::from("\"x\""));
    assert_eq!(
        forward(&mut context, "many[2] + many[1] + many[0]"),
        String::from("\"zyx\"")
    );

    // NB: These tests need to be rewritten once `Display` has been implemented for `Array`
    // Empty
    assert_eq!(
        forward(&mut context, "empty_mapped.length"),
        String::from("0")
    );

    // One
    assert_eq!(
        forward(&mut context, "one_mapped.length"),
        String::from("1")
    );
    assert_eq!(
        forward(&mut context, "one_mapped[0]"),
        String::from("\"_x\"")
    );

    // Many
    assert_eq!(
        forward(&mut context, "many_mapped.length"),
        String::from("3")
    );
    assert_eq!(
        forward(
            &mut context,
            "many_mapped[0] + many_mapped[1] + many_mapped[2]"
        ),
        String::from("\"_x__y__z_\"")
    );

    // One but it uses `this` inside the callback
    let one_with_this = forward(&mut context, "one.map(callbackThatUsesThis, _this)[0];");
    assert_eq!(one_with_this, String::from("\"The answer to life is: 42\""));
}

#[test]
fn slice() {
    let mut context = Context::default();
    let init = r#"
        var empty = [ ].slice();
        var one = ["a"].slice();
        var many1 = ["a", "b", "c", "d"].slice(1);
        var many2 = ["a", "b", "c", "d"].slice(2, 3);
        var many3 = ["a", "b", "c", "d"].slice(7);
        "#;
    eprintln!("{}", forward(&mut context, init));

    assert_eq!(forward(&mut context, "empty.length"), "0");
    assert_eq!(forward(&mut context, "one[0]"), "\"a\"");
    assert_eq!(forward(&mut context, "many1[0]"), "\"b\"");
    assert_eq!(forward(&mut context, "many1[1]"), "\"c\"");
    assert_eq!(forward(&mut context, "many1[2]"), "\"d\"");
    assert_eq!(forward(&mut context, "many1.length"), "3");
    assert_eq!(forward(&mut context, "many2[0]"), "\"c\"");
    assert_eq!(forward(&mut context, "many2.length"), "1");
    assert_eq!(forward(&mut context, "many3.length"), "0");
}

#[test]
fn for_each() {
    let mut context = Context::default();
    let init = r#"
        var a = [2, 3, 4, 5];
        var sum = 0;
        var indexSum = 0;
        var listLengthSum = 0;
        function callingCallback(item, index, list) {
            sum += item;
            indexSum += index;
            listLengthSum += list.length;
        }
        a.forEach(callingCallback);
        "#;
    eprintln!("{}", forward(&mut context, init));

    assert_eq!(forward(&mut context, "sum"), "14");
    assert_eq!(forward(&mut context, "indexSum"), "6");
    assert_eq!(forward(&mut context, "listLengthSum"), "16");
}

#[test]
fn for_each_push_value() {
    let mut context = Context::default();
    let init = r#"
        var a = [1, 2, 3, 4];
        function callingCallback(item, index, list) {
            list.push(item * 2);
        }
        a.forEach(callingCallback);
        "#;
    eprintln!("{}", forward(&mut context, init));

    // [ 1, 2, 3, 4, 2, 4, 6, 8 ]
    assert_eq!(forward(&mut context, "a.length"), "8");
    assert_eq!(forward(&mut context, "a[4]"), "2");
    assert_eq!(forward(&mut context, "a[5]"), "4");
    assert_eq!(forward(&mut context, "a[6]"), "6");
    assert_eq!(forward(&mut context, "a[7]"), "8");
}

#[test]
fn filter() {
    let mut context = Context::default();

    let js = r#"
        var empty = [];
        var one = ["1"];
        var many = ["1", "0", "1"];

        var empty_filtered = empty.filter(v => v === "1");
        var one_filtered = one.filter(v => v === "1");
        var zero_filtered = one.filter(v => v === "0");
        var many_one_filtered = many.filter(v => v === "1");
        var many_zero_filtered = many.filter(v => v === "0");
        "#;

    forward(&mut context, js);

    // assert the old arrays have not been modified
    assert_eq!(forward(&mut context, "one[0]"), String::from("\"1\""));
    assert_eq!(
        forward(&mut context, "many[2] + many[1] + many[0]"),
        String::from("\"101\"")
    );

    // NB: These tests need to be rewritten once `Display` has been implemented for `Array`
    // Empty
    assert_eq!(
        forward(&mut context, "empty_filtered.length"),
        String::from("0")
    );

    // One filtered on "1"
    assert_eq!(
        forward(&mut context, "one_filtered.length"),
        String::from("1")
    );
    assert_eq!(
        forward(&mut context, "one_filtered[0]"),
        String::from("\"1\"")
    );

    //  One filtered on "0"
    assert_eq!(
        forward(&mut context, "zero_filtered.length"),
        String::from("0")
    );

    // Many filtered on "1"
    assert_eq!(
        forward(&mut context, "many_one_filtered.length"),
        String::from("2")
    );
    assert_eq!(
        forward(&mut context, "many_one_filtered[0] + many_one_filtered[1]"),
        String::from("\"11\"")
    );

    // Many filtered on "0"
    assert_eq!(
        forward(&mut context, "many_zero_filtered.length"),
        String::from("1")
    );
    assert_eq!(
        forward(&mut context, "many_zero_filtered[0]"),
        String::from("\"0\"")
    );
}

#[test]
fn some() {
    let mut context = Context::default();
    let init = r#"
        var empty = [];

        var array = [11, 23, 45];
        function lessThan10(element) {
            return element > 10;
        }
        function greaterThan10(element) {
            return element < 10;
        }

        // Cases where callback mutates the array.
        var appendArray = [1,2,3,4];
        function appendingCallback(elem,index,arr) {
          arr.push('new');
          return elem !== "new";
        }

        var delArray = [1,2,3,4];
        function deletingCallback(elem,index,arr) {
          arr.pop()
          return elem < 3;
        }
        "#;
    forward(&mut context, init);
    let result = forward(&mut context, "array.some(lessThan10);");
    assert_eq!(result, "true");

    let result = forward(&mut context, "empty.some(lessThan10);");
    assert_eq!(result, "false");

    let result = forward(&mut context, "array.some(greaterThan10);");
    assert_eq!(result, "false");

    let result = forward(&mut context, "appendArray.some(appendingCallback);");
    let append_array_length = forward(&mut context, "appendArray.length");
    assert_eq!(append_array_length, "5");
    assert_eq!(result, "true");

    let result = forward(&mut context, "delArray.some(deletingCallback);");
    let del_array_length = forward(&mut context, "delArray.length");
    assert_eq!(del_array_length, "3");
    assert_eq!(result, "true");
}

#[test]
fn reduce() {
    let mut context = Context::default();

    let init = r#"
        var arr = [1, 2, 3, 4];
        function add(acc, x) {
            return acc + x;
        }

        function addIdx(acc, _, idx) {
            return acc + idx;
        }

        function addLen(acc, _x, _idx, arr) {
            return acc + arr.length;
        }

        function addResize(acc, x, idx, arr) {
            if(idx == 0) {
                arr.length = 3;
            }
            return acc + x;
        }
        var delArray = [1, 2, 3, 4, 5];
        delete delArray[0];
        delete delArray[1];
        delete delArray[3];

    "#;
    forward(&mut context, init);

    // empty array
    let result = forward(&mut context, "[].reduce(add, 0)");
    assert_eq!(result, "0");

    // simple with initial value
    let result = forward(&mut context, "arr.reduce(add, 0)");
    assert_eq!(result, "10");

    // without initial value
    let result = forward(&mut context, "arr.reduce(add)");
    assert_eq!(result, "10");

    // with some items missing
    let result = forward(&mut context, "delArray.reduce(add, 0)");
    assert_eq!(result, "8");

    // with index
    let result = forward(&mut context, "arr.reduce(addIdx, 0)");
    assert_eq!(result, "6");

    // with array
    let result = forward(&mut context, "arr.reduce(addLen, 0)");
    assert_eq!(result, "16");

    // resizing the array as reduce progresses
    let result = forward(&mut context, "arr.reduce(addResize, 0)");
    assert_eq!(result, "6");

    // Empty array
    let result = forward(
        &mut context,
        r#"
        try {
            [].reduce((acc, x) => acc + x);
        } catch(e) {
            e.message
        }
    "#,
    );
    assert_eq!(
        result,
        "\"Array.prototype.reduce: called on an empty array and with no initial value\""
    );

    // Array with no defined elements
    let result = forward(
        &mut context,
        r#"
        try {
            var arr = [0, 1];
            delete arr[0];
            delete arr[1];
            arr.reduce((acc, x) => acc + x);
        } catch(e) {
            e.message
        }
    "#,
    );
    assert_eq!(
        result,
        "\"Array.prototype.reduce: called on an empty array and with no initial value\""
    );

    // No callback
    let result = forward(
        &mut context,
        r#"
        try {
            arr.reduce("");
        } catch(e) {
            e.message
        }
    "#,
    );
    assert_eq!(
        result,
        "\"Array.prototype.reduce: callback function is not callable\""
    );
}

#[test]
fn reduce_right() {
    let mut context = Context::default();

    let init = r#"
        var arr = [1, 2, 3, 4];
        function sub(acc, x) {
            return acc - x;
        }

        function subIdx(acc, _, idx) {
            return acc - idx;
        }

        function subLen(acc, _x, _idx, arr) {
            return acc - arr.length;
        }

        function subResize(acc, x, idx, arr) {
            if(idx == arr.length - 1) {
                arr.length = 1;
            }
            return acc - x;
        }
        function subResize0(acc, x, idx, arr) {
            if(idx == arr.length - 2) {
                arr.length = 0;
            }
            return acc - x;
        }
        var delArray = [1, 2, 3, 4, 5];
        delete delArray[0];
        delete delArray[1];
        delete delArray[3];

    "#;
    forward(&mut context, init);

    // empty array
    let result = forward(&mut context, "[].reduceRight(sub, 0)");
    assert_eq!(result, "0");

    // simple with initial value
    let result = forward(&mut context, "arr.reduceRight(sub, 0)");
    assert_eq!(result, "-10");

    // without initial value
    let result = forward(&mut context, "arr.reduceRight(sub)");
    assert_eq!(result, "-2");

    // with some items missing
    let result = forward(&mut context, "delArray.reduceRight(sub, 0)");
    assert_eq!(result, "-8");

    // with index
    let result = forward(&mut context, "arr.reduceRight(subIdx)");
    assert_eq!(result, "1");

    // with array
    let result = forward(&mut context, "arr.reduceRight(subLen)");
    assert_eq!(result, "-8");

    // resizing the array as reduce progresses
    let result = forward(&mut context, "arr.reduceRight(subResize, 0)");
    assert_eq!(result, "-5");

    // reset array
    forward(&mut context, "arr = [1, 2, 3, 4];");

    // resizing the array to 0 as reduce progresses
    let result = forward(&mut context, "arr.reduceRight(subResize0, 0)");
    assert_eq!(result, "-7");

    // Empty array
    let result = forward(
        &mut context,
        r#"
        try {
            [].reduceRight((acc, x) => acc + x);
        } catch(e) {
            e.message
        }
    "#,
    );
    assert_eq!(
        result,
        "\"Array.prototype.reduceRight: called on an empty array and with no initial value\""
    );

    // Array with no defined elements
    let result = forward(
        &mut context,
        r#"
        try {
            var arr = [0, 1];
            delete arr[0];
            delete arr[1];
            arr.reduceRight((acc, x) => acc + x);
        } catch(e) {
            e.message
        }
    "#,
    );
    assert_eq!(
        result,
        "\"Array.prototype.reduceRight: called on an empty array and with no initial value\""
    );

    // No callback
    let result = forward(
        &mut context,
        r#"
        try {
            arr.reduceRight("");
        } catch(e) {
            e.message
        }
    "#,
    );
    assert_eq!(
        result,
        "\"Array.prototype.reduceRight: callback function is not callable\""
    );
}

#[test]
fn call_array_constructor_with_one_argument() {
    let mut context = Context::default();
    let init = r#"
        var empty = new Array(0);

        var five = new Array(5);

        var one = new Array("Hello, world!");
        "#;
    forward(&mut context, init);
    // let result = forward(&mut context, "empty.length");
    // assert_eq!(result, "0");

    // let result = forward(&mut context, "five.length");
    // assert_eq!(result, "5");

    // let result = forward(&mut context, "one.length");
    // assert_eq!(result, "1");
}

#[test]
fn array_values_simple() {
    let mut context = Context::default();
    let init = r#"
        var iterator = [1, 2, 3].values();
        var next = iterator.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "1");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "2");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "3");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn array_keys_simple() {
    let mut context = Context::default();
    let init = r#"
        var iterator = [1, 2, 3].keys();
        var next = iterator.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "0");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "1");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "2");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn array_entries_simple() {
    let mut context = Context::default();
    let init = r#"
        var iterator = [1, 2, 3].entries();
        var next = iterator.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "[ 0, 1 ]");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "[ 1, 2 ]");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "[ 2, 3 ]");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn array_values_empty() {
    let mut context = Context::default();
    let init = r#"
        var iterator = [].values();
        var next = iterator.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn array_values_sparse() {
    let mut context = Context::default();
    let init = r#"
        var array = Array();
        array[3] = 5;
        var iterator = array.values();
        var next = iterator.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "5");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn array_symbol_iterator() {
    let mut context = Context::default();
    let init = r#"
        var iterator = [1, 2, 3][Symbol.iterator]();
        var next = iterator.next();
    "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "next.value"), "1");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "2");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "3");
    assert_eq!(forward(&mut context, "next.done"), "false");
    forward(&mut context, "next = iterator.next()");
    assert_eq!(forward(&mut context, "next.value"), "undefined");
    assert_eq!(forward(&mut context, "next.done"), "true");
}

#[test]
fn array_values_symbol_iterator() {
    let mut context = Context::default();
    let init = r#"
        var iterator = [1, 2, 3].values();
        iterator === iterator[Symbol.iterator]();
    "#;
    assert_eq!(forward(&mut context, init), "true");
}

#[test]
fn array_spread_arrays() {
    let mut context = Context::default();
    let init = r#"
        const array1 = [2, 3];
        const array2 = [1, ...array1];
        array2[0] === 1 && array2[1] === 2 && array2[2] === 3;
    "#;
    assert_eq!(forward(&mut context, init), "true");
}

#[test]
fn array_spread_non_iterable() {
    let mut context = Context::default();
    let init = r#"
        try {
            const array2 = [...5];
        } catch (err) {
            err.name === "TypeError" && err.message === "Value is not callable"
        }
    "#;
    assert_eq!(forward(&mut context, init), "true");
}

#[test]
fn get_relative_start() {
    let mut context = Context::default();

    assert_eq!(Array::get_relative_start(&mut context, None, 10), Ok(0));
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::undefined()), 10),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::new(f64::NEG_INFINITY)), 10),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::new(f64::INFINITY)), 10),
        Ok(10)
    );
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::new(-1)), 10),
        Ok(9)
    );
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::new(1)), 10),
        Ok(1)
    );
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::new(-11)), 10),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::new(11)), 10),
        Ok(10)
    );
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::new(f64::MIN)), 10),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_start(
            &mut context,
            Some(&JsValue::new(Number::MIN_SAFE_INTEGER)),
            10
        ),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_start(&mut context, Some(&JsValue::new(f64::MAX)), 10),
        Ok(10)
    );

    // This test is relevant only on 32-bit archs (where usize == u32 thus `len` is u32)
    assert_eq!(
        Array::get_relative_start(
            &mut context,
            Some(&JsValue::new(Number::MAX_SAFE_INTEGER)),
            10
        ),
        Ok(10)
    );
}

#[test]
fn get_relative_end() {
    let mut context = Context::default();

    assert_eq!(Array::get_relative_end(&mut context, None, 10), Ok(10));
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::undefined()), 10),
        Ok(10)
    );
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::new(f64::NEG_INFINITY)), 10),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::new(f64::INFINITY)), 10),
        Ok(10)
    );
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::new(-1)), 10),
        Ok(9)
    );
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::new(1)), 10),
        Ok(1)
    );
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::new(-11)), 10),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::new(11)), 10),
        Ok(10)
    );
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::new(f64::MIN)), 10),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_end(
            &mut context,
            Some(&JsValue::new(Number::MIN_SAFE_INTEGER)),
            10
        ),
        Ok(0)
    );
    assert_eq!(
        Array::get_relative_end(&mut context, Some(&JsValue::new(f64::MAX)), 10),
        Ok(10)
    );

    // This test is relevant only on 32-bit archs (where usize == u32 thus `len` is u32)
    assert_eq!(
        Array::get_relative_end(
            &mut context,
            Some(&JsValue::new(Number::MAX_SAFE_INTEGER)),
            10
        ),
        Ok(10)
    );
}

#[test]
fn array_length_is_not_enumerable() {
    let mut context = Context::default();

    let array =
        Array::array_create(0, None, &mut context).expect("could not create an empty array");
    let desc = array
        .__get_own_property__(&"length".into(), &mut context)
        .expect("accessing length property on array should not throw")
        .expect("there should always be a length property on arrays");
    assert!(!desc.expect_enumerable());
}

#[test]
fn array_sort() {
    let mut context = Context::default();
    let init = r#"
        let arr = ['80', '9', '700', 40, 1, 5, 200];

        function compareNumbers(a, b) {
            return a - b;
        }
    "#;
    forward(&mut context, init);
    assert_eq!(
        forward(&mut context, "arr.sort().join()"),
        "\"1,200,40,5,700,80,9\""
    );
    assert_eq!(
        forward(&mut context, "arr.sort(compareNumbers).join()"),
        "\"1,5,9,40,80,200,700\""
    );
}
