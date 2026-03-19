//! Boa's implementation of the ECMAScript `Iterator` constructor.
//!
//! The `Iterator` constructor is designed to be subclassed. It may be used as the
//! value of an extends clause of a class definition.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-iterator-constructor

use crate::{
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject, object::OrdinaryObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

use super::{
    if_abrupt_close_iterator,
    iterator_helper::{IteratorHelper, IteratorHelperOp},
    wrap_for_valid_iterator::WrapForValidIterator,
};

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
        let get_constructor = BuiltInBuilder::callable(realm, Self::get_constructor)
            .name(js_string!("get constructor"))
            .build();
        let set_constructor = BuiltInBuilder::callable(realm, Self::set_constructor)
            .name(js_string!("set constructor"))
            .build();
        let get_to_string_tag = BuiltInBuilder::callable(realm, Self::get_to_string_tag)
            .name(js_string!("get [Symbol.toStringTag]"))
            .build();
        let set_to_string_tag = BuiltInBuilder::callable(realm, Self::set_to_string_tag)
            .name(js_string!("set [Symbol.toStringTag]"))
            .build();

        // Per the spec, `Iterator.prototype.constructor` must be a configurable,
        // non-enumerable get/set accessor (web-compat requirement).  We use the
        // builder's `constructor_accessor` support so the property is part of the
        // shared-shape allocation rather than a post-build override.
        let builder = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            ))
            // Static methods
            .static_method(Self::from, js_string!("from"), 1)
            .static_method(Self::concat, js_string!("concat"), 0);

        #[cfg(feature = "experimental")]
        let builder = builder
            .static_method(Self::zip, js_string!("zip"), 1)
            .static_method(Self::zip_keyed, js_string!("zipKeyed"), 1);

        builder
            // Prototype methods — lazy (return IteratorHelper)
            .method(Self::map, js_string!("map"), 1)
            .method(Self::filter, js_string!("filter"), 1)
            .method(Self::take, js_string!("take"), 1)
            .method(Self::drop, js_string!("drop"), 1)
            .method(Self::flat_map, js_string!("flatMap"), 1)
            // Prototype methods — eager (consume the iterator)
            .method(Self::reduce, js_string!("reduce"), 1)
            .method(Self::to_array, js_string!("toArray"), 0)
            .method(Self::for_each, js_string!("forEach"), 1)
            .method(Self::some, js_string!("some"), 1)
            .method(Self::every, js_string!("every"), 1)
            .method(Self::find, js_string!("find"), 1)
            // Accessor: Iterator.prototype[@@toStringTag]
            .accessor(
                JsSymbol::to_string_tag(),
                Some(get_to_string_tag),
                Some(set_to_string_tag),
                Attribute::CONFIGURABLE,
            )
            // Accessor: Iterator.prototype.constructor (web-compat, 2 slots)
            .constructor_accessor(get_constructor, set_constructor)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().iterator().constructor()
    }
}

impl BuiltInObject for IteratorConstructor {
    const NAME: JsString = StaticJsStrings::ITERATOR;
}

impl BuiltInConstructor for IteratorConstructor {
    const PROTOTYPE_STORAGE_SLOTS: usize = 14; // 11 methods + @@toStringTag accessor (2 slots) + constructor accessor (2 slots)
    #[cfg(not(feature = "experimental"))]
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 2;
    #[cfg(feature = "experimental")]
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 4;
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
        let prototype = crate::object::internal_methods::get_prototype_from_constructor(
            new_target,
            StandardConstructors::iterator,
            context,
        )?;

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
    // ==================== Static Methods ====================

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

    fn concat(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let iterables be a new empty List.
        let mut iterables = Vec::with_capacity(args.len());

        // 2. For each element item of items, do
        for item in args {
            // a. If item is not an Object, throw a TypeError exception.
            if !item.is_object() {
                return Err(JsNativeError::typ()
                    .with_message("Iterator.concat requires iterable objects")
                    .into());
            }

            // b. Let method be ? GetMethod(item, %Symbol.iterator%).
            // c. If method is undefined, throw a TypeError exception.
            let method = item
                .get_method(JsSymbol::iterator(), context)?
                .ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message("Iterator.concat requires objects with @@iterator")
                })?;

