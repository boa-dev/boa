use crate::{
    builtins::intl::date_time_format::{to_date_time_options, DateTimeReqs},
    builtins::intl::{
        best_available_locale, best_fit_matcher, default_locale, default_number_option,
        get_number_option, get_option, insert_unicode_extension_and_canonicalize, lookup_matcher,
        resolve_locale, unicode_extension_components, DateTimeFormatRecord, GetOptionType,
    },
    object::JsObject,
    Context, JsString, JsValue,
};

use icu_locale_canonicalizer::LocaleCanonicalizer;
use rustc_hash::FxHashMap;

#[test]
fn best_avail_loc() {
    let no_extensions_locale = JsString::new("en-US");
    let available_locales = Vec::<JsString>::new();
    assert_eq!(
        best_available_locale(&available_locales, &no_extensions_locale,),
        None
    );

    let no_extensions_locale = JsString::new("de-DE");
    let available_locales = vec![no_extensions_locale.clone()];
    assert_eq!(
        best_available_locale(&available_locales, &no_extensions_locale,),
        Some(no_extensions_locale)
    );

    let locale_part = "fr".to_string();
    let no_extensions_locale = JsString::new(locale_part.clone() + "-CA");
    let available_locales = vec![JsString::new(locale_part.clone())];
    assert_eq!(
        best_available_locale(&available_locales, &no_extensions_locale,),
        Some(JsString::new(locale_part))
    );

    let ja_kana_t = JsString::new("ja-Kana-JP-t");
    let ja_kana = JsString::new("ja-Kana-JP");
    let no_extensions_locale = JsString::new("ja-Kana-JP-t-it-latn-it");
    let available_locales = vec![ja_kana_t, ja_kana.clone()];
    assert_eq!(
        best_available_locale(&available_locales, &no_extensions_locale,),
        Some(ja_kana)
    );
}

#[test]
fn lookup_match() {
    let provider = icu_testdata::get_provider();
    let canonicalizer =
        LocaleCanonicalizer::new(&provider).expect("Could not create canonicalizer");
    // available: [], requested: []
    let available_locales = Vec::<JsString>::new();
    let requested_locales = Vec::<JsString>::new();

    let matcher = lookup_matcher(&available_locales, &requested_locales, &canonicalizer);
    assert_eq!(
        matcher.locale,
        default_locale(&canonicalizer).to_string().as_str()
    );
    assert_eq!(matcher.extension, "");

    // available: [de-DE], requested: []
    let available_locales = vec![JsString::new("de-DE")];
    let requested_locales = Vec::<JsString>::new();

    let matcher = lookup_matcher(&available_locales, &requested_locales, &canonicalizer);
    assert_eq!(
        matcher.locale,
        default_locale(&canonicalizer).to_string().as_str()
    );
    assert_eq!(matcher.extension, "");

    // available: [fr-FR], requested: [fr-FR-u-hc-h12]
    let available_locales = vec![JsString::new("fr-FR")];
    let requested_locales = vec![JsString::new("fr-FR-u-hc-h12")];

    let matcher = lookup_matcher(&available_locales, &requested_locales, &canonicalizer);
    assert_eq!(matcher.locale, "fr-FR");
    assert_eq!(matcher.extension, "u-hc-h12");

    // available: [es-ES], requested: [es-ES]
    let available_locales = vec![JsString::new("es-ES")];
    let requested_locales = vec![JsString::new("es-ES")];

    let matcher = best_fit_matcher(&available_locales, &requested_locales, &canonicalizer);
    assert_eq!(matcher.locale, "es-ES");
    assert_eq!(matcher.extension, "");
}

#[test]
fn insert_unicode_ext() {
    let provider = icu_testdata::get_provider();
    let canonicalizer =
        LocaleCanonicalizer::new(&provider).expect("Could not create canonicalizer");
    let locale = JsString::new("hu-HU");
    let ext = JsString::empty();
    assert_eq!(
        insert_unicode_extension_and_canonicalize(&locale, &ext, &canonicalizer),
        locale
    );

    let locale = JsString::new("hu-HU");
    let ext = JsString::new("-u-hc-h12");
    assert_eq!(
        insert_unicode_extension_and_canonicalize(&locale, &ext, &canonicalizer),
        JsString::new("hu-HU-u-hc-h12")
    );

    let locale = JsString::new("hu-HU-x-PRIVATE");
    let ext = JsString::new("-u-hc-h12");
    assert_eq!(
        insert_unicode_extension_and_canonicalize(&locale, &ext, &canonicalizer),
        JsString::new("hu-HU-u-hc-h12-x-private")
    );
}

