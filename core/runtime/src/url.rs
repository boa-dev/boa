//! Boa's implementation of JavaScript's `URL` Web API class.
//!
//! The `URL` class can be instantiated from any global object.
//! This relies on the `url` feature.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `URL` specification][spec]
//!
//! Implemented sections in this file:
//! - `URL.searchParams`: <https://url.spec.whatwg.org/#dom-url-searchparams>
//! - `URLSearchParams` constructor and methods: <https://url.spec.whatwg.org/#urlsearchparams>
//!
//! [spec]: https://url.spec.whatwg.org/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/URL
#![allow(clippy::needless_pass_by_value)]

#[cfg(test)]
mod tests;

use boa_engine::builtins::iterable::{IteratorRecord, create_iter_result_object};
use boa_engine::builtins::object::OrdinaryObject;
use boa_engine::class::Class;
use boa_engine::interop::{JsClass, TryFromJsArgument};
use boa_engine::object::{
    ObjectInitializer,
    builtins::{JsArray, TypedJsFunction},
};
use boa_engine::property::Attribute;
use boa_engine::realm::Realm;
use boa_engine::value::Convert;
use boa_engine::{
    Context, Finalize, JsData, JsError, JsObject, JsResult, JsString, JsSymbol, JsValue, Trace,
    boa_class, boa_module, js_error, js_string, native_function::NativeFunction,
};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

/// A callback function for the `URLSearchParams.prototype.forEach` method.
pub type SearchParamsForEachCallback = TypedJsFunction<(JsString, JsString, JsObject), ()>;

#[derive(Debug, Clone, Copy)]
enum UrlSearchParamsIteratorKind {
    Key,
    Value,
    KeyAndValue,
}

fn to_usv_string(string: &JsString) -> JsString {
    JsString::from(string.to_std_string_lossy())
}

fn to_usv_string_value(value: &JsValue, context: &mut Context) -> JsResult<JsString> {
    value
        .to_string(context)
        .map(|string| to_usv_string(&string))
}

fn parse_search_params(input: &JsString) -> Vec<(JsString, JsString)> {
    let input = input.to_std_string_lossy();
    let input = input.strip_prefix('?').unwrap_or(&input);

    url::form_urlencoded::parse(input.as_bytes())
        .map(|(name, value)| {
            (
                JsString::from(name.as_ref()),
                JsString::from(value.as_ref()),
            )
        })
        .collect()
}

fn serialize_search_params(params: &[(JsString, JsString)]) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());

    for (name, value) in params {
        let name = name.to_std_string_lossy();
        let value = value.to_std_string_lossy();
        serializer.append_pair(&name, &value);
    }

    serializer.finish()
}

/// Captures whether an optional argument was actually supplied.
///
/// This is used for overloads where an explicit `undefined` must not be treated
/// the same as an omitted argument.
#[derive(Debug, Clone)]
struct OptionalArg(Option<JsValue>);

impl<'a> TryFromJsArgument<'a> for OptionalArg {
    fn try_from_js_argument(
        _: &'a JsValue,
        rest: &'a [JsValue],
        _: &mut Context,
    ) -> JsResult<(Self, &'a [JsValue])> {
        match rest.split_first() {
            Some((first, rest)) => Ok((Self(Some(first.clone())), rest)),
            None => Ok((Self(None), rest)),
        }
    }
}

fn close_iterator_with_error<T>(
    iterator: &IteratorRecord,
    error: JsError,
    context: &mut Context,
) -> JsResult<T> {
    match iterator.close(Err(error), context) {
        Ok(_) => unreachable!("iterator close with error completion should not succeed"),
        Err(err) => Err(err),
    }
}

