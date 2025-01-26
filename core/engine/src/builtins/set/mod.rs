//! Boa's implementation of ECMAScript's global `Set` object.
//!
//! The ECMAScript `Set` class is a global object that is used in the construction of sets; which
//! are high-level, collections of values.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-set-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set

mod set_iterator;

#[cfg(test)]
mod tests;

pub mod ordered_set;



use self::ordered_set::OrderedSet;
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_profiler::Profiler;
use num_traits::Zero;
pub(crate) use set_iterator::SetIterator;
use super::iterable::IteratorHint;

#[derive(Debug, Clone)]
pub(crate) struct Set;

impl IntrinsicObject for Set {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_species = BuiltInBuilder::callable(realm, Self::get_species)
            .name(js_string!("get [Symbol.species]"))
            .build();

        let size_getter = BuiltInBuilder::callable(realm, Self::size_getter)
            .name(js_string!("get size"))
            .build();

        let values_function = BuiltInBuilder::callable(realm, Self::values)
            .name(js_string!("values"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_accessor(
                JsSymbol::species(),
                Some(get_species),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::add, js_string!("add"), 1)
            .method(Self::clear, js_string!("clear"), 0)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::entries, js_string!("entries"), 0)
            .method(Self::for_each, js_string!("forEach"), 1)
            .method(Self::has, js_string!("has"), 1)
            .method(Self::difference, js_string!("difference"), 1)
            .method(Self::intersection, js_string!("intersection"), 1)
            .method(Self::is_dis_joint_from, js_string!("isDisjointFrom"), 0)
            .method(Self::is_subset_of, js_string!("isSubsetOf"), 0)
            .method(Self::is_superset_of, js_string!("isSupersetOf"), 0)
            .property(
                js_string!("keys"),
                values_function.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("size"),
                Some(size_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("values"),
                values_function.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::iterator(),
                values_function,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }
}

impl BuiltInObject for Set {
    const NAME: JsString = StaticJsStrings::SET;
}

impl BuiltInConstructor for Set {
    const LENGTH: usize = 0;
    const P: usize = 11;
    const SP: usize = 1;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::set;

    /// [`Set ( [ iterable ] )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-iterable
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling a builtin Set constructor without new is forbidden")
                .into());
        }

        // 2. Let set be ? OrdinaryCreateFromConstructor(NewTarget, "%Set.prototype%", « [[SetData]] »).
        // 3. Set set.[[SetData]] to a new empty List.
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::set, context)?;
        let set = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrderedSet::default(),
        );

        // 4. If iterable is either undefined or null, return set.
        let iterable = args.get_or_undefined(0);
        if iterable.is_null_or_undefined() {
            return Ok(set.into());
        }

        // 5. Let adder be ? Get(set, "add").
        let adder = set.get(js_string!("add"), context)?;

        // 6. If IsCallable(adder) is false, throw a TypeError exception.
        let adder = adder.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("'add' of 'newTarget' is not a function")
        })?;

        // 7. Let iteratorRecord be ? GetIterator(iterable, sync).
        let mut iterator_record = iterable.clone().get_iterator(IteratorHint::Sync, context)?;

        // 8. Repeat,
        //     a. Let next be ? IteratorStepValue(iteratorRecord).
        while let Some(next) = iterator_record.step_value(context)? {
            // c. Let status be Completion(Call(adder, set, « next »)).
            if let Err(status) = adder.call(&set.clone().into(), &[next], context) {
                // d. IfAbruptCloseIterator(status, iteratorRecord).
                return iterator_record.close(Err(status), context);
            }
        }

        //     b. If next is done, return set.
        Ok(set.into())
    }
}

