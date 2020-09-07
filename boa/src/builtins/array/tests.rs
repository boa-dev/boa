use crate::{forward, Context, Value};

#[test]
fn is_array() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [];
        var new_arr = new Array();
        var many = ["a", "b", "c"];
        "#;
    engine.eval(init).unwrap();
    assert_eq!(
        engine.eval("Array.isArray(empty)").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        engine.eval("Array.isArray(new_arr)").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        engine.eval("Array.isArray(many)").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        engine.eval("Array.isArray([1, 2, 3])").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        engine.eval("Array.isArray([])").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        engine.eval("Array.isArray({})").unwrap(),
        Value::Boolean(false)
    );
    // assert_eq!(engine.eval("Array.isArray(new Array)"), "true");
    assert_eq!(
        engine.eval("Array.isArray()").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        engine
            .eval("Array.isArray({ constructor: Array })")
            .unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        engine
            .eval("Array.isArray({ push: Array.prototype.push, concat: Array.prototype.concat })")
            .unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        engine.eval("Array.isArray(17)").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        engine
            .eval("Array.isArray({ __proto__: Array.prototype })")
            .unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        engine.eval("Array.isArray({ length: 0 })").unwrap(),
        Value::Boolean(false)
    );
}

#[test]
#[ignore]
fn concat() {
    //TODO: array display formatter
    let mut engine = Context::new();
    let init = r#"
    var empty = new Array();
    var one = new Array(1);
    "#;
    engine.eval(init).unwrap();
    // Empty ++ Empty
    let ee = engine
        .eval("empty.concat(empty)")
        .unwrap()
        .to_string(&mut engine)
        .unwrap();
    assert_eq!(ee, "[]");
    // Empty ++ NonEmpty
    let en = engine
        .eval("empty.concat(one)")
        .unwrap()
        .to_string(&mut engine)
        .unwrap();
    assert_eq!(en, "[a]");
    // NonEmpty ++ Empty
    let ne = engine
        .eval("one.concat(empty)")
        .unwrap()
        .to_string(&mut engine)
        .unwrap();
    assert_eq!(ne, "a.b.c");
    // NonEmpty ++ NonEmpty
    let nn = engine
        .eval("one.concat(one)")
        .unwrap()
        .to_string(&mut engine)
        .unwrap();
    assert_eq!(nn, "a.b.c");
}

#[test]
fn join() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut engine, init));
    // Empty
    let empty = forward(&mut engine, "empty.join('.')");
    assert_eq!(empty, String::from("\"\""));
    // One
    let one = forward(&mut engine, "one.join('.')");
    assert_eq!(one, String::from("\"a\""));
    // Many
    let many = forward(&mut engine, "many.join('.')");
    assert_eq!(many, String::from("\"a.b.c\""));
}

#[test]
fn to_string() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut engine, init));
    // Empty
    let empty = forward(&mut engine, "empty.toString()");
    assert_eq!(empty, String::from("\"\""));
    // One
    let one = forward(&mut engine, "one.toString()");
    assert_eq!(one, String::from("\"a\""));
    // Many
    let many = forward(&mut engine, "many.toString()");
    assert_eq!(many, String::from("\"a,b,c\""));
}

#[test]
fn every() {
    let mut engine = Context::new();
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
    eprintln!("{}", forward(&mut engine, init));
    let result = forward(&mut engine, "array.every(callback);");
    assert_eq!(result, "true");

    let result = forward(&mut engine, "empty.every(callback);");
    assert_eq!(result, "true");

    let result = forward(&mut engine, "array.every(callback2);");
    assert_eq!(result, "false");

    let result = forward(&mut engine, "appendArray.every(appendingCallback);");
    assert_eq!(result, "true");

    let result = forward(&mut engine, "delArray.every(deletingCallback);");
    assert_eq!(result, "true");
}

#[test]
fn find() {
    let mut engine = Context::new();
    let init = r#"
        function comp(a) {
            return a == "a";
        }
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let found = forward(&mut engine, "many.find(comp)");
    assert_eq!(found, String::from("\"a\""));
}