#[test]
fn uni_ext_comp() {
    let ext = JsString::new("-u-ca-japanese-hc-h12");
    let components = unicode_extension_components(&ext);
    assert!(components.attributes.is_empty());
    assert_eq!(components.keywords.len(), 2);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "japanese");
    assert_eq!(components.keywords[1].key, "hc");
    assert_eq!(components.keywords[1].value, "h12");

    let ext = JsString::new("-u-alias-co-phonebk-ka-shifted");
    let components = unicode_extension_components(&ext);
    assert_eq!(components.attributes, vec![JsString::new("alias")]);
    assert_eq!(components.keywords.len(), 2);
    assert_eq!(components.keywords[0].key, "co");
    assert_eq!(components.keywords[0].value, "phonebk");
    assert_eq!(components.keywords[1].key, "ka");
    assert_eq!(components.keywords[1].value, "shifted");

    let ext = JsString::new("-u-ca-buddhist-kk-nu-thai");
    let components = unicode_extension_components(&ext);
    assert!(components.attributes.is_empty());
    assert_eq!(components.keywords.len(), 3);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "buddhist");
    assert_eq!(components.keywords[1].key, "kk");
    assert_eq!(components.keywords[1].value, "");
    assert_eq!(components.keywords[2].key, "nu");
    assert_eq!(components.keywords[2].value, "thai");

    let ext = JsString::new("-u-ca-islamic-civil");
    let components = unicode_extension_components(&ext);
    assert!(components.attributes.is_empty());
    assert_eq!(components.keywords.len(), 1);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "islamic-civil");
}

#[test]
fn locale_resolution() {
    let mut context = Context::default();

    // test lookup
    let available_locales = Vec::<JsString>::new();
    let requested_locales = Vec::<JsString>::new();
    let relevant_extension_keys = Vec::<JsString>::new();
    let locale_data = FxHashMap::default();
    let options = DateTimeFormatRecord {
        locale_matcher: JsString::new("lookup"),
        properties: FxHashMap::default(),
    };

    let locale_record = resolve_locale(
        &available_locales,
        &requested_locales,
        &options,
        &relevant_extension_keys,
        &locale_data,
        &mut context,
    );
    assert_eq!(
        locale_record.locale,
        default_locale(context.icu().locale_canonicalizer())
            .to_string()
            .as_str()
    );
    assert_eq!(
        locale_record.data_locale,
        default_locale(context.icu().locale_canonicalizer())
            .to_string()
            .as_str()
    );
    assert!(locale_record.properties.is_empty());

    // test best fit
    let available_locales = Vec::<JsString>::new();
    let requested_locales = Vec::<JsString>::new();
    let relevant_extension_keys = Vec::<JsString>::new();
    let locale_data = FxHashMap::default();
    let options = DateTimeFormatRecord {
        locale_matcher: JsString::new("best-fit"),
        properties: FxHashMap::default(),
    };

    let locale_record = resolve_locale(
        &available_locales,
        &requested_locales,
        &options,
        &relevant_extension_keys,
        &locale_data,
        &mut context,
    );
    assert_eq!(
        locale_record.locale,
        default_locale(context.icu().locale_canonicalizer())
            .to_string()
            .as_str()
    );
    assert_eq!(
        locale_record.data_locale,
        default_locale(context.icu().locale_canonicalizer())
            .to_string()
            .as_str()
    );
    assert!(locale_record.properties.is_empty());

    // available: [es-ES], requested: [es-ES]
    let available_locales = vec![JsString::new("es-ES")];
    let requested_locales = vec![JsString::new("es-ES")];
    let relevant_extension_keys = Vec::<JsString>::new();
    let locale_data = FxHashMap::default();
    let options = DateTimeFormatRecord {
        locale_matcher: JsString::new("lookup"),
        properties: FxHashMap::default(),
    };

    let locale_record = resolve_locale(
        &available_locales,
        &requested_locales,
        &options,
        &relevant_extension_keys,
        &locale_data,
        &mut context,
    );
    assert_eq!(locale_record.locale, "es-ES");
    assert_eq!(locale_record.data_locale, "es-ES");
    assert!(locale_record.properties.is_empty());

    // available: [zh-CN], requested: []
    let available_locales = vec![JsString::new("zh-CN")];
    let requested_locales = Vec::<JsString>::new();
    let relevant_extension_keys = Vec::<JsString>::new();
    let locale_data = FxHashMap::default();
    let options = DateTimeFormatRecord {
        locale_matcher: JsString::new("lookup"),
        properties: FxHashMap::default(),
    };

    let locale_record = resolve_locale(
        &available_locales,
        &requested_locales,
        &options,
        &relevant_extension_keys,
        &locale_data,
        &mut context,
    );
    assert_eq!(
        locale_record.locale,
        default_locale(context.icu().locale_canonicalizer())
            .to_string()
            .as_str()
    );
    assert_eq!(
        locale_record.data_locale,
        default_locale(context.icu().locale_canonicalizer())
            .to_string()
            .as_str()
    );
    assert!(locale_record.properties.is_empty());
}