impl Set {
    /// Utility for constructing `Set` objects.
    pub(crate) fn set_create(prototype: Option<JsObject>, context: &mut Context) -> JsObject {
        let prototype =
            prototype.unwrap_or_else(|| context.intrinsics().constructors().set().prototype());

        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrderedSet::new(),
        )
    }

    /// Utility for constructing `Set` objects from an iterator of `JsValue`'s.
    pub(crate) fn create_set_from_list<I>(elements: I, context: &mut Context) -> JsObject
    where
        I: IntoIterator<Item = JsValue>,
    {
        // Create empty Set
        let set = Self::set_create(None, context);
        // For each element e of elements, do
        for elem in elements {
            Self::add(&set.clone().into(), &[elem], context)
                .expect("adding new element shouldn't error out");
        }

        set
    }

    /// `get Set [ @@species ]`
    ///
    /// The Set[Symbol.species] accessor property returns the Set constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-set-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `Set.prototype.add( value )`
    ///
    /// This method adds an entry with value into the set. Returns the set object
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.add
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/add
    pub(crate) fn add(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(mut set) = this
            .as_object()
            .and_then(JsObject::downcast_mut::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.add called on incompatible receiver")
                .into());
        };

        // 3. For each element e of S.[[SetData]], do
        // a. If e is not empty and SameValueZero(e, value) is true, then
        // i. Return S.
        // 4. If value is -0𝔽, set value to +0𝔽.
        let value = args.get_or_undefined(0);
        let value = match value.as_number() {
            Some(n) if n.is_zero() => &JsValue::new(0),
            _ => value,
        };

        // 5. Append value to S.[[SetData]].
        set.add(value.clone());

        Ok(this.clone())
        // 6. Return S.
    }

    /// `Set.prototype.clear( )`
    ///
    /// This method removes all entries from the set.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.clear
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/clear
    pub(crate) fn clear(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let Some(mut set) = this
            .as_object()
            .and_then(JsObject::downcast_mut::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("'this' is not a Set")
                .into());
        };

        set.clear();

        Ok(JsValue::undefined())
    }

    /// `Set.prototype.delete( value )`
    ///
    /// This method removes the entry for the given value if it exists.
    /// Returns true if there was an element, false otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.delete
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/delete
    pub(crate) fn delete(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(mut set) = this
            .as_object()
            .and_then(JsObject::downcast_mut::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.delete called on incompatible receiver")
                .into());
        };

        let value = args.get_or_undefined(0);
        let value = match value.as_number() {
            Some(n) if n.is_zero() => &JsValue::new(0),
            _ => value,
        };

        // 3. For each element e of S.[[SetData]], do
        // a. If e is not empty and SameValueZero(e, value) is true, then
        // i. Replace the element of S.[[SetData]] whose value is e with an element whose value is empty.
        // ii. Return true.
        // 4. Return false.
        Ok(set.delete(value).into())
    }

    /// `Set.prototype.entries( )`
    ///
    /// This method returns an iterator over the entries of the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.entries
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/entries
    pub(crate) fn entries(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let Some(lock) = this.as_object().and_then(|o| {
            o.downcast_mut::<OrderedSet>()
                .map(|mut set| set.lock(o.clone()))
        }) else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.entries called on incompatible receiver")
                .into());
        };

        Ok(SetIterator::create_set_iterator(
            this.clone(),
            PropertyNameKind::KeyAndValue,
            lock,
            context,
        ))
    }

    /// `Set.prototype.forEach( callbackFn [ , thisArg ] )`
    ///
    /// This method executes the provided callback function for each value in the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.foreach
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/foreach
    pub(crate) fn for_each(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(lock) = this.as_object().and_then(|o| {
            o.downcast_mut::<OrderedSet>()
                .map(|mut set| set.lock(o.clone()))
        }) else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.forEach called on incompatible receiver")
                .into());
        };

        // 3. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let Some(callback_fn) = args.get_or_undefined(0).as_callable() else {
            return Err(JsNativeError::typ()
                .with_message(
                    "Method Set.prototype.forEach called with non-callable callback function",
                )
                .into());
        };

        // 4. Let entries be S.[[SetData]].
        // 5. Let numEntries be the number of elements in entries.
        // 6. Let index be 0.
        let mut index = 0;

        // 7. Repeat, while index < numEntries,
        while index < Self::get_size_full(this)? {
            // a. Let e be entries[index].
            let Some(set) = this
                .as_object()
                .and_then(JsObject::downcast_ref::<OrderedSet>)
            else {
                return Err(JsNativeError::typ()
                    .with_message("Method Set.prototype.forEach called on incompatible receiver")
                    .into());
            };

            let e = set.get_index(index).cloned();
            drop(set);

            // b. Set index to index + 1.
            index += 1;

            // c. If e is not empty, then
            if let Some(e) = e {
                // i. Perform ? Call(callbackfn, thisArg, « e, e, S »).
                // ii. NOTE: The number of elements in entries may have increased during execution of callbackfn.
                // iii. Set numEntries to the number of elements in entries.
                callback_fn.call(
                    args.get_or_undefined(1),
                    &[e.clone(), e.clone(), this.clone()],
                    context,
                )?;
            }
        }

        drop(lock);

        // 8. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `Map.prototype.has( key )`
    ///
    /// This method checks if the map contains an entry with the given key.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.has
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/has
    pub(crate) fn has(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.has called on incompatible receiver")
                .into());
        };

        let value = args.get_or_undefined(0);
        let value = match value.as_number() {
            Some(n) if n.is_zero() => &JsValue::new(0),
            _ => value,
        };

        // 3. For each element e of S.[[SetData]], do
        // a. If e is not empty and SameValueZero(e, value) is true, return true.
        // 4. Return false.
        Ok(set.contains(value).into())
    }

    /// `Set.prototype.values( )`
    ///
    /// This method returns an iterator over the values of the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/values
    pub(crate) fn values(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let Some(lock) = this.as_object().and_then(|o| {
            o.downcast_mut::<OrderedSet>()
                .map(|mut set| set.lock(o.clone()))
        }) else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.values called on incompatible receiver")
                .into());
        };

        Ok(SetIterator::create_set_iterator(
            this.clone(),
            PropertyNameKind::Value,
            lock,
            context,
        ))
    }

    /// ` Set.prototype.difference ( other ) `
    /// 
    /// This method returns a new Set containing all elements that are in the current Set
    /// but not in the given iterable `other`.
    /// 
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    /// 
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.difference
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/difference

    pub(crate) fn difference(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        //    (ECMAScript 2022, 24.2.3.6 steps 1–2)
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.difference called on incompatible receiver")
                .into());
        };

        // 3. Let otherRec be ? GetSetRecord(other).
        //    (ECMAScript 2022, 24.2.3.6 step 3)
        let other = args.get_or_undefined(0);
        let Some(other_set) = other
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.difference called on incompatible receiver")
                .into());
        };

        // 4. Let resultSetData be a copy of O.[[SetData]].
        //    (ECMAScript 2022, 24.2.3.6 step 4)
        let mut result_set = set.clone();

        // 5. If SetDataSize(O.[[SetData]]) ≤ otherRec.[[Size]], then:
        //    (ECMAScript 2022, 24.2.3.6 step 5)
        if Self::get_size_full(this)? <= other_set.len() {
            // Iterate over elements of the current set.
            let elements: Vec<_> = result_set.iter().cloned().collect();
            for element in elements {
                // Remove elements from resultSetData that are in otherRec.
                if other_set.contains(&element) {
                    result_set.delete(&element);
                }
            }
        } else {
            // Otherwise, iterate over elements of the other set.
            let other_elements: Vec<_> = other_set.iter().cloned().collect();
            for element in other_elements {
                // Remove elements from resultSetData that are in otherRec.
                result_set.delete(&element);
            }
        }

        // 6. Return a new set with the updated resultSetData.
        //    (ECMAScript 2022, 24.2.3.6 step 6)
        Ok(Self::create_set_from_list(result_set.iter().cloned(), context).into())
    }

    /// `Set.prototype.intersection ( other )`
    ///
    /// This method returns a new Set containing all elements that are present in both
    /// the current Set and the given iterable `other`.
    ///
    /// It effectively computes the intersection of the two Sets.
    ///
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.intersection
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/intersection
    pub(crate) fn intersection(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // Here, the variable `S` holds the value of `this`, which represents the current set over which the operation is being performed.
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message(
                    "Method Set.prototype.difference called on incompatible receiver"
                )
                .into());
        };

        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        // This step checks if the object calling the method has an internal data structure `[[SetData]]`.
        // This is important to ensure that the object is a valid set that can be operated on.
        // The error handling for this case is already done in the previous step when trying to access the internal data.

        // 3. Let other be the first argument.
        // We retrieve the first argument passed to the `intersection` method. This is the second set with which we want to find the intersection.
        let other = args.get_or_undefined(0);

        // 4. Let other_set be the second argument.
        // We try to downcast the second argument into an `OrderedSet`, which is the other set with which we perform the intersection.
        let Some(other_set) = other
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message(
                    "Method Set.prototype.difference called on incompatible receiver"
                )
                .into());
        };

        // 5. If S or other is empty, return an empty Set.
        // If either of the sets is empty, the intersection will also be empty.
        // In this case, we immediately return an empty set.
        if set.is_empty() || other_set.is_empty() {
            return Ok(Self::create_set_from_list(set.iter().cloned(), context).into());
        }

        // 6. Create an empty result set.
        // We create an empty set that will hold the common elements of the two sets.
        let mut result_set = OrderedSet::new();

        // 7. Let iter_set and check_set be the set with fewer elements.
        // For optimization, we choose to iterate over the smaller set. This reduces the number of operations when one set is much smaller than the other.
        let (
            iter_set, check_set
        ) = if set.len() <= other_set.len() {
            (other_set.iter(), set)
        } else {
            (set.iter(), other_set)
        };

        // 8. Iterate through iter_set and add elements to result_set if they are contained in check_set.
        // We loop through the smaller set and add the elements that are found in the larger set to the result set.
        for value in iter_set {
            if check_set.contains(value) {
                result_set.add(value.clone());
            }
        }

        // 9. Return the result set.
        // After the iteration, we return the result set containing the intersected elements.
        Ok(Set::create_set_from_list(result_set.iter().cloned(), context).into())
    }




    /// `Set.prototype.isDisjointFrom ( other )`
    ///
    /// This method checks whether the current Set and the given iterable `other` have no elements in common.
    /// It returns `true` if the two Sets are disjoint (i.e., they have no overlapping elements),
    /// and `false` otherwise.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.isdisjointfrom
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/isDisjointFrom
    pub(crate) fn is_dis_joint_from (
        this: &JsValue,
        args: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isDisjointFrom called on incompatible receiver")
                .into());
        };

        // 3. Let otherRec be ? GetSetRecord(other).
        let other = args.get_or_undefined(0);
        let Some(other_set) = other
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isDisjointFrom called on incompatible receiver")
                .into());
        };

        // 4. Iterate over the smaller set to check for common elements.
        if Self::get_size_full(this)? <= other_set.len(){
            for value in set.iter() {
                if other_set.contains(value) {
                    return Ok(JsValue::from(false));
                }
            }
        } else {
            for value in other_set.iter() {
                if set.contains(value) {
                    return Ok(JsValue::from(false));
                }
            }
        }

        // 5. If no common elements are found, return true.
        Ok(JsValue::from(true))
    }

    /// `Set.prototype.isSubsetOf ( other )`
    ///
    /// This method checks whether the current Set is a subset of the given iterable `other`.
    /// It returns `true` if all elements of the current Set are present in the given iterable,
    /// and `false` otherwise.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.issubsetof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/isSubsetOf
    pub(crate) fn is_subset_of(
        this: &JsValue,
        args: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isSubsetOf called on incompatible receiver")
                .into());
        };

        // 2. Perform ? RequireInternalSlot(O, [[SetData]]).
        // 3. Let otherRec be ? GetSetRecord(other).
        let other = args.get_or_undefined(0);
        let Some(other_set) = other
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isSubsetOf called on incompatible argument")
                .into());
        };

        // 4. If SetDataSize(O.[[SetData]]) > otherRec.[[Size]], return false.
        if set.len() > other_set.len() {
            return Ok(JsValue::from(false));
        }

        // 5. Let thisSize be the number of elements in O.[[SetData]].

        // 6. Let index be 0.
        for value in set.iter() {
            // 7. If e is not empty, then
            // 8. Call the Has method of other_set to check if `value` is contained in it.
            if !other_set.contains(value) {
                return Ok(JsValue::from(false));
            }
        }

        // 9. Return true if all elements of `this` are in `other`.
        Ok(JsValue::from(true))
    }

    /// `Set.prototype.isSupersetOf ( other )`
    ///
    /// This method checks whether the current Set is a superset of the given iterable `other`.
    /// It returns `true` if the current Set contains all elements from the given iterable,
    /// and `false` otherwise.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.issupersetof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/isSupersetOf
    pub(crate) fn is_superset_of(
        this: &JsValue,
        args: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isSupersetOf called on incompatible receiver")
                .into());
        };

        // 2. Perform ? RequireInternalSlot(O, [[SetData]]).
        let other = args.get_or_undefined(0);
        let Some(other_set) = other
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isSupersetOf called on incompatible argument")
                .into());
        };

        // 3. If SetDataSize(O.[[SetData]]) < otherRec.[[Size]], return false.
        if Self::get_size_full(this)? <= other_set.len() {
            return Ok(JsValue::from(false));
        }

        // 4. Let thisSize be the number of elements in O.[[SetData]].
        for value in other_set.iter() {
            if !set.contains(value) {
                return Ok(JsValue::from(false));
            }
        }

        // 5. Return true if all elements of `other` are in `this`.
        Ok(JsValue::from(true))
    }

    fn size_getter(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_size(this).map(JsValue::from)
    }

    /// Helper function to get the size of the `Set` object.
    pub(crate) fn get_size(set: &JsValue) -> JsResult<usize> {
        set.as_object()
            .and_then(|obj| {
                obj.borrow()
                    .downcast_ref::<OrderedSet>()
                    .map(OrderedSet::len)
            })
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Set")
                    .into()
            })
    }

    /// Helper function to get the full size of the `Set` object.
    pub(crate) fn get_size_full(set: &JsValue) -> JsResult<usize> {
        set.as_object()
            .and_then(|obj| {
                obj.borrow()
                    .downcast_ref::<OrderedSet>()
                    .map(OrderedSet::full_len)
            })
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Set")
                    .into()
            })
    }
}