fn collect_sequence_item_pair(
    item: &JsObject,
    context: &mut Context,
) -> JsResult<(JsString, JsString)> {
    let Some(iterator_method) = item.get_method(JsSymbol::iterator(), context)? else {
        return Err(js_error!(
            TypeError: "URLSearchParams constructor expects each sequence item to be an iterable pair"
        ));
    };

    let item_value = JsValue::from(item.clone());
    let mut pair_iterator = item_value.get_iterator_from_method(&iterator_method, context)?;

    let Some(name) = pair_iterator.step_value(context)? else {
        return Err(js_error!(
            TypeError: "URLSearchParams constructor expects each sequence item to contain exactly two values"
        ));
    };
    let name = match to_usv_string_value(&name, context) {
        Ok(name) => name,
        Err(err) => return close_iterator_with_error(&pair_iterator, err, context),
    };

    let Some(value) = pair_iterator.step_value(context)? else {
        return Err(js_error!(
            TypeError: "URLSearchParams constructor expects each sequence item to contain exactly two values"
        ));
    };
    let value = match to_usv_string_value(&value, context) {
        Ok(value) => value,
        Err(err) => return close_iterator_with_error(&pair_iterator, err, context),
    };

    if pair_iterator.step_value(context)?.is_some() {
        return close_iterator_with_error(
            &pair_iterator,
            js_error!(
                TypeError: "URLSearchParams constructor expects each sequence item to contain exactly two values"
            ),
            context,
        );
    }

    Ok((name, value))
}

