pub static STRING_COMPARE: &str = r#"
(function(){
    var a = "hello";
    var b = "world";

    var c = a == b;

    var d = b;
    var e = d == b;
})();
"#;
