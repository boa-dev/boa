// TODO: implement `Segmenter` when https://github.com/unicode-org/icu4x/issues/2259 closes.

use boa_profiler::Profiler;
use tap::{Conv, Pipe};

use crate::{builtins::BuiltIn, object::ConstructorBuilder, Context, JsResult, JsValue};

mod options;
#[allow(unused)]
pub(crate) use options::*;

#[derive(Debug, Clone)]
pub(crate) struct Segmenter;

impl BuiltIn for Segmenter {
    const NAME: &'static str = "Segmenter";

    fn init(context: &mut Context<'_>) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().segmenter().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Segmenter {
    pub(crate) const LENGTH: usize = 0;

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn constructor(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }
}