fn collect_sequence_pairs(
    init: &JsValue,
    iterator_method: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<(JsString, JsString)>> {
    let mut items = init.get_iterator_from_method(iterator_method, context)?;
    let mut pairs = Vec::new();

    while let Some(item) = items.step_value(context)? {
        let Some(item_object) = item.as_object() else {
            return close_iterator_with_error(
                &items,
                js_error!(
                    TypeError: "URLSearchParams constructor expects each sequence item to be an iterable pair"
                ),
                context,
            );
        };

        let pair = match collect_sequence_item_pair(&item_object, context) {
            Ok(pair) => pair,
            Err(err) => return close_iterator_with_error(&items, err, context),
        };
        pairs.push(pair);
    }

    Ok(pairs)
}

fn collect_record_pairs(
    object: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<(JsString, JsString)>> {
    let keys = object.own_property_keys(context)?;
    let mut pairs = Vec::new();

    for key in keys {
        let enumerable = OrdinaryObject::property_is_enumerable(
            &object.clone().into(),
            &[key.clone().into()],
            context,
        )?
        .to_boolean();

        if !enumerable {
            continue;
        }

        let name = to_usv_string_value(&JsValue::from(key.clone()), context)?;
        let value = to_usv_string_value(&object.get(key, context)?, context)?;
        pairs.push((name, value));
    }

    Ok(pairs)
}

/// The `URLSearchParams` class represents the query portion of a URL.
#[derive(Debug, JsData, Trace, Finalize)]
pub struct UrlSearchParams {
    list: Vec<(JsString, JsString)>,
    #[unsafe_ignore_trace]
    url: Option<Rc<RefCell<url::Url>>>,
}

impl UrlSearchParams {
    fn from_url(url: Rc<RefCell<url::Url>>, context: &mut Context) -> JsResult<JsObject<Self>> {
        Self::from_data(
            Self {
                list: Vec::new(),
                url: Some(url),
            },
            context,
        )?
        .downcast::<Self>()
        .map_err(|_| js_error!(Error: "URLSearchParams class should be registered"))
    }

    fn pairs(&self) -> Vec<(JsString, JsString)> {
        if let Some(url) = &self.url {
            let url = url.borrow();
            return url
                .query_pairs()
                .map(|(name, value)| {
                    (
                        JsString::from(name.as_ref()),
                        JsString::from(value.as_ref()),
                    )
                })
                .collect();
        }

        self.list.clone()
    }

    fn update(&mut self, pairs: Vec<(JsString, JsString)>) {
        if let Some(url) = &self.url {
            let mut url = url.borrow_mut();

            if pairs.is_empty() {
                url.set_query(None);
            } else {
                let query = serialize_search_params(&pairs);
                url.set_query(Some(&query));
            }
            return;
        }

        self.list = pairs;
    }
}

#[derive(Debug, JsData, Trace, Finalize)]
struct UrlSearchParamsIterator {
    search_params: JsObject<UrlSearchParams>,
    next_index: usize,
    #[unsafe_ignore_trace]
    kind: UrlSearchParamsIteratorKind,
    done: bool,
}

impl UrlSearchParamsIterator {
    fn create(
        search_params: JsObject<UrlSearchParams>,
        kind: UrlSearchParamsIteratorKind,
        context: &mut Context,
    ) -> JsValue {
        ObjectInitializer::with_native_data_and_proto(
            Self {
                search_params,
                next_index: 0,
                kind,
                done: false,
            },
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator(),
            context,
        )
        .function(
            NativeFunction::from_fn_ptr(Self::next),
            js_string!("next"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(Self::iterator),
            JsSymbol::iterator(),
            0,
        )
        .property(
            JsSymbol::to_string_tag(),
            js_string!("URLSearchParams Iterator"),
            Attribute::CONFIGURABLE,
        )
        .build()
        .into()
    }

    #[allow(clippy::unnecessary_wraps)]
    fn iterator(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(this.clone())
    }

    fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this
            .as_object()
            .ok_or_else(|| js_error!(TypeError: "`this` is not a URLSearchParams iterator"))?;
        let mut iterator = object
            .downcast_mut::<Self>()
            .ok_or_else(|| js_error!(TypeError: "`this` is not a URLSearchParams iterator"))?;

        if iterator.done {
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        let pair = iterator
            .search_params
            .borrow()
            .data()
            .pairs()
            .get(iterator.next_index)
            .cloned();

        let Some((name, value)) = pair else {
            iterator.done = true;
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        };

        iterator.next_index += 1;

        let result: JsValue = match iterator.kind {
            UrlSearchParamsIteratorKind::Key => name.into(),
            UrlSearchParamsIteratorKind::Value => value.into(),
            UrlSearchParamsIteratorKind::KeyAndValue => {
                JsArray::from_iter([name.into(), value.into()], context).into()
            }
        };

        Ok(create_iter_result_object(result, false, context))
    }
}

#[boa_class(rename = "URLSearchParams")]
#[boa(rename_all = "camelCase")]
impl UrlSearchParams {
    /// WHATWG URL: <https://url.spec.whatwg.org/#dom-urlsearchparams-urlsearchparams>
    ///
    /// This implements the constructor branches for:
    /// - empty / null init
    /// - sequence input via @@iterator
    /// - record input via enumerable own properties
    /// - string input parsed as application/x-www-form-urlencoded
    #[boa(constructor)]
    fn constructor(init: JsValue, context: &mut Context) -> JsResult<Self> {
        let list = if init.is_undefined() || init.is_null() {
            Vec::new()
        } else if let Some(object) = init.as_object() {
            if let Some(iterator_method) = object.get_method(JsSymbol::iterator(), context)? {
                collect_sequence_pairs(&init, &iterator_method, context)?
            } else {
                collect_record_pairs(&object, context)?
            }
        } else {
            parse_search_params(&to_usv_string_value(&init, context)?)
        };

        Ok(Self { list, url: None })
    }

    #[boa(getter)]
    fn size(&self) -> usize {
        self.pairs().len()
    }

    fn append(&mut self, name: Convert<JsString>, value: Convert<JsString>) {
        let mut pairs = self.pairs();
        pairs.push((to_usv_string(&name.0), to_usv_string(&value.0)));
        self.update(pairs);
    }

    fn delete(
        &mut self,
        name: Convert<JsString>,
        value: OptionalArg,
        context: &mut Context,
    ) -> JsResult<()> {
        // WHATWG URL: <https://url.spec.whatwg.org/#dom-urlsearchparams-delete>
        // The second argument only participates when it was actually supplied.
        let name = to_usv_string(&name.0);
        let value = value
            .0
            .as_ref()
            .map(|value| value.try_js_into::<Convert<JsString>>(context))
            .transpose()?
            .map(|value| to_usv_string(&value.0));
        let mut pairs = self.pairs();

        match value {
            Some(value) => {
                pairs.retain(|(existing_name, existing_value)| {
                    existing_name != &name || existing_value != &value
                });
            }
            None => {
                pairs.retain(|(existing_name, _)| existing_name != &name);
            }
        }

        self.update(pairs);
        Ok(())
    }

    fn entries(this: JsClass<Self>, context: &mut Context) -> JsValue {
        UrlSearchParamsIterator::create(
            this.inner(),
            UrlSearchParamsIteratorKind::KeyAndValue,
            context,
        )
    }

    #[boa(method)]
    fn for_each(
        this: JsClass<Self>,
        callback: SearchParamsForEachCallback,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<()> {
        let object = this.inner().upcast();
        let this_arg = this_arg.unwrap_or_default();
        let mut index = 0usize;

        loop {
            let pair = {
                let params = this.borrow();
                params.pairs().get(index).cloned()
            };
            let Some((name, value)) = pair else {
                break;
            };

            callback.call_with_this(&this_arg, context, (value, name, object.clone()))?;
            index += 1;
        }

        Ok(())
    }

    fn get(&self, name: Convert<JsString>) -> JsValue {
        let name = to_usv_string(&name.0);
        self.pairs()
            .into_iter()
            .find_map(|(existing_name, value)| (existing_name == name).then_some(value.into()))
            .unwrap_or_else(JsValue::null)
    }

    fn get_all(&self, name: Convert<JsString>) -> Vec<JsString> {
        let name = to_usv_string(&name.0);
        self.pairs()
            .into_iter()
            .filter_map(|(existing_name, value)| (existing_name == name).then_some(value))
            .collect()
    }

    fn has(
        &self,
        name: Convert<JsString>,
        value: OptionalArg,
        context: &mut Context,
    ) -> JsResult<bool> {
        // WHATWG URL: <https://url.spec.whatwg.org/#dom-urlsearchparams-has>
        // The second argument only participates when it was actually supplied.
        let name = to_usv_string(&name.0);
        let value = value
            .0
            .as_ref()
            .map(|value| value.try_js_into::<Convert<JsString>>(context))
            .transpose()?
            .map(|value| to_usv_string(&value.0));

        Ok(match value {
            Some(value) => self
                .pairs()
                .into_iter()
                .any(|(existing_name, existing_value)| {
                    existing_name == name && existing_value == value
                }),
            None => self
                .pairs()
                .into_iter()
                .any(|(existing_name, _)| existing_name == name),
        })
    }

    #[boa(symbol = "iterator")]
    fn iterator(this: JsClass<Self>, context: &mut Context) -> JsValue {
        Self::entries(this, context)
    }

    fn keys(this: JsClass<Self>, context: &mut Context) -> JsValue {
        UrlSearchParamsIterator::create(this.inner(), UrlSearchParamsIteratorKind::Key, context)
    }

    fn set(&mut self, name: Convert<JsString>, value: Convert<JsString>) {
        let name = to_usv_string(&name.0);
        let value = to_usv_string(&value.0);
        let mut found = false;
        let mut result = Vec::with_capacity(self.pairs().len() + 1);

        for (existing_name, existing_value) in self.pairs() {
            if existing_name == name {
                if !found {
                    result.push((existing_name, value.clone()));
                    found = true;
                }
            } else {
                result.push((existing_name, existing_value));
            }
        }

        if !found {
            result.push((name, value));
        }

        self.update(result);
    }

    fn sort(&mut self) {
        let mut pairs = self.pairs();
        pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
        self.update(pairs);
    }

    fn to_string(&self) -> JsString {
        JsString::from(serialize_search_params(&self.pairs()))
    }

    fn values(this: JsClass<Self>, context: &mut Context) -> JsValue {
        UrlSearchParamsIterator::create(this.inner(), UrlSearchParamsIteratorKind::Value, context)
    }
}

/// The `URL` class represents a (properly parsed) Uniform Resource Locator.
#[derive(Debug, JsData, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub struct Url {
    #[unsafe_ignore_trace]
    inner: Rc<RefCell<url::Url>>,
    search_params: Option<JsObject<UrlSearchParams>>,
}

impl Url {
    /// Register the `URL` class into the realm. Pass `None` for the realm to
    /// register globally.
    ///
    /// # Errors
    /// This will error if the context or realm cannot register the class.
    pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
        js_module::boa_register(realm, context)
    }

    /// Create a native `Url` value from Rust code.
    ///
    /// # Errors
    /// Returns an error if `url` cannot be parsed against the optional `base`.
    pub fn new(Convert(ref url): Convert<String>, base: Option<Convert<String>>) -> JsResult<Self> {
        Ok(Self {
            inner: Rc::new(RefCell::new(Self::parse_url(
                url,
                base.as_ref().map(|base| base.0.as_str()),
            )?)),
            search_params: None,
        })
    }

    /// Create a JavaScript `URL` object from native `Url` data.
    ///
    /// # Errors
    /// Returns an error if the object or its eagerly-created `searchParams`
    /// view cannot be allocated.
    pub fn from_data(mut data: Self, context: &mut Context) -> JsResult<JsObject> {
        if data.search_params.is_none() {
            data.search_params = Some(UrlSearchParams::from_url(data.inner.clone(), context)?);
        }

        <Self as Class>::from_data(data, context)
    }

    fn parse_url(url: &str, base: Option<&str>) -> JsResult<url::Url> {
        if let Some(base) = base {
            let base_url = url::Url::parse(base)
                .map_err(|e| js_error!(TypeError: "Failed to parse base URL: {}", e))?;

            base_url
                .join(url)
                .map_err(|e| js_error!(TypeError: "Failed to parse URL: {}", e))
        } else {
            url::Url::parse(url).map_err(|e| js_error!(TypeError: "Failed to parse URL: {}", e))
        }
    }

    fn from_parsed(url: url::Url, context: &mut Context) -> JsResult<Self> {
        let inner = Rc::new(RefCell::new(url));
        let search_params = UrlSearchParams::from_url(inner.clone(), context)?;

        Ok(Self {
            inner,
            search_params: Some(search_params),
        })
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.borrow())
    }
}

#[boa_class(rename = "URL")]
#[boa(rename_all = "camelCase")]
impl Url {
    /// Create a new `URL` object. Meant to be called from the JavaScript constructor.
    ///
    /// # Errors
    /// Any errors that might occur during URL parsing.
    #[boa(constructor)]
    pub fn constructor(
        Convert(ref url): Convert<String>,
        base: Option<Convert<String>>,
        context: &mut Context,
    ) -> JsResult<Self> {
        Self::from_parsed(
            Self::parse_url(url, base.as_ref().map(|base| base.0.as_str()))?,
            context,
        )
    }

    #[boa(getter)]
    fn hash(&self) -> JsString {
        JsString::from(url::quirks::hash(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "hash")]
    fn set_hash(&mut self, value: Convert<String>) {
        url::quirks::set_hash(&mut self.inner.borrow_mut(), &value.0);
    }

    #[boa(getter)]
    fn hostname(&self) -> JsString {
        JsString::from(url::quirks::hostname(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "hostname")]
    fn set_hostname(&mut self, value: Convert<String>) {
        let _ = url::quirks::set_hostname(&mut self.inner.borrow_mut(), &value.0);
    }

    #[boa(getter)]
    fn host(&self) -> JsString {
        JsString::from(url::quirks::host(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "host")]
    fn set_host(&mut self, value: Convert<String>) {
        let _ = url::quirks::set_host(&mut self.inner.borrow_mut(), &value.0);
    }

    #[boa(getter)]
    fn href(&self) -> JsString {
        JsString::from(url::quirks::href(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "href")]
    fn set_href(&mut self, value: Convert<String>) -> JsResult<()> {
        url::quirks::set_href(&mut self.inner.borrow_mut(), &value.0)
            .map_err(|e| js_error!(TypeError: "Failed to set href: {}", e))
    }

    #[boa(getter)]
    fn origin(&self) -> JsString {
        JsString::from(url::quirks::origin(&self.inner.borrow()))
    }

    #[boa(getter)]
    fn password(&self) -> JsString {
        JsString::from(url::quirks::password(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "password")]
    fn set_password(&mut self, value: Convert<String>) {
        let _ = url::quirks::set_password(&mut self.inner.borrow_mut(), &value.0);
    }

    #[boa(getter)]
    fn pathname(&self) -> JsString {
        JsString::from(url::quirks::pathname(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "pathname")]
    fn set_pathname(&mut self, value: Convert<String>) {
        let () = url::quirks::set_pathname(&mut self.inner.borrow_mut(), &value.0);
    }

    #[boa(getter)]
    fn port(&self) -> JsString {
        JsString::from(url::quirks::port(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "port")]
    fn set_port(&mut self, value: Convert<JsString>) {
        let _ = url::quirks::set_port(&mut self.inner.borrow_mut(), &value.0.to_std_string_lossy());
    }

    #[boa(getter)]
    fn protocol(&self) -> JsString {
        JsString::from(url::quirks::protocol(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "protocol")]
    fn set_protocol(&mut self, value: Convert<String>) {
        let _ = url::quirks::set_protocol(&mut self.inner.borrow_mut(), &value.0);
    }

    #[boa(getter)]
    fn search(&self) -> JsString {
        JsString::from(url::quirks::search(&self.inner.borrow()))
    }

    #[boa(setter)]
    #[boa(rename = "search")]
    fn set_search(&mut self, value: Convert<String>) {
        url::quirks::set_search(&mut self.inner.borrow_mut(), &value.0);
    }

    #[boa(getter)]
    fn search_params(&self) -> JsObject {
        // WHATWG URL: <https://url.spec.whatwg.org/#dom-url-searchparams>
        // `searchParams` is created during URL construction and the getter
        // returns that same live object.
        self.search_params
            .clone()
            .expect("URL.searchParams should be initialized during construction")
            .upcast()
    }

    #[boa(getter)]
    fn username(&self) -> JsString {
        JsString::from(self.inner.borrow().username())
    }

    #[boa(setter)]
    #[boa(rename = "username")]
    fn set_username(&mut self, value: Convert<String>) {
        let _ = self.inner.borrow_mut().set_username(&value.0);
    }

    fn to_string(&self) -> JsString {
        JsString::from(format!("{}", self.inner.borrow()))
    }

    #[boa(rename = "toJSON")]
    fn to_json(&self) -> JsString {
        JsString::from(format!("{}", self.inner.borrow()))
    }

    #[boa(static)]
    fn create_object_url() -> JsResult<()> {
        Err(js_error!(Error: "URL.createObjectURL is not implemented"))
    }

    #[boa(static)]
    fn can_parse(url: Convert<String>, base: Option<Convert<String>>) -> bool {
        Self::parse_url(&url.0, base.as_ref().map(|base| base.0.as_str())).is_ok()
    }

    #[boa(static)]
    fn parse(
        url: Convert<String>,
        base: Option<Convert<String>>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::parse_url(&url.0, base.as_ref().map(|base| base.0.as_str()))
            .map_or(Ok(JsValue::null()), |url| {
                Self::from_data(Self::from_parsed(url, context)?, context).map(JsValue::from)
            })
    }

    #[boa(static)]
    fn revoke_object_url() -> JsResult<()> {
        Err(js_error!(Error: "URL.revokeObjectURL is not implemented"))
    }
}

/// JavaScript module containing the Url class.
#[boa_module]
pub mod js_module {
    type Url = super::Url;
    type UrlSearchParams = super::UrlSearchParams;
}
