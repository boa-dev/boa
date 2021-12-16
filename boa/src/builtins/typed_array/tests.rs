use crate::{forward, forward_val, Context};

#[test]
fn from() {
    let mut context = Context::new();
    let init = r#"
        const uint16 = Int16Array.from('12345');
        const s = new Set([1, 2, 3]);
        const uint8 = Uint8Array.from(s);
        const f32 = Float32Array.from([1, 2, 3], x => x + x);
        const uint8_seq = Uint8Array.from({length: 5}, (v, k) => k);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint16.length"), "5");
    for i in 0..5 {
        assert_eq!(
            forward_val(&mut context, format!("uint16[{}]", i))
                .unwrap()
                .to_uint16(&mut context)
                .unwrap(),
            i + 1
        );
    }
    assert_eq!(forward(&mut context, "uint8.length"), "3");
    for i in 0..3 {
        assert_eq!(
            forward_val(&mut context, format!("uint8[{}]", i))
                .unwrap()
                .to_uint8(&mut context)
                .unwrap(),
            i + 1
        );
    }
    assert_eq!(forward(&mut context, "f32.length"), "3");
    for i in 0..3 {
        assert!(float_cmp::approx_eq!(
            f64,
            forward_val(&mut context, format!("f32[{}]", i))
                .unwrap()
                .to_numeric_number(&mut context)
                .unwrap(),
            (2 * (i + 1)) as f64
        ));
    }
    assert_eq!(forward(&mut context, "uint8_seq.length"), "5");
    for i in 0..5 {
        assert_eq!(
            forward_val(&mut context, format!("uint8_seq[{}]", i))
                .unwrap()
                .to_uint8(&mut context)
                .unwrap(),
            i
        );
    }
}

#[test]
fn of() {
    let mut context = Context::new();
    let init = r#"
        let int16array = new Int16Array;
        int16array = Int16Array.of('10', '20', '30', '40', '50');
        let u8 = Uint8Array.of(1);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "int16array.length"), "5");
    for i in 0..5 {
        assert_eq!(
            forward_val(&mut context, format!("int16array[{}]", i))
                .unwrap()
                .to_int16(&mut context)
                .unwrap(),
            (i + 1) * 10
        );
    }
    assert_eq!(forward(&mut context, "u8.length"), "1");
    assert_eq!(
        forward_val(&mut context, "u8[0]")
            .unwrap()
            .to_uint8(&mut context)
            .unwrap(),
        1
    );
}

#[test]
fn at() {
    let mut context = Context::new();
    let init = r#"
        const int8 = new Int8Array([0, 10, -10, 20, -30, 40, -50]);
        let index = 1;
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "int8.at(index)"), "10");
    forward(&mut context, "index = -2");
    assert_eq!(forward(&mut context, "int8.at(index)"), "40");
    let ind_min_3 = forward(&mut context, "int8.at(-3)");
    assert_eq!(ind_min_3, forward(&mut context, "int8[int8.length-3]"));
    assert_eq!(ind_min_3, forward(&mut context, "int8.slice(-3, -2)[0]"));
    assert!(forward_val(&mut context, "int8.at('infinity')")
        .unwrap()
        .is_undefined());
    assert!(forward_val(&mut context, "int8.at(7)")
        .unwrap()
        .is_undefined());
}

#[test]
fn buffer() {
    let mut context = Context::new();
    let init = r#"
        const buffer = new ArrayBuffer(8);
        const uint16 = new Uint16Array(buffer);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint16.buffer.byteLength"), "8");
}

#[test]
fn byte_length() {
    let mut context = Context::new();
    let init = r#"
        var buffer = new ArrayBuffer(8);
        const uint8_a = new Uint8Array(buffer);
        const uint8_b = new Uint8Array(buffer, 1, 5);
        const uint8_c = new Uint8Array(buffer, 2);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint8_a.byteLength"), "8");
    assert_eq!(forward(&mut context, "uint8_b.byteLength"), "5");
    assert_eq!(forward(&mut context, "uint8_c.byteLength"), "6");
}