#[test]
fn find_index() {
    let mut engine = Context::new();

    let code = r#"
        function comp(item) {
            return item == 2;
        }
        var many = [1, 2, 3];
        var empty = [];
        var missing = [4, 5, 6];
        "#;

    forward(&mut engine, code);

    let many = forward(&mut engine, "many.findIndex(comp)");
    assert_eq!(many, String::from("1"));

    let empty = forward(&mut engine, "empty.findIndex(comp)");
    assert_eq!(empty, String::from("-1"));

    let missing = forward(&mut engine, "missing.findIndex(comp)");
    assert_eq!(missing, String::from("-1"));
}

#[test]
fn push() {
    let mut engine = Context::new();
    let init = r#"
        var arr = [1, 2];
        "#;
    eprintln!("{}", forward(&mut engine, init));

    assert_eq!(forward(&mut engine, "arr.push()"), "2");
    assert_eq!(forward(&mut engine, "arr.push(3, 4)"), "4");
    assert_eq!(forward(&mut engine, "arr[2]"), "3");
    assert_eq!(forward(&mut engine, "arr[3]"), "4");
}

#[test]
fn pop() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [ ];
        var one = [1];
        var many = [1, 2, 3, 4];
        "#;
    eprintln!("{}", forward(&mut engine, init));

    assert_eq!(
        forward(&mut engine, "empty.pop()"),
        String::from("undefined")
    );
    assert_eq!(forward(&mut engine, "one.pop()"), "1");
    assert_eq!(forward(&mut engine, "one.length"), "0");
    assert_eq!(forward(&mut engine, "many.pop()"), "4");
    assert_eq!(forward(&mut engine, "many[0]"), "1");
    assert_eq!(forward(&mut engine, "many.length"), "3");
}

#[test]
fn shift() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [ ];
        var one = [1];
        var many = [1, 2, 3, 4];
        "#;
    eprintln!("{}", forward(&mut engine, init));

    assert_eq!(
        forward(&mut engine, "empty.shift()"),
        String::from("undefined")
    );
    assert_eq!(forward(&mut engine, "one.shift()"), "1");
    assert_eq!(forward(&mut engine, "one.length"), "0");
    assert_eq!(forward(&mut engine, "many.shift()"), "1");
    assert_eq!(forward(&mut engine, "many[0]"), "2");
    assert_eq!(forward(&mut engine, "many.length"), "3");
}

#[test]
fn unshift() {
    let mut engine = Context::new();
    let init = r#"
        var arr = [3, 4];
        "#;
    eprintln!("{}", forward(&mut engine, init));

    assert_eq!(forward(&mut engine, "arr.unshift()"), "2");
    assert_eq!(forward(&mut engine, "arr.unshift(1, 2)"), "4");
    assert_eq!(forward(&mut engine, "arr[0]"), "1");
    assert_eq!(forward(&mut engine, "arr[1]"), "2");
}

#[test]
fn reverse() {
    let mut engine = Context::new();
    let init = r#"
        var arr = [1, 2];
        var reversed = arr.reverse();
        "#;
    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(forward(&mut engine, "reversed[0]"), "2");
    assert_eq!(forward(&mut engine, "reversed[1]"), "1");
    assert_eq!(forward(&mut engine, "arr[0]"), "2");
    assert_eq!(forward(&mut engine, "arr[1]"), "1");
}