#[test]
fn get_opt() {
    let mut context = Context::default();

    let values = Vec::<JsString>::new();
    let fallback = JsValue::String(JsString::new("fallback"));
    let options_obj = JsObject::empty();
    let option_type = GetOptionType::String;
    let get_option_result = get_option(
        &options_obj,
        "",
        &option_type,
        &values,
        &fallback,
        &mut context,
    )
    .expect("GetOption should not fail on fallback test");
    assert_eq!(get_option_result, fallback);

    let values = Vec::<JsString>::new();
    let fallback = JsValue::String(JsString::new("fallback"));
    let options_obj = JsObject::empty();
    let locale_value = JsValue::String(JsString::new("en-US"));
    options_obj
        .set("Locale", locale_value.clone(), true, &mut context)
        .expect("Setting a property should not fail");
    let option_type = GetOptionType::String;
    let get_option_result = get_option(
        &options_obj,
        "Locale",
        &option_type,
        &values,
        &fallback,
        &mut context,
    )
    .expect("GetOption should not fail on string test");
    assert_eq!(get_option_result, locale_value);

    let fallback = JsValue::String(JsString::new("fallback"));
    let options_obj = JsObject::empty();
    let locale_string = JsString::new("en-US");
    let locale_value = JsValue::String(locale_string.clone());
    let values = vec![locale_string];
    options_obj
        .set("Locale", locale_value.clone(), true, &mut context)
        .expect("Setting a property should not fail");
    let option_type = GetOptionType::String;
    let get_option_result = get_option(
        &options_obj,
        "Locale",
        &option_type,
        &values,
        &fallback,
        &mut context,
    )
    .expect("GetOption should not fail on values test");
    assert_eq!(get_option_result, locale_value);

    let fallback = JsValue::new(false);
    let options_obj = JsObject::empty();
    let boolean_value = JsValue::new(true);
    let values = Vec::<JsString>::new();
    options_obj
        .set("boolean_val", boolean_value.clone(), true, &mut context)
        .expect("Setting a property should not fail");
    let option_type = GetOptionType::Boolean;
    let get_option_result = get_option(
        &options_obj,
        "boolean_val",
        &option_type,
        &values,
        &fallback,
        &mut context,
    )
    .expect("GetOption should not fail on boolean test");
    assert_eq!(get_option_result, boolean_value);

    let fallback = JsValue::String(JsString::new("fallback"));
    let options_obj = JsObject::empty();
    let locale_value = JsValue::String(JsString::new("en-US"));
    let other_locale_str = JsString::new("de-DE");
    let values = vec![other_locale_str];
    options_obj
        .set("Locale", locale_value, true, &mut context)
        .expect("Setting a property should not fail");
    let option_type = GetOptionType::String;
    let get_option_result = get_option(
        &options_obj,
        "Locale",
        &option_type,
        &values,
        &fallback,
        &mut context,
    );
    assert!(get_option_result.is_err());

    let value = JsValue::undefined();
    let minimum = 1.0;
    let maximum = 10.0;
    let fallback_val = 5.0;
    let fallback = Some(fallback_val);
    let get_option_result = default_number_option(&value, minimum, maximum, fallback, &mut context);
    assert_eq!(get_option_result, Ok(fallback));

    let value = JsValue::nan();
    let minimum = 1.0;
    let maximum = 10.0;
    let fallback = Some(5.0);
    let get_option_result = default_number_option(&value, minimum, maximum, fallback, &mut context);
    assert!(get_option_result.is_err());

    let value = JsValue::new(0);
    let minimum = 1.0;
    let maximum = 10.0;
    let fallback = Some(5.0);
    let get_option_result = default_number_option(&value, minimum, maximum, fallback, &mut context);
    assert!(get_option_result.is_err());

    let value = JsValue::new(11);
    let minimum = 1.0;
    let maximum = 10.0;
    let fallback = Some(5.0);
    let get_option_result = default_number_option(&value, minimum, maximum, fallback, &mut context);
    assert!(get_option_result.is_err());

    let value_f64 = 7.0;
    let value = JsValue::new(value_f64);
    let minimum = 1.0;
    let maximum = 10.0;
    let fallback = Some(5.0);
    let get_option_result = default_number_option(&value, minimum, maximum, fallback, &mut context);
    assert_eq!(get_option_result, Ok(Some(value_f64)));

    let options = JsObject::empty();
    let property = "fractionalSecondDigits";
    let minimum = 1.0;
    let maximum = 10.0;
    let fallback_val = 5.0;
    let fallback = Some(fallback_val);
    let get_option_result =
        get_number_option(&options, property, minimum, maximum, fallback, &mut context);
    assert_eq!(get_option_result, Ok(fallback));

    let options = JsObject::empty();
    let value_f64 = 8.0;
    let value = JsValue::new(value_f64);
    let property = "fractionalSecondDigits";
    options
        .set(property, value, true, &mut context)
        .expect("Setting a property should not fail");
    let minimum = 1.0;
    let maximum = 10.0;
    let fallback = Some(5.0);
    let get_option_result =
        get_number_option(&options, property, minimum, maximum, fallback, &mut context);
    assert_eq!(get_option_result, Ok(Some(value_f64)));
}