            // d. Append the Record { [[OpenMethod]]: method, [[Iterable]]: item } to iterables.
            iterables.push((method, item.clone()));
        }

        // 3. Let closure be a new Abstract Closure with no parameters that captures iterables
        //    and performs the following steps when called:
        //    (implemented via IteratorHelperOp::Concat in execute_next)
        // 4-5. Let result be CreateIteratorFromClosure(closure, "Iterator Helper", ...)
        //      with [[UnderlyingIterators]] set to a new empty List.
        let helper = IteratorHelper::create(
            vec![],
            IteratorHelperOp::Concat {
                iterables,
                current_index: 0,
                inner: None,
            },
            context,
        );

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
            if let Some(opts) = options.as_object() {
                let p = opts.get(js_string!("padding"), context)?;
                if p.is_undefined() {
                    None
                } else if !p.is_object() {
                    return Err(JsNativeError::typ()
                        .with_message("padding must be an object")
                        .into());
                } else {
                    Some(p)
                }
            } else {
                None
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
            if let Some(opts) = options.as_object() {
                let p = opts.get(js_string!("padding"), context)?;
                if p.is_undefined() {
                    None
                } else if !p.is_object() {
                    return Err(JsNativeError::typ()
                        .with_message("padding must be an object")
                        .into());
                } else {
                    Some(p)
                }
            } else {
                None
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

    // ==================== Prototype Accessor Properties ====================

    /// `get Iterator.prototype.constructor`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.constructor
    #[allow(clippy::unnecessary_wraps)]
    fn get_constructor(
        _this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(context
            .intrinsics()
            .constructors()
            .iterator()
            .constructor()
            .into())
    }

    /// `set Iterator.prototype.constructor`
    ///
    /// `SetterThatIgnoresPrototypeProperties(this, %Iterator.prototype%, "constructor", v)`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.constructor
    fn set_constructor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::setter_that_ignores_prototype_properties(
            this,
            &context.intrinsics().constructors().iterator().prototype(),
            js_string!("constructor"),
            args.get_or_undefined(0),
            context,
        )
    }

    /// `get Iterator.prototype[@@toStringTag]`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype-%40%40tostringtag
    #[allow(clippy::unnecessary_wraps)]
    fn get_to_string_tag(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(js_string!("Iterator").into())
    }

    /// `set Iterator.prototype[@@toStringTag]`
    ///
    /// `SetterThatIgnoresPrototypeProperties(this, %Iterator.prototype%, @@toStringTag, v)`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype-%40%40tostringtag
    fn set_to_string_tag(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::setter_that_ignores_prototype_properties(
            this,
            &context.intrinsics().constructors().iterator().prototype(),
            JsSymbol::to_string_tag(),
            args.get_or_undefined(0),
            context,
        )
    }

    /// `SetterThatIgnoresPrototypeProperties ( this, home, p, v )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-SetterThatIgnoresPrototypeProperties
    fn setter_that_ignores_prototype_properties<K: Into<crate::property::PropertyKey>>(
        this: &JsValue,
        home: &JsObject,
        p: K,
        v: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let p = p.into();

        // 1. If this is not an Object, then
        let Some(this_obj) = this.as_object() else {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("Cannot set property on a non-object")
                .into());
        };

        // 2. If this is home, then
        if JsObject::equals(&this_obj, home) {
            // a. NOTE: Throwing here emulates the behavior of a Set handler ...
            // b. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("Cannot set property directly on the prototype")
                .into());
        }

        // 3. Let desc be ? this.[[GetOwnProperty]](p).
        let desc = this_obj.__get_own_property__(&p, &mut context.into())?;

        // 4. If desc is undefined, then
        if desc.is_none() {
            // a. Perform ? CreateDataPropertyOrThrow(this, p, v).
            this_obj.create_data_property_or_throw(p, v.clone(), context)?;
        } else {
            // 5. Else,
            // a. Perform ? Set(this, p, v, true).
            this_obj.set(p, v.clone(), true, context)?;
        }

        // 6. Return undefined.
        Ok(JsValue::undefined())
    }

    // ==================== Prototype Methods — Lazy ====================

    /// `Iterator.prototype.map ( mapper )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.map
    fn map(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.map called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = super::get_iterator_direct(&o, context)?;

        // 4. If IsCallable(mapper) is false, then
        //    a. Let error be ThrowCompletion(a newly created TypeError object).
        //    b. Return ? IteratorClose(iterated, error).
        let mapper = args.get_or_undefined(0);
        let Some(mapper_obj) = mapper.as_callable() else {
            return iterated.close(
                Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.map: mapper is not callable")
                    .into()),
                context,
            );
        };

        // 5-17. Create IteratorHelper with map operation.
        let helper = IteratorHelper::create(
            vec![iterated],
            IteratorHelperOp::Map {
                mapper: mapper_obj.clone(),
                counter: 0,
            },
            context,
        );

        Ok(helper.into())
    }

    /// `Iterator.prototype.filter ( predicate )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.filter
    fn filter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.filter called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = super::get_iterator_direct(&o, context)?;

        // 4. If IsCallable(predicate) is false, then
        //    a. Let error be ThrowCompletion(a newly created TypeError object).
        //    b. Return ? IteratorClose(iterated, error).
        let predicate = args.get_or_undefined(0);
        let Some(predicate_obj) = predicate.as_callable() else {
            return iterated.close(
                Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.filter: predicate is not callable")
                    .into()),
                context,
            );
        };

        // 5-13. Create IteratorHelper with filter operation.
        let helper = IteratorHelper::create(
            vec![iterated],
            IteratorHelperOp::Filter {
                predicate: predicate_obj.clone(),
                counter: 0,
            },
            context,
        );

        Ok(helper.into())
    }

    /// `Iterator.prototype.take ( limit )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.take
    fn take(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.take called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = super::get_iterator_direct(&o, context)?;

        // 4. Let numLimit be ? ToNumber(limit).
        let limit = args.get_or_undefined(0);
        let num_limit = if_abrupt_close_iterator!(limit.to_number(context), iterated, context);

        // 5. If numLimit is NaN, throw a RangeError exception.
        if num_limit.is_nan() {
            return iterated.close(
                Err(JsNativeError::range()
                    .with_message("Iterator.prototype.take: limit is NaN")
                    .into()),
                context,
            );
        }

        // 6. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
        let integer_limit =
            if_abrupt_close_iterator!(limit.to_integer_or_infinity(context), iterated, context);

        // 7. If integerLimit < 0, throw a RangeError exception.
        let integer_limit = match integer_limit {
            crate::value::IntegerOrInfinity::Integer(n) if n < 0 => {
                return iterated.close(
                    Err(JsNativeError::range()
                        .with_message("Iterator.prototype.take: limit is negative")
                        .into()),
                    context,
                );
            }
            crate::value::IntegerOrInfinity::Integer(n) => n as u64,
            crate::value::IntegerOrInfinity::PositiveInfinity => u64::MAX,
            crate::value::IntegerOrInfinity::NegativeInfinity => {
                return iterated.close(
                    Err(JsNativeError::range()
                        .with_message("Iterator.prototype.take: limit is negative infinity")
                        .into()),
                    context,
                );
            }
        };

        // 8-10. Return CreateIteratorHelper with a take closure.
        let helper = IteratorHelper::create(
            vec![iterated],
            IteratorHelperOp::Take {
                remaining: integer_limit,
            },
            context,
        );

        Ok(helper.into())
    }

    /// `Iterator.prototype.drop ( limit )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.drop
    fn drop(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.drop called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = super::get_iterator_direct(&o, context)?;

        // 4. Let numLimit be ? ToNumber(limit).
        let limit = args.get_or_undefined(0);
        let num_limit = if_abrupt_close_iterator!(limit.to_number(context), iterated, context);

        // 5. If numLimit is NaN, throw a RangeError exception.
        if num_limit.is_nan() {
            return iterated.close(
                Err(JsNativeError::range()
                    .with_message("Iterator.prototype.drop: limit is NaN")
                    .into()),
                context,
            );
        }

        // 6. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
        let integer_limit =
            if_abrupt_close_iterator!(limit.to_integer_or_infinity(context), iterated, context);

        // 7. If integerLimit < 0, throw a RangeError exception.
        let integer_limit = match integer_limit {
            crate::value::IntegerOrInfinity::Integer(n) if n < 0 => {
                return iterated.close(
                    Err(JsNativeError::range()
                        .with_message("Iterator.prototype.drop: limit is negative")
                        .into()),
                    context,
                );
            }
            crate::value::IntegerOrInfinity::Integer(n) => n as u64,
            crate::value::IntegerOrInfinity::PositiveInfinity => u64::MAX,
            crate::value::IntegerOrInfinity::NegativeInfinity => {
                return iterated.close(
                    Err(JsNativeError::range()
                        .with_message("Iterator.prototype.drop: limit is negative infinity")
                        .into()),
                    context,
                );
            }
        };

        // 8-10. Return CreateIteratorHelper with a drop closure.
        let helper = IteratorHelper::create(
            vec![iterated],
            IteratorHelperOp::Drop {
                remaining: integer_limit,
                done_dropping: false,
            },
            context,
        );

        Ok(helper.into())
    }

    /// `Iterator.prototype.flatMap ( mapper )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.flatmap
    fn flat_map(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.flatMap called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = super::get_iterator_direct(&o, context)?;

        // 4. If IsCallable(mapper) is false, then
        //    a. Let error be ThrowCompletion(a newly created TypeError object).
        //    b. Return ? IteratorClose(iterated, error).
        let mapper = args.get_or_undefined(0);
        let Some(mapper_obj) = mapper.as_callable() else {
            return iterated.close(
                Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.flatMap: mapper is not callable")
                    .into()),
                context,
            );
        };

        // 5+. Create IteratorHelper with flatMap operation.
        let helper = IteratorHelper::create(
            vec![iterated],
            IteratorHelperOp::FlatMap {
                mapper: mapper_obj.clone(),
                counter: 0,
                inner_iterator: None,
            },
            context,
        );

        Ok(helper.into())
    }

    // ==================== Prototype Methods — Eager (Consuming) ====================

    /// `Iterator.prototype.reduce ( reducer [ , initialValue ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.reduce
    fn reduce(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.reduce called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = super::get_iterator_direct(&o, context)?;

        // 4. If IsCallable(reducer) is false, then
        //    a. Let error be ThrowCompletion(a newly created TypeError object).
        //    b. Return ? IteratorClose(iterated, error).
        let Some(reducer) = args.get_or_undefined(0).as_callable() else {
            return iterated.close(
                Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.reduce: reducer is not callable")
                    .into()),
                context,
            );
        };

        let mut accumulator;
        let mut counter;

        // If initialValue is not present
        if args.len() < 2 {
            // Let accumulator be ? IteratorStepValue(iterated).
            let first = iterated.step_value(context)?;
            match first {
                None => {
                    return Err(JsNativeError::typ()
                        .with_message(
                            "Iterator.prototype.reduce: reduce of empty iterator with no initial value",
                        )
                        .into());
                }
                Some(val) => {
                    accumulator = val;
                    counter = 1u64;
                }
            }
        } else {
            accumulator = args.get_or_undefined(1).clone();
            counter = 0;
        }

        // Repeat
        loop {
            let value = iterated.step_value(context)?;
            match value {
                None => return Ok(accumulator),
                Some(value) => {
                    let result = reducer.call(
                        &JsValue::undefined(),
                        &[accumulator, value, JsValue::new(counter)],
                        context,
                    );

                    match result {
                        Ok(val) => {
                            accumulator = val;
                        }
                        Err(err) => {
                            return iterated.close(Err(err), context);
                        }
                    }

                    counter += 1;
                }
            }
        }
    }

    /// `Iterator.prototype.toArray ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.toarray
    fn to_array(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.toArray called on non-object")
        })?;

        let iterated = super::get_iterator_direct(&o, context)?;

        // Let items be a new empty List.
        // Repeat ... append to items.
        let items = iterated.into_list(context)?;

        // Return CreateArrayFromList(items).
        Ok(crate::builtins::array::Array::create_array_from_list(items, context).into())
    }

    /// `Iterator.prototype.forEach ( fn )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.foreach
    fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.forEach called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = super::get_iterator_direct(&o, context)?;

        // 4. If IsCallable(fn) is false, then
        //    a. Let error be ThrowCompletion(a newly created TypeError object).
        //    b. Return ? IteratorClose(iterated, error).
        let Some(func) = args.get_or_undefined(0).as_callable() else {
            return iterated.close(
                Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.forEach: argument is not callable")
                    .into()),
                context,
            );
        };
        let mut counter = 0u64;

        loop {
            let value = iterated.step_value(context)?;
            match value {
                None => return Ok(JsValue::undefined()),
                Some(value) => {
                    let result = func.call(
                        &JsValue::undefined(),
                        &[value, JsValue::new(counter)],
                        context,
                    );

                    if let Err(err) = result {
                        return iterated.close(Err(err), context);
                    }

                    counter += 1;
                }
            }
        }
    }

    /// `Iterator.prototype.some ( predicate )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.some
    fn some(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.some called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = super::get_iterator_direct(&o, context)?;

        // 4. If IsCallable(predicate) is false, then
        //    a. Let error be ThrowCompletion(a newly created TypeError object).
        //    b. Return ? IteratorClose(iterated, error).
        let Some(predicate) = args.get_or_undefined(0).as_callable() else {
            return iterated.close(
                Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.some: predicate is not callable")
                    .into()),
                context,
            );
        };
        let mut counter = 0u64;

        loop {
            let value = iterated.step_value(context)?;
            match value {
                None => return Ok(JsValue::new(false)),
                Some(value) => {
                    let result = predicate.call(
                        &JsValue::undefined(),
                        &[value, JsValue::new(counter)],
                        context,
                    );

                    match result {
                        Ok(val) => {
                            if val.to_boolean() {
                                return iterated.close(Ok(JsValue::new(true)), context);
                            }
                        }
                        Err(err) => {
                            return iterated.close(Err(err), context);
                        }
                    }

                    counter += 1;
                }
            }
        }
    }

    /// `Iterator.prototype.every ( predicate )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.every
    fn every(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.every called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = super::get_iterator_direct(&o, context)?;

        // 4. If IsCallable(predicate) is false, then
        //    a. Let error be ThrowCompletion(a newly created TypeError object).
        //    b. Return ? IteratorClose(iterated, error).
        let Some(predicate) = args.get_or_undefined(0).as_callable() else {
            return iterated.close(
                Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.every: predicate is not callable")
                    .into()),
                context,
            );
        };
        let mut counter = 0u64;

        loop {
            let value = iterated.step_value(context)?;
            match value {
                None => return Ok(JsValue::new(true)),
                Some(value) => {
                    let result = predicate.call(
                        &JsValue::undefined(),
                        &[value, JsValue::new(counter)],
                        context,
                    );

                    match result {
                        Ok(val) => {
                            if !val.to_boolean() {
                                return iterated.close(Ok(JsValue::new(false)), context);
                            }
                        }
                        Err(err) => {
                            return iterated.close(Err(err), context);
                        }
                    }

                    counter += 1;
                }
            }
        }
    }

    /// `Iterator.prototype.find ( predicate )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.find
    fn find(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.find called on non-object")
        })?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let mut iterated = super::get_iterator_direct(&o, context)?;

        // 4. If IsCallable(predicate) is false, then
        //    a. Let error be ThrowCompletion(a newly created TypeError object).
        //    b. Return ? IteratorClose(iterated, error).
        let Some(predicate) = args.get_or_undefined(0).as_callable() else {
            return iterated.close(
                Err(JsNativeError::typ()
                    .with_message("Iterator.prototype.find: predicate is not callable")
                    .into()),
                context,
            );
        };
        let mut counter = 0u64;

        loop {
            let value = iterated.step_value(context)?;
            match value {
                None => return Ok(JsValue::undefined()),
                Some(value) => {
                    let result = predicate.call(
                        &JsValue::undefined(),
                        &[value.clone(), JsValue::new(counter)],
                        context,
                    );

                    match result {
                        Ok(val) => {
                            if val.to_boolean() {
                                return iterated.close(Ok(value), context);
                            }
                        }
                        Err(err) => {
                            return iterated.close(Err(err), context);
                        }
                    }

                    counter += 1;
                }
            }
        }
    }
}
