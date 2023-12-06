#![allow(unused, unused_tuple_struct_fields)]

use boa_engine::value::TryFromJs;

#[derive(TryFromJs)]
struct TestStruct {
    inner: bool,
}

fn main() {}