#[test]
fn index_of() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        "#;
    eprintln!("{}", forward(&mut engine, init));

    // Empty
    let empty = forward(&mut engine, "empty.indexOf('a')");
    assert_eq!(empty, String::from("-1"));

    // One
    let one = forward(&mut engine, "one.indexOf('a')");
    assert_eq!(one, String::from("0"));
    // Missing from one
    let missing_from_one = forward(&mut engine, "one.indexOf('b')");
    assert_eq!(missing_from_one, String::from("-1"));

    // First in many
    let first_in_many = forward(&mut engine, "many.indexOf('a')");
    assert_eq!(first_in_many, String::from("0"));
    // Second in many
    let second_in_many = forward(&mut engine, "many.indexOf('b')");
    assert_eq!(second_in_many, String::from("1"));

    // First in duplicates
    let first_in_many = forward(&mut engine, "duplicates.indexOf('a')");
    assert_eq!(first_in_many, String::from("0"));
    // Second in duplicates
    let second_in_many = forward(&mut engine, "duplicates.indexOf('b')");
    assert_eq!(second_in_many, String::from("1"));

    // Positive fromIndex greater than array length
    let fromindex_greater_than_length = forward(&mut engine, "one.indexOf('a', 2)");
    assert_eq!(fromindex_greater_than_length, String::from("-1"));
    // Positive fromIndex missed match
    let fromindex_misses_match = forward(&mut engine, "many.indexOf('a', 1)");
    assert_eq!(fromindex_misses_match, String::from("-1"));
    // Positive fromIndex matched
    let fromindex_matches = forward(&mut engine, "many.indexOf('b', 1)");
    assert_eq!(fromindex_matches, String::from("1"));
    // Positive fromIndex with duplicates
    let first_in_many = forward(&mut engine, "duplicates.indexOf('a', 1)");
    assert_eq!(first_in_many, String::from("3"));

    // Negative fromIndex greater than array length
    let fromindex_greater_than_length = forward(&mut engine, "one.indexOf('a', -2)");
    assert_eq!(fromindex_greater_than_length, String::from("0"));
    // Negative fromIndex missed match
    let fromindex_misses_match = forward(&mut engine, "many.indexOf('b', -1)");
    assert_eq!(fromindex_misses_match, String::from("-1"));
    // Negative fromIndex matched
    let fromindex_matches = forward(&mut engine, "many.indexOf('c', -1)");
    assert_eq!(fromindex_matches, String::from("2"));
    // Negative fromIndex with duplicates
    let second_in_many = forward(&mut engine, "duplicates.indexOf('b', -2)");
    assert_eq!(second_in_many, String::from("4"));
}

#[test]
fn last_index_of() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        "#;
    eprintln!("{}", forward(&mut engine, init));

    // Empty
    let empty = forward(&mut engine, "empty.lastIndexOf('a')");
    assert_eq!(empty, String::from("-1"));

    // One
    let one = forward(&mut engine, "one.lastIndexOf('a')");
    assert_eq!(one, String::from("0"));
    // Missing from one
    let missing_from_one = forward(&mut engine, "one.lastIndexOf('b')");
    assert_eq!(missing_from_one, String::from("-1"));

    // First in many
    let first_in_many = forward(&mut engine, "many.lastIndexOf('a')");
    assert_eq!(first_in_many, String::from("0"));
    // Second in many
    let second_in_many = forward(&mut engine, "many.lastIndexOf('b')");
    assert_eq!(second_in_many, String::from("1"));

    // 4th in duplicates
    let first_in_many = forward(&mut engine, "duplicates.lastIndexOf('a')");
    assert_eq!(first_in_many, String::from("3"));
    // 5th in duplicates
    let second_in_many = forward(&mut engine, "duplicates.lastIndexOf('b')");
    assert_eq!(second_in_many, String::from("4"));

    // Positive fromIndex greater than array length
    let fromindex_greater_than_length = forward(&mut engine, "one.lastIndexOf('a', 2)");
    assert_eq!(fromindex_greater_than_length, String::from("0"));
    // Positive fromIndex missed match
    let fromindex_misses_match = forward(&mut engine, "many.lastIndexOf('c', 1)");
    assert_eq!(fromindex_misses_match, String::from("-1"));
    // Positive fromIndex matched
    let fromindex_matches = forward(&mut engine, "many.lastIndexOf('b', 1)");
    assert_eq!(fromindex_matches, String::from("1"));
    // Positive fromIndex with duplicates
    let first_in_many = forward(&mut engine, "duplicates.lastIndexOf('a', 1)");
    assert_eq!(first_in_many, String::from("0"));

    // Negative fromIndex greater than array length
    let fromindex_greater_than_length = forward(&mut engine, "one.lastIndexOf('a', -2)");
    assert_eq!(fromindex_greater_than_length, String::from("-1"));
    // Negative fromIndex missed match
    let fromindex_misses_match = forward(&mut engine, "many.lastIndexOf('c', -2)");
    assert_eq!(fromindex_misses_match, String::from("-1"));
    // Negative fromIndex matched
    let fromindex_matches = forward(&mut engine, "many.lastIndexOf('c', -1)");
    assert_eq!(fromindex_matches, String::from("2"));
    // Negative fromIndex with duplicates
    let second_in_many = forward(&mut engine, "duplicates.lastIndexOf('b', -2)");
    assert_eq!(second_in_many, String::from("1"));
}