#[test]
fn to_date_time_opts() {
    let mut context = Context::default();

    let options_obj = JsObject::empty();
    options_obj
        .set("timeStyle", JsObject::empty(), true, &mut context)
        .expect("Setting a property should not fail");
    let date_time_opts = to_date_time_options(
        &JsValue::new(options_obj),
        &DateTimeReqs::Date,
        &DateTimeReqs::Date,
        &mut context,
    );
    assert!(date_time_opts.is_err());

    let options_obj = JsObject::empty();
    options_obj
        .set("dateStyle", JsObject::empty(), true, &mut context)
        .expect("Setting a property should not fail");
    let date_time_opts = to_date_time_options(
        &JsValue::new(options_obj),
        &DateTimeReqs::Time,
        &DateTimeReqs::Time,
        &mut context,
    );
    assert!(date_time_opts.is_err());

    let date_time_opts = to_date_time_options(
        &JsValue::undefined(),
        &DateTimeReqs::Date,
        &DateTimeReqs::Date,
        &mut context,
    )
    .expect("toDateTimeOptions should not fail in date test");

    let numeric_jsstring = JsValue::String(JsString::new("numeric"));
    assert_eq!(
        date_time_opts.get("year", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("month", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("day", &mut context),
        Ok(numeric_jsstring)
    );

    let date_time_opts = to_date_time_options(
        &JsValue::undefined(),
        &DateTimeReqs::Time,
        &DateTimeReqs::Time,
        &mut context,
    )
    .expect("toDateTimeOptions should not fail in time test");

    let numeric_jsstring = JsValue::String(JsString::new("numeric"));
    assert_eq!(
        date_time_opts.get("hour", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("minute", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("second", &mut context),
        Ok(numeric_jsstring)
    );

    let date_time_opts = to_date_time_options(
        &JsValue::undefined(),
        &DateTimeReqs::AnyAll,
        &DateTimeReqs::AnyAll,
        &mut context,
    )
    .expect("toDateTimeOptions should not fail when testing required = 'any'");

    let numeric_jsstring = JsValue::String(JsString::new("numeric"));
    assert_eq!(
        date_time_opts.get("year", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("month", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("day", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("hour", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("minute", &mut context),
        Ok(numeric_jsstring.clone())
    );
    assert_eq!(
        date_time_opts.get("second", &mut context),
        Ok(numeric_jsstring)
    );
}
