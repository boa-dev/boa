use crate::{exec::Interpreter, forward, realm::Realm};

#[test]
fn is_array() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var empty = [];
        var new_arr = new Array();
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(forward(&mut engine, "Array.isArray(empty)"), "true");
    assert_eq!(forward(&mut engine, "Array.isArray(new_arr)"), "true");
    assert_eq!(forward(&mut engine, "Array.isArray(many)"), "true");
    assert_eq!(forward(&mut engine, "Array.isArray([1, 2, 3])"), "true");
    assert_eq!(forward(&mut engine, "Array.isArray([])"), "true");
    assert_eq!(forward(&mut engine, "Array.isArray({})"), "false");
    // assert_eq!(forward(&mut engine, "Array.isArray(new Array)"), "true");
    assert_eq!(forward(&mut engine, "Array.isArray()"), "false");
    assert_eq!(
        forward(&mut engine, "Array.isArray({ constructor: Array })"),
        "false"
    );
    assert_eq!(
        forward(
            &mut engine,
            "Array.isArray({ push: Array.prototype.push, concat: Array.prototype.concat })"
        ),
        "false"
    );
    assert_eq!(forward(&mut engine, "Array.isArray(17)"), "false");
    assert_eq!(
        forward(&mut engine, "Array.isArray({ __proto__: Array.prototype })"),
        "false"
    );
    assert_eq!(
        forward(&mut engine, "Array.isArray({ length: 0 })"),
        "false"
    );
}

#[test]
fn concat() {
    //TODO: array display formatter
    // let realm = Realm::create();
    // let mut engine = Interpreter::new(realm);
    // let init = r#"
    // var empty = new Array();
    // var one = new Array(1);
    // "#;
    // eprintln!("{}", forward(&mut engine, init));
    // // Empty ++ Empty
    // let ee = forward(&mut engine, "empty.concat(empty)");
    // assert_eq!(ee, String::from("[]"));
    // // Empty ++ NonEmpty
    // let en = forward(&mut engine, "empty.concat(one)");
    // assert_eq!(en, String::from("[a]"));
    // // NonEmpty ++ Empty
    // let ne = forward(&mut engine, "one.concat(empty)");
    // assert_eq!(ne, String::from("a.b.c"));
    // // NonEmpty ++ NonEmpty
    // let nn = forward(&mut engine, "one.concat(one)");
    // assert_eq!(nn, String::from("a.b.c"));
}

#[test]
fn join() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut engine, init));
    // Empty
    let empty = forward(&mut engine, "empty.join('.')");
    assert_eq!(empty, String::from(""));
    // One
    let one = forward(&mut engine, "one.join('.')");
    assert_eq!(one, String::from("a"));
    // Many
    let many = forward(&mut engine, "many.join('.')");
    assert_eq!(many, String::from("a.b.c"));
}

#[test]
fn to_string() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut engine, init));
    // Empty
    let empty = forward(&mut engine, "empty.toString()");
    assert_eq!(empty, String::from(""));
    // One
    let one = forward(&mut engine, "one.toString()");
    assert_eq!(one, String::from("a"));
    // Many
    let many = forward(&mut engine, "many.toString()");
    assert_eq!(many, String::from("a,b,c"));
}

#[test]
fn every() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        function comp(a) {
            return a == "a";
        }
        var many = ["a", "b", "c"];
        "#;
    eprintln!("{}", forward(&mut engine, init));
    let found = forward(&mut engine, "many.find(comp)");
    assert_eq!(found, String::from("a"));
}

