//! Boa's implementation of the ECMAScript `Iterator` constructor.
//!
//! The `Iterator` constructor is designed to be subclassed. It may be used as the
//! value of an extends clause of a class definition.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-iterator-constructor

use std::collections::VecDeque;

use crate::{
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        iterable::iterator_helper::{self, IterableRecord},
        object::OrdinaryObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_error, js_string,
    object::{JsFunction, JsObject, PROTOTYPE, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

use super::{iterator_helper::IteratorHelper, wrap_for_valid_iterator::WrapForValidIterator};

#[cfg(feature = "experimental")]
use super::{
    IteratorHint,
    zip_iterator::{ZipIterator, ZipMode, ZipResultKind},
};

/// The `Iterator` constructor.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator-constructor
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub(crate) struct IteratorConstructor;

impl IntrinsicObject for IteratorConstructor {
    fn init(realm: &Realm) {
        let iterator_prototype = realm.intrinsics().constructors().iterator().prototype();
        let builder = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(iterator_prototype.clone()))
            // Static methods
            .static_method(Self::from, js_string!("from"), 1)
            .static_method(Self::concat, js_string!("concat"), 0);

        #[cfg(feature = "experimental")]
        let builder = builder
            .static_method(Self::zip, js_string!("zip"), 1)
            .static_method(Self::zip_keyed, js_string!("zipKeyed"), 1);

        builder
            .static_property(PROTOTYPE, iterator_prototype, Attribute::empty())
            .build_without_prototype();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().iterator().constructor()
    }
}

impl BuiltInObject for IteratorConstructor {
    const NAME: JsString = StaticJsStrings::ITERATOR;
}

impl BuiltInConstructor for IteratorConstructor {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    #[cfg(not(feature = "experimental"))]
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 3;
    #[cfg(feature = "experimental")]
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 5;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::iterator;

    /// `Iterator ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator
    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined or the active function object, throw a TypeError exception.
        if new_target.is_undefined()
            || new_target
                == &context
                    .active_function_object()
                    .unwrap_or_else(|| context.intrinsics().constructors().iterator().constructor())
                    .into()
        {
            return Err(JsNativeError::typ()
                .with_message(if new_target.is_undefined() {
                    "Iterator constructor requires 'new'"
                } else {
                    "Abstract class Iterator not directly constructable"
                })
                .into());
        }

        // 2. Return ? OrdinaryCreateFromConstructor(NewTarget, "%Iterator.prototype%").
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::iterator, context)?;

        // Create an ordinary object (Iterator instances have no internal data slots).
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrdinaryObject,
        )
        .upcast()
        .into())
    }
}

impl IteratorConstructor {
    /// `Iterator.from ( O )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.from
    fn from(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);

        // 1. Let iteratorRecord be ? GetIteratorFlattenable(O, iterate-strings).
        let iterator_record = super::get_iterator_flattenable(o, true, context)?;

        // 2. Let hasInstance be ? OrdinaryHasInstance(%Iterator%, iteratorRecord.[[Iterator]]).
        let iterator_constructor = context.intrinsics().constructors().iterator().constructor();
        let has_instance = JsValue::ordinary_has_instance(
            &iterator_constructor.clone().into(),
            &iterator_record.iterator().clone().into(),
            context,
        )?;

        // 3. If hasInstance is true, then
        if has_instance {
            // a. Return iteratorRecord.[[Iterator]].
            return Ok(iterator_record.iterator().clone().into());
        }