#[test]
fn byte_offset() {
    let mut context = Context::new();
    let init = r#"
        var buffer = new ArrayBuffer(8);
        const uint8_a = new Uint8Array(buffer);
        const uint8_b = new Uint8Array(buffer, 3);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint8_a.byteOffset"), "0");
    assert_eq!(forward(&mut context, "uint8_b.byteOffset"), "3");
}

#[test]
fn length() {
    let mut context = Context::new();
    let init = r#"const buffer = new ArrayBuffer(8);
        const uint8 = new Uint8Array(buffer, 2);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint8.length"), "6");
}

#[test]
fn copy_within() {
    let mut context = Context::new();
    let init = r#"
        var buffer = new ArrayBuffer(8);
        var uint8 = new Uint8Array(buffer);
        uint8.set([1,2,3]);
        "#;
    forward(&mut context, init);
    let v1 = vec![1, 2, 3, 0, 0, 0, 0, 0];
    for i in 0..8 {
        assert_eq!(
            forward_val(&mut context, format!("uint8[{}]", i))
                .unwrap()
                .to_uint8(&mut context)
                .unwrap(),
            v1[i]
        );
    }
    forward(&mut context, "uint8.copyWithin(3,0,3);");
    let v2 = vec![1, 2, 3, 1, 2, 3, 0, 0];
    for i in 0..8 {
        assert_eq!(
            forward_val(&mut context, format!("uint8[{}]", i))
                .unwrap()
                .to_uint8(&mut context)
                .unwrap(),
            v2[i]
        );
    }
}

#[test]
fn entries() {
    let mut context = Context::new();
    let init = r#"
        var arr = new Uint8Array([10, 20, 30, 40, 50]);
        var eArr = arr.entries();
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "eArr.next().value"), "[ 0, 10 ]");
    assert_eq!(forward(&mut context, "eArr.next().value"), "[ 1, 20 ]");
    assert_eq!(forward(&mut context, "eArr.next().value"), "[ 2, 30 ]");
    assert_eq!(forward(&mut context, "eArr.next().value"), "[ 3, 40 ]");
    assert_eq!(forward(&mut context, "eArr.next().value"), "[ 4, 50 ]");
    assert!(forward_val(&mut context, "eArr.next().value")
        .unwrap()
        .is_undefined());
}

#[test]
fn every() {
    let mut context = Context::new();
    let init = r#"
        function isBigEnough(element, index, array) {
          return element >= 10;
        }
        "#;
    forward(&mut context, init);
    assert_eq!(
        forward(
            &mut context,
            "new Uint8Array([12, 5, 8, 130, 44]).every(isBigEnough)"
        ),
        "false"
    );
    assert_eq!(
        forward(
            &mut context,
            "new Uint8Array([12, 54, 18, 130, 44]).every(isBigEnough)"
        ),
        "true"
    );
    assert_eq!(
        forward(
            &mut context,
            "new Uint8Array([12, 5, 8, 130, 44]).every(elem => elem >= 10)"
        ),
        "false"
    );
    assert_eq!(
        forward(
            &mut context,
            "new Uint8Array([12, 54, 18, 130, 44]).every(elem => elem >= 10)"
        ),
        "true"
    );
}

#[test]
fn fill() {
    let mut context = Context::new();
    let init = r#"
        const a = new Uint8Array([1, 2, 3]).fill(4);
        const b = new Uint8Array([1, 2, 3]).fill(4, 1);
        const c = new Uint8Array([1, 2, 3]).fill(4, 1, 2);
        const d = new Uint8Array([1, 2, 3]).fill(4, 1, 1);
        const e = new Uint8Array([1, 2, 3]).fill(4, -3, -2);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "a.join()"), "\"4,4,4\"");
    assert_eq!(forward(&mut context, "b.join()"), "\"1,4,4\"");
    assert_eq!(forward(&mut context, "c.join()"), "\"1,4,3\"");
    assert_eq!(forward(&mut context, "d.join()"), "\"1,2,3\"");
    assert_eq!(forward(&mut context, "e.join()"), "\"4,2,3\"");
}