#[test]
fn fill_obj_ref() {
    let mut engine = Context::new();

    // test object reference
    forward(&mut engine, "a = (new Array(3)).fill({});");
    forward(&mut engine, "a[0].hi = 'hi';");
    assert_eq!(forward(&mut engine, "a[0].hi"), "\"hi\"");
}

#[test]
fn fill() {
    let mut engine = Context::new();

    forward(&mut engine, "var a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4).join()"),
        String::from("\"4,4,4\"")
    );
    // make sure the array is modified
    assert_eq!(forward(&mut engine, "a.join()"), String::from("\"4,4,4\""));

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, '1').join()"),
        String::from("\"1,4,4\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 1, 2).join()"),
        String::from("\"1,4,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 1, 1).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 3, 3).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, -3, -2).join()"),
        String::from("\"4,2,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, NaN, NaN).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 3, 5).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, '1.2', '2.5').join()"),
        String::from("\"1,4,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 'str').join()"),
        String::from("\"4,4,4\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 'str', 'str').join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, undefined, null).join()"),
        String::from("\"1,2,3\"")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, undefined, undefined).join()"),
        String::from("\"4,4,4\"")
    );

    assert_eq!(
        forward(&mut engine, "a.fill().join()"),
        String::from("\"undefined,undefined,undefined\"")
    );

    // test object reference
    forward(&mut engine, "a = (new Array(3)).fill({});");
    forward(&mut engine, "a[0].hi = 'hi';");
    assert_eq!(forward(&mut engine, "a[0].hi"), String::from("\"hi\""));
}

#[test]
fn includes_value() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        var undefined = [undefined];
        "#;
    eprintln!("{}", forward(&mut engine, init));

    // Empty
    let empty = forward(&mut engine, "empty.includes('a')");
    assert_eq!(empty, String::from("false"));

    // One
    let one = forward(&mut engine, "one.includes('a')");
    assert_eq!(one, String::from("true"));
    // Missing from one
    let missing_from_one = forward(&mut engine, "one.includes('b')");
    assert_eq!(missing_from_one, String::from("false"));

    // In many
    let first_in_many = forward(&mut engine, "many.includes('c')");
    assert_eq!(first_in_many, String::from("true"));
    // Missing from many
    let second_in_many = forward(&mut engine, "many.includes('d')");
    assert_eq!(second_in_many, String::from("false"));

    // In duplicates
    let first_in_many = forward(&mut engine, "duplicates.includes('a')");
    assert_eq!(first_in_many, String::from("true"));
    // Missing from duplicates
    let second_in_many = forward(&mut engine, "duplicates.includes('d')");
    assert_eq!(second_in_many, String::from("false"));
}

#[test]
fn map() {
    let mut engine = Context::new();

    let js = r#"
        var empty = [];
        var one = ["x"];
        var many = ["x", "y", "z"];

        // TODO: uncomment when `this` has been implemented
        // var _this = { answer: 42 };

        // function callbackThatUsesThis() {
        //      return 'The answer to life is: ' + this.answer;
        // }

        var empty_mapped = empty.map(v => v + '_');
        var one_mapped = one.map(v => '_' + v);
        var many_mapped = many.map(v => '_' + v + '_');
        "#;

    forward(&mut engine, js);

    // assert the old arrays have not been modified
    assert_eq!(forward(&mut engine, "one[0]"), String::from("\"x\""));
    assert_eq!(
        forward(&mut engine, "many[2] + many[1] + many[0]"),
        String::from("\"zyx\"")
    );

    // NB: These tests need to be rewritten once `Display` has been implemented for `Array`
    // Empty
    assert_eq!(
        forward(&mut engine, "empty_mapped.length"),
        String::from("0")
    );

    // One
    assert_eq!(forward(&mut engine, "one_mapped.length"), String::from("1"));
    assert_eq!(
        forward(&mut engine, "one_mapped[0]"),
        String::from("\"_x\"")
    );

    // Many
    assert_eq!(
        forward(&mut engine, "many_mapped.length"),
        String::from("3")
    );
    assert_eq!(
        forward(
            &mut engine,
            "many_mapped[0] + many_mapped[1] + many_mapped[2]"
        ),
        String::from("\"_x__y__z_\"")
    );

    // TODO: uncomment when `this` has been implemented
    // One but it uses `this` inside the callback
    // let one_with_this = forward(&mut engine, "one.map(callbackThatUsesThis, _this)[0];");
    // assert_eq!(one_with_this, String::from("The answer to life is: 42"))
}

