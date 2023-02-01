// TODO: implement `Segmenter` when https://github.com/unicode-org/icu4x/issues/2259 closes.

use boa_profiler::Profiler;

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::JsObject,
    Context, JsResult, JsValue,
};

mod options;
#[allow(unused)]
pub(crate) use options::*;

#[derive(Debug, Clone)]
pub(crate) struct Segmenter;

impl IntrinsicObject for Segmenter {
    fn init(intrinsics: &Intrinsics) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor::<Self>(intrinsics).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Segmenter {
    const NAME: &'static str = "Segmenter";
}

impl BuiltInConstructor for Segmenter {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::segmenter;

    #[allow(clippy::unnecessary_wraps)]
    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }
}
