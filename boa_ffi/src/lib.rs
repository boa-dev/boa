use boa_engine::Context;

pub fn boa_exec(src: &str) -> String {
    let src_bytes: &[u8] = src.as_ref();

    match Context::default().eval(src_bytes) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let scenario = r#"
        function abc() {}
        "#;

        assert_eq!(&boa_exec(scenario), "undefined");
    }
}