#[test]
fn slice() {
    let mut engine = Context::new();
    let init = r#"
        var empty = [ ].slice();
        var one = ["a"].slice();
        var many1 = ["a", "b", "c", "d"].slice(1);
        var many2 = ["a", "b", "c", "d"].slice(2, 3);
        var many3 = ["a", "b", "c", "d"].slice(7);
        "#;
    eprintln!("{}", forward(&mut engine, init));

    assert_eq!(forward(&mut engine, "empty.length"), "0");
    assert_eq!(forward(&mut engine, "one[0]"), "\"a\"");
    assert_eq!(forward(&mut engine, "many1[0]"), "\"b\"");
    assert_eq!(forward(&mut engine, "many1[1]"), "\"c\"");
    assert_eq!(forward(&mut engine, "many1[2]"), "\"d\"");
    assert_eq!(forward(&mut engine, "many1.length"), "3");
    assert_eq!(forward(&mut engine, "many2[0]"), "\"c\"");
    assert_eq!(forward(&mut engine, "many2.length"), "1");
    assert_eq!(forward(&mut engine, "many3.length"), "0");
}

#[test]
fn for_each() {
    let mut engine = Context::new();
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
    eprintln!("{}", forward(&mut engine, init));

    assert_eq!(forward(&mut engine, "sum"), "14");
    assert_eq!(forward(&mut engine, "indexSum"), "6");
    assert_eq!(forward(&mut engine, "listLengthSum"), "16");
}

#[test]
fn for_each_push_value() {
    let mut engine = Context::new();
    let init = r#"
        var a = [1, 2, 3, 4];
        function callingCallback(item, index, list) {
            list.push(item * 2);
        }
        a.forEach(callingCallback);
        "#;
    eprintln!("{}", forward(&mut engine, init));

    // [ 1, 2, 3, 4, 2, 4, 6, 8 ]
    assert_eq!(forward(&mut engine, "a.length"), "8");
    assert_eq!(forward(&mut engine, "a[4]"), "2");
    assert_eq!(forward(&mut engine, "a[5]"), "4");
    assert_eq!(forward(&mut engine, "a[6]"), "6");
    assert_eq!(forward(&mut engine, "a[7]"), "8");
}

#[test]
fn filter() {
    let mut engine = Context::new();

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

    forward(&mut engine, js);

    // assert the old arrays have not been modified
    assert_eq!(forward(&mut engine, "one[0]"), String::from("\"1\""));
    assert_eq!(
        forward(&mut engine, "many[2] + many[1] + many[0]"),
        String::from("\"101\"")
    );

    // NB: These tests need to be rewritten once `Display` has been implemented for `Array`
    // Empty
    assert_eq!(
        forward(&mut engine, "empty_filtered.length"),
        String::from("0")
    );

    // One filtered on "1"
    assert_eq!(
        forward(&mut engine, "one_filtered.length"),
        String::from("1")
    );
    assert_eq!(
        forward(&mut engine, "one_filtered[0]"),
        String::from("\"1\"")
    );

    //  One filtered on "0"
    assert_eq!(
        forward(&mut engine, "zero_filtered.length"),
        String::from("0")
    );

    // Many filtered on "1"
    assert_eq!(
        forward(&mut engine, "many_one_filtered.length"),
        String::from("2")
    );
    assert_eq!(
        forward(&mut engine, "many_one_filtered[0] + many_one_filtered[1]"),
        String::from("\"11\"")
    );

    // Many filtered on "0"
    assert_eq!(
        forward(&mut engine, "many_zero_filtered.length"),
        String::from("1")
    );
    assert_eq!(
        forward(&mut engine, "many_zero_filtered[0]"),
        String::from("\"0\"")
    );
}