#[test]
fn filter() {
    let mut context = Context::new();
    let init = r#"
        function isBigEnough(element, index, array) {
          return element >= 10;
        }
        const u8 = new Uint8Array([12, 5, 8, 130, 44]).filter(isBigEnough);"#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "u8.join()"), "\"12,130,44\"");
    assert_eq!(
        forward(
            &mut context,
            "new Uint8Array([12, 5, 8, 130, 44]).filter(elem => elem >= 10).join()"
        ),
        "\"12,130,44\""
    );
}

#[test]
fn find() {
    let mut context = Context::new();
    let init = r#"
        function isPrime(element, index, array) {
            var start = 2;
            while (start <= Math.sqrt(element)) {
                if (element % start++ < 1) {
                  return false;
                }
            }
            return element > 1;
        }
        const uint8 = new Uint8Array([4, 5, 8, 12]);
        const int8 = new Int8Array([10, 0, -10, 20, -30, 40, -50]);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint8.find(isPrime)"), "5");
    assert_eq!(
        forward(&mut context, "int8.find((element) => element < 0)"),
        "-10"
    );
    assert_eq!(
        forward(
            &mut context,
            "int8.find((element,index) => element < 0 && index > 4)"
        ),
        "-50"
    );
    assert!(forward_val(
        &mut context,
        "int8.find((element,index, array) => element < 0 && array.length < index)"
    )
    .unwrap()
    .is_undefined());
}

#[test]
fn find_index() {
    let mut context = Context::new();
    let init = r#"
        function isPrime(element, index, array) {
          var start = 2;
          while (start <= Math.sqrt(element)) {
            if (element % start++ < 1) {
              return false;
            }
          }
          return element > 1;
        }
        const uint8 = new Uint8Array([4, 6, 8, 12]);
        const uint16 = new Uint16Array([4, 6, 7, 12]);
        const int8 = new Int8Array([10, 0, -10, 20, -30, 40, -50]);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint8.findIndex(isPrime)"), "-1");
    assert_eq!(forward(&mut context, "uint16.findIndex(isPrime)"), "2");
    assert_eq!(
        forward(&mut context, "int8.findIndex((element) => element < 0)"),
        "2"
    );
    assert_eq!(
        forward(
            &mut context,
            "int8.findIndex((element,index) => element < 0 && index > 4)"
        ),
        "6"
    );
    assert_eq!(
        forward(
            &mut context,
            "int8.findIndex((element,index, array) => element < 0 && array.length < index)"
        ),
        "-1"
    );
}

#[test]
fn foreach() {
    let mut context = Context::new();
    let init = r#"
        function doubleArrayElements(element, index, array) {
          array[index] *= 2;
        }
        x = new Uint8Array([0, 1, 2, 3]);
        "#;
    forward(&mut context, init);
    forward(&mut context, "x.forEach(doubleArrayElements);");
    assert_eq!(forward(&mut context, "x.join()"), "\"0,2,4,6\"");
    forward(&mut context, "x.forEach((element) => { element *= 2; });");
    assert_eq!(forward(&mut context, "x.join()"), "\"0,2,4,6\"")
}

#[test]
fn includes() {
    let mut context = Context::new();
    let init = r#"
        const uint8 = new Uint8Array([10, 20, 30, 40, 50]);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint8.includes(20)"), "true");
    assert_eq!(forward(&mut context, "uint8.includes(20, 3)"), "false");
}

#[test]
fn index_of() {
    let mut context = Context::new();
    let init = r#"
        var uint8 = new Uint8Array([2, 5, 9]);
        "#;
    forward(&mut context, init);
    assert_eq!(forward(&mut context, "uint8.indexOf(2);"), "0");
    assert_eq!(forward(&mut context, "uint8.indexOf(7);"), "-1");
    assert_eq!(forward(&mut context, "uint8.indexOf(9, 2);"), "2");
    assert_eq!(forward(&mut context, "uint8.indexOf(2, -1);"), "-1");
    assert_eq!(forward(&mut context, "uint8.indexOf(2, -3);"), "0");
}
