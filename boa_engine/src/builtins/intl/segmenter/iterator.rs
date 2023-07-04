use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use icu_segmenter::{
    GraphemeClusterBreakIteratorUtf16, SentenceBreakIteratorUtf16, WordBreakIteratorUtf16,
};

use crate::{
    builtins::{iterable::create_iter_result_object, BuiltInBuilder, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    object::ObjectData,
    property::Attribute,
    realm::Realm,
    Context, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};

use super::create_segment_data_object;

pub(crate) enum NativeSegmentIterator<'l, 's> {
    Grapheme(GraphemeClusterBreakIteratorUtf16<'l, 's>),
    Word(WordBreakIteratorUtf16<'l, 's>),
    Sentence(SentenceBreakIteratorUtf16<'l, 's>),
}

impl Iterator for NativeSegmentIterator<'_, '_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NativeSegmentIterator::Grapheme(g) => g.next(),
            NativeSegmentIterator::Word(w) => w.next(),
            NativeSegmentIterator::Sentence(s) => s.next(),
        }
    }
}

impl NativeSegmentIterator<'_, '_> {
    /// If the iterator is a word break iterator, returns `Some(true)` when the segment preceding
    /// the current boundary is word-like.
    pub(crate) fn is_word_like(&self) -> Option<bool> {
        if let Self::Word(w) = self {
            Some(w.is_word_like())
        } else {
            None
        }
    }
}

#[derive(Debug, Trace, Finalize)]
pub struct SegmentIterator {
    segmenter: JsObject,
    string: JsString,
    next_segment_index: usize,
}

impl IntrinsicObject for SegmentIterator {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event("%SegmentIteratorPrototype%", "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Segmenter String Iterator"),
                Attribute::CONFIGURABLE,
            )
            .static_method(Self::next, js_string!("next"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().segment()
    }
}

impl SegmentIterator {
    /// [`CreateSegmentIterator ( segmenter, string )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-createsegmentiterator
    pub(crate) fn create(
        segmenter: JsObject,
        string: JsString,
        context: &mut Context<'_>,
    ) -> JsObject {
        // 1. Let internalSlotsList be « [[IteratingSegmenter]], [[IteratedString]], [[IteratedStringNextSegmentCodeUnitIndex]] ».
        // 2. Let iterator be OrdinaryObjectCreate(%SegmentIteratorPrototype%, internalSlotsList).
        // 3. Set iterator.[[IteratingSegmenter]] to segmenter.
        // 4. Set iterator.[[IteratedString]] to string.
        // 5. Set iterator.[[IteratedStringNextSegmentCodeUnitIndex]] to 0.
        // 6. Return iterator.
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .segment(),
            ObjectData::segment_iterator(Self {
                segmenter,
                string,
                next_segment_index: 0,
            }),
        )
    }
    /// [`%SegmentIteratorPrototype%.next ( )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-%segmentiteratorprototype%.next
    fn next(this: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let iterator be the this value.
        // 2. Perform ? RequireInternalSlot(iterator, [[IteratingSegmenter]]).
        let mut iter = this.as_object().map(JsObject::borrow_mut).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`next` can only be called on a `Segment Iterator` object")
        })?;
        let iter = iter.as_segment_iterator_mut().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`next` can only be called on a `Segment Iterator` object")
        })?;

        // 5. Let startIndex be iterator.[[IteratedStringNextSegmentCodeUnitIndex]].
        let start = iter.next_segment_index;

        // 4. Let string be iterator.[[IteratedString]].
        // 6. Let endIndex be ! FindBoundary(segmenter, string, startIndex, after).
        let Some((end, is_word_like)) = iter.string.get(start..).and_then(|string| {
            // 3. Let segmenter be iterator.[[IteratingSegmenter]].
            let segmenter = iter.segmenter.borrow();
            let segmenter = segmenter
                .as_segmenter()
                .expect("segment iterator object should contain a segmenter");
            let mut segments = segmenter.native.segment(string);
            // the first elem is always 0.
            segments.next();
            segments
                .next()
                .map(|end| (start + end, segments.is_word_like()))
        }) else {
            // 7. If endIndex is not finite, then
            //     a. Return CreateIterResultObject(undefined, true).
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        };
        // 8. Set iterator.[[IteratedStringNextSegmentCodeUnitIndex]] to endIndex.
        iter.next_segment_index = end;

        // 9. Let segmentData be ! CreateSegmentDataObject(segmenter, string, startIndex, endIndex).
        let segment_data =
            create_segment_data_object(iter.string.clone(), start..end, is_word_like, context);

        // 10. Return CreateIterResultObject(segmentData, false).
        Ok(create_iter_result_object(
            segment_data.into(),
            false,
            context,
        ))
    }
}
