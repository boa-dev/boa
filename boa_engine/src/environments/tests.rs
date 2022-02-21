use crate::exec;

#[test]
fn let_is_block_scoped() {
    let scenario = r#"
          {
            let bar = "bar";
          }

          try{
            bar;
          } catch (err) {
            err.message
          }
        "#;

    assert_eq!(&exec(scenario), "\"bar is not defined\"");
}

#[test]
fn const_is_block_scoped() {
    let scenario = r#"
          {
            const bar = "bar";
          }

          try{
            bar;
          } catch (err) {
            err.message
          }
        "#;

    assert_eq!(&exec(scenario), "\"bar is not defined\"");
}

#[test]
fn var_not_block_scoped() {
    let scenario = r#"
          {
            var bar = "bar";
          }
          bar == "bar";
        "#;

    assert_eq!(&exec(scenario), "true");
}

#[test]
fn functions_use_declaration_scope() {
    let scenario = r#"
          function foo() {
            try {
                bar;
            } catch (err) {
                return err.message;
            }
          }
          {
            let bar = "bar";
            foo();
          }
        "#;

    assert_eq!(&exec(scenario), "\"bar is not defined\"");
}

#[test]
fn set_outer_var_in_block_scope() {
    let scenario = r#"
          var bar;
          {
            bar = "foo";
          }
          bar == "foo";
        "#;

    assert_eq!(&exec(scenario), "true");
}

#[test]
fn set_outer_let_in_block_scope() {
    let scenario = r#"
          let bar;
          {
            bar = "foo";
          }
          bar == "foo";
        "#;

    assert_eq!(&exec(scenario), "true");
}
