use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object},
    object::{JsObject, ObjectData},
    property::PropertyDescriptor,
    property::PropertyKey,
    symbol::WellKnownSymbols,
    Context, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

/// The `ForInIterator` object represents an iteration over some specific object.
/// It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-for-in-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct ForInIterator {
    object: JsValue,
    visited_keys: FxHashSet<JsString>,
    remaining_keys: VecDeque<JsString>,
    object_was_visited: bool,
}

impl ForInIterator {
    pub(crate) const NAME: &'static str = "ForInIterator";

    fn new(object: JsValue) -> Self {
        Self {
            object,
            visited_keys: FxHashSet::default(),
            remaining_keys: VecDeque::default(),
            object_was_visited: false,
        }
    }

    /// `CreateForInIterator( object )`
    ///
    /// Creates a new iterator over the given object.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createforiniterator
    pub(crate) fn create_for_in_iterator(object: JsValue, context: &Context) -> JsValue {
        let for_in_iterator = JsObject::from_proto_and_data(
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .for_in_iterator(),
            ObjectData::for_in_iterator(Self::new(object)),
        );
        for_in_iterator.into()
    }

    /// %ForInIteratorPrototype%.next( )
    ///
    /// Gets the next result in the object.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%foriniteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let mut iterator = this.as_object().map(JsObject::borrow_mut);
        let iterator = iterator
            .as_mut()
            .and_then(|obj| obj.as_for_in_iterator_mut())
            .ok_or_else(|| context.construct_type_error("`this` is not a ForInIterator"))?;
        let mut object = iterator.object.to_object(context)?;
        loop {
            if !iterator.object_was_visited {
                let keys = object.__own_property_keys__(context)?;
                for k in keys {
                    match k {
                        PropertyKey::String(ref k) => {
                            iterator.remaining_keys.push_back(k.clone());
                        }
                        PropertyKey::Index(i) => {
                            iterator.remaining_keys.push_back(i.to_string().into());
                        }
                        PropertyKey::Symbol(_) => {}
                    }
                }
                iterator.object_was_visited = true;
            }
            while let Some(r) = iterator.remaining_keys.pop_front() {
                if !iterator.visited_keys.contains(&r) {
                    if let Some(desc) =
                        object.__get_own_property__(&PropertyKey::from(r.clone()), context)?
                    {
                        iterator.visited_keys.insert(r.clone());
                        if desc.expect_enumerable() {
                            return Ok(create_iter_result_object(
                                JsValue::new(r.to_string()),
                                false,
                                context,
                            ));
                        }
                    }
                }
            }
            let proto = object.prototype().clone();
            match proto {
                Some(o) => {
                    object = o;
                }
                _ => {
                    return Ok(create_iter_result_object(
                        JsValue::undefined(),
                        true,
                        context,
                    ))
                }
            }
            iterator.object = JsValue::new(object.clone());
            iterator.object_was_visited = false;
        }
    }

    /// Create the `%ArrayIteratorPrototype%` object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%foriniteratorprototype%-object
    pub(crate) fn create_prototype(
        iterator_prototype: JsObject,
        context: &mut Context,
    ) -> JsObject {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let for_in_iterator =
            JsObject::from_proto_and_data(iterator_prototype, ObjectData::ordinary());
        make_builtin_fn(Self::next, "next", &for_in_iterator, 0, context);

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let to_string_tag_property = PropertyDescriptor::builder()
            .value("For In Iterator")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        for_in_iterator.insert(to_string_tag, to_string_tag_property);
        for_in_iterator
    }
}