#[test]
fn some() {
    let mut engine = Context::new();
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
    forward(&mut engine, init);
    let result = forward(&mut engine, "array.some(lessThan10);");
    assert_eq!(result, "true");

    let result = forward(&mut engine, "empty.some(lessThan10);");
    assert_eq!(result, "false");

    let result = forward(&mut engine, "array.some(greaterThan10);");
    assert_eq!(result, "false");

    let result = forward(&mut engine, "appendArray.some(appendingCallback);");
    let append_array_length = forward(&mut engine, "appendArray.length");
    assert_eq!(append_array_length, "5");
    assert_eq!(result, "true");

    let result = forward(&mut engine, "delArray.some(deletingCallback);");
    let del_array_length = forward(&mut engine, "delArray.length");
    assert_eq!(del_array_length, "3");
    assert_eq!(result, "true");
}

#[test]
fn reduce() {
    let mut engine = Context::new();

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
    forward(&mut engine, init);

    // empty array
    let result = forward(&mut engine, "[].reduce(add, 0)");
    assert_eq!(result, "0");

    // simple with initial value
    let result = forward(&mut engine, "arr.reduce(add, 0)");
    assert_eq!(result, "10");

    // without initial value
    let result = forward(&mut engine, "arr.reduce(add)");
    assert_eq!(result, "10");

    // with some items missing
    let result = forward(&mut engine, "delArray.reduce(add, 0)");
    assert_eq!(result, "8");

    // with index
    let result = forward(&mut engine, "arr.reduce(addIdx, 0)");
    assert_eq!(result, "6");

    // with array
    let result = forward(&mut engine, "arr.reduce(addLen, 0)");
    assert_eq!(result, "16");

    // resizing the array as reduce progresses
    let result = forward(&mut engine, "arr.reduce(addResize, 0)");
    assert_eq!(result, "6");

    // Empty array
    let result = forward(
        &mut engine,
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
        "\"Reduce was called on an empty array and with no initial value\""
    );

    // Array with no defined elements
    let result = forward(
        &mut engine,
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
        "\"Reduce was called on an empty array and with no initial value\""
    );

    // No callback
    let result = forward(
        &mut engine,
        r#"
        try {
            arr.reduce("");
        } catch(e) {
            e.message
        }
    "#,
    );
    assert_eq!(result, "\"Reduce was called without a callback\"");
}

#[test]
fn reduce_right() {
    let mut engine = Context::new();

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
    forward(&mut engine, init);

    // empty array
    let result = forward(&mut engine, "[].reduceRight(sub, 0)");
    assert_eq!(result, "0");

    // simple with initial value
    let result = forward(&mut engine, "arr.reduceRight(sub, 0)");
    assert_eq!(result, "-10");

    // without initial value
    let result = forward(&mut engine, "arr.reduceRight(sub)");
    assert_eq!(result, "-2");

    // with some items missing
    let result = forward(&mut engine, "delArray.reduceRight(sub, 0)");
    assert_eq!(result, "-8");

    // with index
    let result = forward(&mut engine, "arr.reduceRight(subIdx)");
    assert_eq!(result, "1");

    // with array
    let result = forward(&mut engine, "arr.reduceRight(subLen)");
    assert_eq!(result, "-8");

    // resizing the array as reduce progresses
    let result = forward(&mut engine, "arr.reduceRight(subResize, 0)");
    assert_eq!(result, "-5");

    // reset array
    forward(&mut engine, "arr = [1, 2, 3, 4];");

    // resizing the array to 0 as reduce progresses
    let result = forward(&mut engine, "arr.reduceRight(subResize0, 0)");
    assert_eq!(result, "-7");

    // Empty array
    let result = forward(
        &mut engine,
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
        "\"reduceRight was called on an empty array and with no initial value\""
    );

    // Array with no defined elements
    let result = forward(
        &mut engine,
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
        "\"reduceRight was called on an empty array and with no initial value\""
    );

    // No callback
    let result = forward(
        &mut engine,
        r#"
        try {
            arr.reduceRight("");
        } catch(e) {
            e.message
        }
    "#,
    );
    assert_eq!(result, "\"reduceRight was called without a callback\"");
}

#[test]
fn call_array_constructor_with_one_argument() {
    let mut engine = Context::new();
    let init = r#"
        var empty = new Array(0);

        var five = new Array(5);

        var one = new Array("Hello, world!");
        "#;
    forward(&mut engine, init);
    // let result = forward(&mut engine, "empty.length");
    // assert_eq!(result, "0");

    // let result = forward(&mut engine, "five.length");
    // assert_eq!(result, "5");

    // let result = forward(&mut engine, "one.length");
    // assert_eq!(result, "1");
}
