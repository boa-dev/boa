use crate::property::PropertyKey;
use crate::value::RcString;
use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object},
    gc::{Finalize, Trace},
    object::{GcObject, ObjectData},
    property::{Attribute, DataDescriptor},
    BoaProfiler, Context, Result, Value,
};
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

/// The ForInIterator object represents an iteration over some specific object.
/// It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-for-in-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct ForInIterator {
    object: Value,
    visited_keys: FxHashSet<RcString>,
    remaining_keys: VecDeque<RcString>,
    object_was_visited: bool,
}

impl ForInIterator {
    pub(crate) const NAME: &'static str = "ForInIterator";

    fn new(object: Value) -> Self {
        ForInIterator {
            object,
            visited_keys: FxHashSet::default(),
            remaining_keys: VecDeque::default(),
            object_was_visited: false,
        }
    }

    /// CreateForInIterator( object )
    ///
    /// Creates a new iterator over the given object.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createforiniterator
    pub(crate) fn create_for_in_iterator(context: &Context, object: Value) -> Value {
        let for_in_iterator = Value::new_object(context);
        for_in_iterator.set_data(ObjectData::ForInIterator(Self::new(object)));
        for_in_iterator
            .as_object()
            .expect("for in iterator object")
            .set_prototype_instance(context.iterator_prototypes().for_in_iterator().into());
        for_in_iterator
    }

    /// %ForInIteratorPrototype%.next( )
    ///
    /// Gets the next result in the object.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%foriniteratorprototype%.next
    pub(crate) fn next(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        if let Value::Object(ref o) = this {
            let mut for_in_iterator = o.borrow_mut();
            if let Some(iterator) = for_in_iterator.as_for_in_iterator_mut() {
                let mut object = iterator.object.to_object(context)?;
                loop {
                    if !iterator.object_was_visited {
                        let keys = object.own_property_keys();
                        for k in keys {
                            match k {
                                PropertyKey::String(ref k) => {
                                    iterator.remaining_keys.push_back(k.clone());
                                }
                                PropertyKey::Index(i) => {
                                    iterator.remaining_keys.push_back(i.to_string().into());
                                }
                                _ => {}
                            }
                        }
                        iterator.object_was_visited = true;
                    }
                    while let Some(r) = iterator.remaining_keys.pop_front() {
                        if !iterator.visited_keys.contains(&r) {
                            if let Some(desc) =
                                object.get_own_property(&PropertyKey::from(r.clone()))
                            {
                                iterator.visited_keys.insert(r.clone());
                                if desc.enumerable() {
                                    return Ok(create_iter_result_object(
                                        context,
                                        Value::from(r.to_string()),
                                        false,
                                    ));
                                }
                            }
                        }
                    }
                    match object.prototype_instance().to_object(context) {
                        Ok(o) => {
                            object = o;
                        }
                        _ => {
                            return Ok(create_iter_result_object(context, Value::undefined(), true))
                        }
                    }
                    iterator.object = Value::from(object.clone());
                    iterator.object_was_visited = false;
                }
            } else {
                context.throw_type_error("`this` is not a ForInIterator")
            }
        } else {
            context.throw_type_error("`this` is not an ForInIterator")
        }
    }

    /// Create the %ArrayIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%foriniteratorprototype%-object
    pub(crate) fn create_prototype(context: &mut Context, iterator_prototype: Value) -> GcObject {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype
        let mut for_in_iterator = context.construct_object();
        make_builtin_fn(Self::next, "next", &for_in_iterator, 0, context);
        for_in_iterator.set_prototype_instance(iterator_prototype);

        let to_string_tag = context.well_known_symbols().to_string_tag_symbol();
        let to_string_tag_property =
            DataDescriptor::new("For In Iterator", Attribute::CONFIGURABLE);
        for_in_iterator.insert(to_string_tag, to_string_tag_property);
        for_in_iterator
    }
}
