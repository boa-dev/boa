use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use itertools::Itertools;

use crate::{
    builtins::{BuiltInBuilder, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    object::ObjectData,
    realm::Realm,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};

use super::{create_segment_data_object, SegmentIterator};

#[derive(Debug, Trace, Finalize)]
pub struct Segments {
    segmenter: JsObject,
    string: JsString,
}

impl IntrinsicObject for Segments {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event("%SegmentsPrototype%", "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(Self::containing, "containing", 1)
            .static_method(
                Self::iterator,
                (JsSymbol::iterator(), js_string!("[Symbol.iterator]")),
                0,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().segments_prototype()
    }
}

impl Segments {
    /// [`CreateSegmentsObject ( segmenter, string )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-createsegmentsobject
    pub(crate) fn create(
        segmenter: JsObject,
        string: JsString,
        context: &mut dyn Context<'_>,
    ) -> JsObject {
        // 1. Let internalSlotsList be « [[SegmentsSegmenter]], [[SegmentsString]] ».
        // 2. Let segments be OrdinaryObjectCreate(%SegmentsPrototype%, internalSlotsList).
        // 3. Set segments.[[SegmentsSegmenter]] to segmenter.
        // 4. Set segments.[[SegmentsString]] to string.
        // 5. Return segments.
        JsObject::from_proto_and_data(
            context.intrinsics().objects().segments_prototype(),
            ObjectData::segments(Self { segmenter, string }),
        )
    }

    /// [`%SegmentsPrototype%.containing ( index )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-%segmentsprototype%.containing
    fn containing(
        this: &JsValue,
        args: &[JsValue],
        context: &mut dyn Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let segments be the this value.
        // 2. Perform ? RequireInternalSlot(segments, [[SegmentsSegmenter]]).
        let segments = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`containing` can only be called on a `Segments` object")
        })?;
        let segments = segments.as_segments().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`containing` can only be called on a `Segments` object")
        })?;

        // 3. Let segmenter be segments.[[SegmentsSegmenter]].
        let segmenter = segments.segmenter.borrow();
        let segmenter = segmenter
            .as_segmenter()
            .expect("segments object should contain a segmenter");

        // 4. Let string be segments.[[SegmentsString]].
        // 5. Let len be the length of string.
        let len = segments.string.len() as i64;

        // 6. Let n be ? ToIntegerOrInfinity(index).
        let Some(n) = args
            .get_or_undefined(0)
            .to_integer_or_infinity(context)?
            .as_integer()
            // 7. If n < 0 or n ≥ len, return undefined.
            .filter(|i| (0..len).contains(i))
            .map(|n| n as usize) else {
            return Ok(JsValue::undefined());
        };

        // 8. Let startIndex be ! FindBoundary(segmenter, string, n, before).
        // 9. Let endIndex be ! FindBoundary(segmenter, string, n, after).
        let (range, is_word_like) = {
            let mut segments = segmenter.native.segment(&segments.string);
            std::iter::from_fn(|| segments.next().map(|i| (i, segments.is_word_like())))
                .tuple_windows()
                .find(|((i, _), (j, _))| (*i..*j).contains(&n))
                .map(|((i, _), (j, word))| ((i..j), word))
                .expect("string should have at least a length of 1, and `n` must be in range")
        };

        // 10. Return ! CreateSegmentDataObject(segmenter, string, startIndex, endIndex).
        Ok(
            create_segment_data_object(segments.string.clone(), range, is_word_like, context)
                .into(),
        )
    }

    /// [`%SegmentsPrototype% [ @@iterator ] ( )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-%segmentsprototype%-@@iterator
    fn iterator(this: &JsValue, _: &[JsValue], context: &mut dyn Context<'_>) -> JsResult<JsValue> {
        // 1. Let segments be the this value.
        // 2. Perform ? RequireInternalSlot(segments, [[SegmentsSegmenter]]).
        let segments = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`containing` can only be called on a `Segments` object")
        })?;
        let segments = segments.as_segments().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`containing` can only be called on a `Segments` object")
        })?;

        // 3. Let segmenter be segments.[[SegmentsSegmenter]].
        // 4. Let string be segments.[[SegmentsString]].
        // 5. Return ! CreateSegmentIterator(segmenter, string).
        Ok(
            SegmentIterator::create(segments.segmenter.clone(), segments.string.clone(), context)
                .into(),
        )
    }
}