        // 4. Let wrapper be OrdinaryObjectCreate(%WrapForValidIteratorPrototype%, « [[Iterated]] »).
        // 5. Set wrapper.[[Iterated]] to iteratorRecord.
        // 6. Return wrapper.
        let wrapper = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .wrap_for_valid_iterator(),
            WrapForValidIterator {
                iterated: iterator_record,
            },
        );

        Ok(wrapper.into())
    }

    /// `Iterator.concat ( ...items )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.concat
    fn concat(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let iterables be a new empty List.
        let mut iterables = VecDeque::with_capacity(args.len());

        // 2. For each element item of items, do
        for item in args {
            // a. If item is not an Object, throw a TypeError exception.
            let Some(item) = item.as_object() else {
                return Err(js_error!(TypeError: "Iterator.concat requires iterable objects"));
            };

            // b. Let method be ? GetMethod(item, %Symbol.iterator%).
            // c. If method is undefined, throw a TypeError exception.
            let method = item.get_method(JsSymbol::iterator(), context)?.ok_or_else(
                || js_error!(TypeError: "Iterator.concat requires objects with @@iterator"),
            )?;

            // d. Append the Record { [[OpenMethod]]: method, [[Iterable]]: item } to iterables.
            iterables.push_back(IterableRecord {
                iterable: item,
                open_method: JsFunction::from_object_unchecked(method),
            });
        }

        // 3. Let closure be a new Abstract Closure with no parameters that captures iterables
        //    and performs the following steps when called:
        //    (implemented via IteratorHelperOp::Concat in execute_next)
        // 4-5. Let result be CreateIteratorFromClosure(closure, "Iterator Helper", ...)
        //      with [[UnderlyingIterators]] set to a new empty List.
        let helper = IteratorHelper::create(iterator_helper::Concat::new(iterables), context);

        // 6. Return result.
        Ok(helper.into())
    }

    // ==================== Static Methods — Experimental ====================

    #[cfg(feature = "experimental")]
    /// `Iterator.zip ( iterables [ , options ] )`
    ///
    /// More information:
    ///  - [TC39 proposal][spec]
    ///
    /// [spec]: https://tc39.es/proposal-joint-iteration/#sec-iterator.zip
    fn zip(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterables = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. If iterables is not an Object, throw a TypeError exception.
        let iterables_obj = iterables.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.zip requires an iterable object")
        })?;

        // 2. Set options to ? GetOptionsObject(options).
        // 3. Let mode be ? Get(options, "mode").
        // 4. If mode is undefined, set mode to "shortest".
        // 5. If mode is not one of "shortest", "longest", or "strict", throw a TypeError exception.
        let mode = Self::parse_zip_mode(options, context)?;

        // 6. Let paddingOption be undefined.
        // 7. If mode is "longest", then
        //     a. Set paddingOption to ? Get(options, "padding").
        //     b. If paddingOption is not undefined and paddingOption is not an Object, throw a TypeError exception.
        let padding_option = if mode == ZipMode::Longest {
            let p = options
                .as_object()
                .map(|opts| opts.get(js_string!("padding"), context))
                .transpose()?
                .unwrap_or_default();

            if p.is_undefined() {
                None
            } else if p.is_object() {
                Some(p)
            } else {
                return Err(JsNativeError::typ()
                    .with_message("padding must be an object")
                    .into());
            }
        } else {
            None
        };

        // 8. Let iters be a new empty List.
        let mut iters: Vec<super::IteratorRecord> = Vec::new();

        // 9. Let padding be a new empty List.
        // (padding list built later in build_padding)

        // 10. Let inputIter be ? GetIterator(iterables, sync).
        let iterables_val: JsValue = iterables_obj.clone().into();
        let mut input_iter = iterables_val.get_iterator(IteratorHint::Sync, context)?;

        // 11. Let next be not-started.
        // 12. Repeat, while next is not done,
        //     a. Set next to Completion(IteratorStepValue(inputIter)).
        //     b. IfAbruptCloseIterators(next, iters).
        //     c. If next is not done, then
        //         i. Let iter be Completion(GetIteratorFlattenable(next, reject-primitives)).
        //         ii. IfAbruptCloseIterators(iter, the list-concatenation of « inputIter » and iters).
        //         iii. Append iter to iters.
        loop {
            let next = input_iter.step_value(context);
            match next {
                Err(err) => {
                    // IfAbruptCloseIterators(next, iters)
                    for iter in &iters {
                        drop(iter.close(Ok(JsValue::undefined()), context));
                    }
                    return Err(err);
                }
                Ok(None) => break, // done
                Ok(Some(value)) => {
                    // GetIteratorFlattenable(next, reject-primitives)
                    if !value.is_object() {
                        // Close all collected iterators and the input iterator.
                        for iter in &iters {
                            drop(iter.close(Ok(JsValue::undefined()), context));
                        }
                        drop(input_iter.close(Ok(JsValue::undefined()), context));
                        return Err(JsNativeError::typ()
                            .with_message("iterator value is not an object")
                            .into());
                    }
                    let iter_result = value.get_iterator(IteratorHint::Sync, context);
                    match iter_result {
                        Err(err) => {
                            for iter in &iters {
                                drop(iter.close(Ok(JsValue::undefined()), context));
                            }
                            drop(input_iter.close(Ok(JsValue::undefined()), context));
                            return Err(err);
                        }
                        Ok(iter) => iters.push(iter),
                    }
                }
            }
        }

        // 13. Let iterCount be the number of elements in iters.
        let iter_count = iters.len();

        // 14. If mode is "longest", then ... Build padding list.
        let padding = Self::build_padding(padding_option, iter_count, &iters, context)?;

        // 15. Let finishResults be a new Abstract Closure ... (handled in ZipIterator::create_zip_iterator)
        // 16. Return ? IteratorZip(iters, mode, padding, finishResults).
        Ok(ZipIterator::create_zip_iterator(
            iters,
            mode,
            padding,
            ZipResultKind::Array,
            context,
        ))
    }

    #[cfg(feature = "experimental")]
    /// `Iterator.zipKeyed ( iterables [ , options ] )`
    ///
    /// More information:
    ///  - [TC39 proposal][spec]
    ///
    /// [spec]: https://tc39.es/proposal-joint-iteration/#sec-iterator.zipkeyed
    fn zip_keyed(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterables = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. If iterables is not an Object, throw a TypeError exception.
        let iterables_obj = iterables.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.zipKeyed requires an object")
        })?;

        // 2. Set options to ? GetOptionsObject(options).
        // 3. Let mode be ? Get(options, "mode").
        // 4. If mode is undefined, set mode to "shortest".
        // 5. If mode is not one of "shortest", "longest", or "strict", throw a TypeError exception.
        let mode = Self::parse_zip_mode(options, context)?;

        // 6. Let paddingOption be undefined.
        // 7. If mode is "longest", then
        //     a. Set paddingOption to ? Get(options, "padding").
        //     b. If paddingOption is not undefined and paddingOption is not an Object, throw a TypeError exception.
        let padding_option = if mode == ZipMode::Longest {
            let p = options
                .as_object()
                .map(|opts| opts.get(js_string!("padding"), context))
                .transpose()?
                .unwrap_or_default();

            if p.is_undefined() {
                None
            } else if p.is_object() {
                Some(p)
            } else {
                return Err(JsNativeError::typ()
                    .with_message("padding must be an object")
                    .into());
            }
        } else {
            None
        };

        // 8. Let iters be a new empty List.
        let mut iters: Vec<super::IteratorRecord> = Vec::new();
        // 9. Let keys be a new empty List.
        let mut keys: Vec<JsValue> = Vec::new();

        // 10. Let iterablesKeys be ? EnumerableOwnProperties(iterables, key).
        let all_keys = iterables_obj.own_property_keys(context)?;
        // 11. For each element key of iterablesKeys, do
        //     a. Let value be ? Get(iterables, key).
        //     b. If value is not undefined, then
        //         i. Append key to keys.
        //         ii. Let iter be Completion(GetIteratorFlattenable(value, reject-primitives)).
        //         iii. IfAbruptCloseIterators(iter, iters).
        //         iv. Append iter to iters.
        for key in all_keys {
            let key_val: JsValue = key.clone().into();
            let value = iterables_obj.get(key.clone(), context)?;
            if !value.is_undefined() {
                keys.push(key_val);
                if !value.is_object() {
                    for iter in &iters {
                        drop(iter.close(Ok(JsValue::undefined()), context));
                    }
                    return Err(JsNativeError::typ()
                        .with_message("iterator value is not an object")
                        .into());
                }
                let iter = value.get_iterator(IteratorHint::Sync, context);
                match iter {
                    Err(err) => {
                        for it in &iters {
                            drop(it.close(Ok(JsValue::undefined()), context));
                        }
                        return Err(err);
                    }
                    Ok(iter) => iters.push(iter),
                }
            }
        }

        // 12. Let iterCount be the number of elements in iters.
        let iter_count = iters.len();

        // 13. Let padding be a new empty List.
        // 14. If mode is "longest", then ... (Build padding for zipKeyed)
        let padding = if mode == ZipMode::Longest {
            match padding_option {
                None => vec![JsValue::undefined(); iter_count],
                Some(pad_obj) => {
                    let pad = pad_obj
                        .as_object()
                        .expect("padding object verification already executed above");
                    let mut padding = Vec::with_capacity(iter_count);
                    for key in &keys {
                        let prop_key = key.to_string(context).unwrap_or_default();
                        let val = pad.get(prop_key, context)?;
                        padding.push(val);
                    }
                    padding
                }
            }
        } else {
            Vec::new()
        };

        // 15. Let finishResults be a new Abstract Closure ... (handled in ZipIterator::create_zip_iterator)
        // 16. Return ? IteratorZip(iters, mode, padding, finishResults).
        Ok(ZipIterator::create_zip_iterator(
            iters,
            mode,
            padding,
            ZipResultKind::Keyed(keys),
            context,
        ))
    }

    #[cfg(feature = "experimental")]
    /// Parses the `mode` option from the options object.
    fn parse_zip_mode(options: &JsValue, context: &mut Context) -> JsResult<ZipMode> {
        if options.is_undefined() || options.is_null() {
            return Ok(ZipMode::Shortest);
        }
        let opts = options
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("options must be an object"))?;
        let mode_val = opts.get(js_string!("mode"), context)?;
        if mode_val.is_undefined() {
            return Ok(ZipMode::Shortest);
        }
        let mode_str = mode_val.to_string(context)?;
        match mode_str.to_std_string_escaped().as_str() {
            "shortest" => Ok(ZipMode::Shortest),
            "longest" => Ok(ZipMode::Longest),
            "strict" => Ok(ZipMode::Strict),
            _ => Err(JsNativeError::typ()
                .with_message("mode must be \"shortest\", \"longest\", or \"strict\"")
                .into()),
        }
    }

    #[cfg(feature = "experimental")]
    /// Builds the padding list for "longest" mode.
    fn build_padding(
        padding_option: Option<JsValue>,
        iter_count: usize,
        iters: &[super::IteratorRecord],
        context: &mut Context,
    ) -> JsResult<Vec<JsValue>> {
        match padding_option {
            None => Ok(vec![JsValue::undefined(); iter_count]),
            Some(pad_val) => {
                let mut padding_iter = pad_val
                    .get_iterator(IteratorHint::Sync, context)
                    .inspect_err(|_err| {
                        for iter in iters {
                            drop(iter.close(Ok(JsValue::undefined()), context));
                        }
                    })?;
                let mut padding = Vec::new();
                let mut using_iterator = true;

                for _ in 0..iter_count {
                    if using_iterator {
                        match padding_iter.step_value(context) {
                            Err(err) => {
                                for iter in iters {
                                    drop(iter.close(Ok(JsValue::undefined()), context));
                                }
                                return Err(err);
                            }
                            Ok(None) => {
                                using_iterator = false;
                                padding.push(JsValue::undefined());
                            }
                            Ok(Some(val)) => {
                                padding.push(val);
                            }
                        }
                    } else {
                        padding.push(JsValue::undefined());
                    }
                }

                if using_iterator {
                    let close_result = padding_iter.close(Ok(JsValue::undefined()), context);
                    if let Err(err) = close_result {
                        for iter in iters {
                            drop(iter.close(Ok(JsValue::undefined()), context));
                        }
                        return Err(err);
                    }
                }

                Ok(padding)
            }
        }
    }
}
