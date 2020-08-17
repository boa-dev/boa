pub static ARRAY_CREATE: &str = r#"
(function(){
    let testArr = [];
    for (let a = 0; a <= 500; a++) {
        testArr[a] = ('p' + a);
    }

    return testArr;
})();
"#;