#[test]
fn find_index() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
fn fill() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    forward(&mut engine, "var a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4).join()"),
        String::from("4,4,4")
    );
    // make sure the array is modified
    assert_eq!(forward(&mut engine, "a.join()"), String::from("4,4,4"));

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, '1').join()"),
        String::from("1,4,4")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 1, 2).join()"),
        String::from("1,4,3")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 1, 1).join()"),
        String::from("1,2,3")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 3, 3).join()"),
        String::from("1,2,3")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, -3, -2).join()"),
        String::from("4,2,3")
    );

    // TODO: uncomment when NaN support is added
    // forward(&mut engine, "a = [1, 2, 3];");
    // assert_eq!(
    //     forward(&mut engine, "a.fill(4, NaN, NaN).join()"),
    //     String::from("1,2,3")
    // );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 3, 5).join()"),
        String::from("1,2,3")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, '1.2', '2.5').join()"),
        String::from("1,4,3")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 'str').join()"),
        String::from("4,4,4")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, 'str', 'str').join()"),
        String::from("1,2,3")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, undefined, null).join()"),
        String::from("1,2,3")
    );

    forward(&mut engine, "a = [1, 2, 3];");
    assert_eq!(
        forward(&mut engine, "a.fill(4, undefined, undefined).join()"),
        String::from("4,4,4")
    );

    assert_eq!(
        forward(&mut engine, "a.fill().join()"),
        String::from("undefined,undefined,undefined")
    );

    // test object reference
    forward(&mut engine, "a = (new Array(3)).fill({});");
    forward(&mut engine, "a[0].hi = 'hi';");
    assert_eq!(forward(&mut engine, "a[0].hi"), String::from("hi"));
}

#[test]
fn includes_value() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

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
    assert_eq!(forward(&mut engine, "one[0]"), String::from("x"));
    assert_eq!(
        forward(&mut engine, "many[2] + many[1] + many[0]"),
        String::from("zyx")
    );

    // NB: These tests need to be rewritten once `Display` has been implemented for `Array`
    // Empty
    assert_eq!(
        forward(&mut engine, "empty_mapped.length"),
        String::from("0")
    );

    // One
    assert_eq!(forward(&mut engine, "one_mapped.length"), String::from("1"));
    assert_eq!(forward(&mut engine, "one_mapped[0]"), String::from("_x"));

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
        String::from("_x__y__z_")
    );

    // TODO: uncomment when `this` has been implemented
    // One but it uses `this` inside the callback
    // let one_with_this = forward(&mut engine, "one.map(callbackThatUsesThis, _this)[0];");
    // assert_eq!(one_with_this, String::from("The answer to life is: 42"))
}

#[test]
fn slice() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var empty = [ ].slice();
        var one = ["a"].slice();
        var many1 = ["a", "b", "c", "d"].slice(1);
        var many2 = ["a", "b", "c", "d"].slice(2, 3);
        var many3 = ["a", "b", "c", "d"].slice(7);
        "#;
    eprintln!("{}", forward(&mut engine, init));

    assert_eq!(forward(&mut engine, "empty.length"), "0");
    assert_eq!(forward(&mut engine, "one[0]"), "a");
    assert_eq!(forward(&mut engine, "many1[0]"), "b");
    assert_eq!(forward(&mut engine, "many1[1]"), "c");
    assert_eq!(forward(&mut engine, "many1[2]"), "d");
    assert_eq!(forward(&mut engine, "many1.length"), "3");
    assert_eq!(forward(&mut engine, "many2[0]"), "c");
    assert_eq!(forward(&mut engine, "many2.length"), "1");
    assert_eq!(forward(&mut engine, "many3.length"), "0");
}

#[test]
fn for_each() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

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
    assert_eq!(forward(&mut engine, "one[0]"), String::from("1"));
    assert_eq!(
        forward(&mut engine, "many[2] + many[1] + many[0]"),
        String::from("101")
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
    assert_eq!(forward(&mut engine, "one_filtered[0]"), String::from("1"));

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
        String::from("11")
    );

    // Many filtered on "0"
    assert_eq!(
        forward(&mut engine, "many_zero_filtered.length"),
        String::from("1")
    );
    assert_eq!(
        forward(&mut engine, "many_zero_filtered[0]"),
        String::from("0")
    );
}

#[test]
fn some() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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
fn call_array_constructor_with_one_argument() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var empty = new Array(0);

        var five = new Array(5);

        var one = new Array("Hello, world!");
        "#;
    forward(&mut engine, init);
    let result = forward(&mut engine, "empty.length");
    assert_eq!(result, "0");

    let result = forward(&mut engine, "five.length");
    assert_eq!(result, "5");

    let result = forward(&mut engine, "one.length");
    assert_eq!(result, "1");
}
