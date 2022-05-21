use crate::{
    builtins::intl::date_time_format::{
        basic_format_matcher, build_formats, canonicalize_time_zone_name, date_time_style_format,
        is_valid_time_zone_name, month_to_value, numeric_to_value, string_to_hour_cycle,
        text_to_value, time_zone_to_value, to_date_time_options, value_to_date_style,
        value_to_time_style, year_to_value, DateTimeReqs, FormatOptionsRecord, StylesRecord,
    },
    builtins::intl::{
        best_available_locale, best_fit_matcher, default_locale, default_number_option,
        get_number_option, get_option, insert_unicode_extension_and_canonicalize, lookup_matcher,
        resolve_locale, unicode_extension_components, DateTimeFormatRecord, GetOptionType,
    },
    object::JsObject,
    Context, JsString, JsValue,
};

use icu::datetime::{
    options::{components, length, preferences},
    DateTimeFormatOptions,
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

    let ext = JsString::new("u-ca-islamic-civil");
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

#[test]
fn nonterminals() {
    let nonterminal_calendar_options = vec![
        JsString::new(""),
        JsString::new("a"),
        JsString::new("ab"),
        JsString::new("abcdefghi"),
        JsString::new("abc-abcdefghi"),
        JsString::new("!invalid!"),
        JsString::new("-gregory-"),
        JsString::new("gregory-"),
        JsString::new("gregory--"),
        JsString::new("gregory-nu"),
        JsString::new("gregory-nu-"),
        JsString::new("gregory-nu-latn"),
        JsString::new("gregoryé"),
        JsString::new("gregory역법"),
    ];

    for calendar_opt in nonterminal_calendar_options {
        assert_eq!(
            crate::builtins::intl::date_time_format::is_terminal(&calendar_opt),
            true
        );
    }

    let terminal_calendar_options = vec![
        JsString::new("buddhist"),
        JsString::new("chinese"),
        JsString::new("coptic"),
        JsString::new("dangi"),
        JsString::new("ethioaa"),
        JsString::new("ethiopic"),
        JsString::new("gregory"),
        JsString::new("hebrew"),
        JsString::new("indian"),
        JsString::new("islamic"),
        JsString::new("islamic-umalqura"),
        JsString::new("islamic-tbla"),
        JsString::new("islamic-civil"),
        JsString::new("islamic-rgsa"),
        JsString::new("iso8601"),
        JsString::new("japanese"),
        JsString::new("persian"),
        JsString::new("roc"),
    ];

    for calendar_opt in terminal_calendar_options {
        assert_eq!(
            crate::builtins::intl::date_time_format::is_terminal(&calendar_opt),
            false
        );
    }
}

#[test]
fn build_date_time_fmt() {
    let mut context = Context::default();

    let date_time_fmt_obj = crate::builtins::intl::date_time_format::DateTimeFormat::constructor(
        &JsValue::undefined(),
        &Vec::<JsValue>::new(),
        &mut context,
    );
    assert_eq!(date_time_fmt_obj.is_err(), false);
}

#[test]
fn is_valid_tz() {
    assert_eq!(is_valid_time_zone_name(&JsString::new("UTC")), true);
    assert_eq!(is_valid_time_zone_name(&JsString::new("Israel")), true);
    assert_eq!(
        is_valid_time_zone_name(&JsString::new("Atlantic/Reykjavik")),
        true
    );
    assert_eq!(is_valid_time_zone_name(&JsString::new("Etc/Zulu")), true);
    assert_eq!(
        is_valid_time_zone_name(&JsString::new("Etc/Jamaica")),
        false
    );

    println!(
        "DEBUG: {}",
        canonicalize_time_zone_name(&JsString::new("Brazil/West")).to_string()
    );
}

#[test]
fn js_to_dtf() {
    let mut context = Context::default();

    assert_eq!(
        string_to_hour_cycle(&JsString::new("h11")),
        preferences::HourCycle::H11
    );
    assert_eq!(
        string_to_hour_cycle(&JsString::new("h12")),
        preferences::HourCycle::H12
    );
    assert_eq!(
        string_to_hour_cycle(&JsString::new("h23")),
        preferences::HourCycle::H23
    );
    assert_eq!(
        string_to_hour_cycle(&JsString::new("h24")),
        preferences::HourCycle::H24
    );

    assert_eq!(
        value_to_date_style(&JsValue::String(JsString::new("full")), &mut context),
        Some(length::Date::Full)
    );
    assert_eq!(
        value_to_date_style(&JsValue::String(JsString::new("long")), &mut context),
        Some(length::Date::Long)
    );
    assert_eq!(
        value_to_date_style(&JsValue::String(JsString::new("medium")), &mut context),
        Some(length::Date::Medium)
    );
    assert_eq!(
        value_to_date_style(&JsValue::String(JsString::new("short")), &mut context),
        Some(length::Date::Short)
    );
    assert_eq!(
        value_to_date_style(&JsValue::String(JsString::new("narrow")), &mut context),
        None
    );

    assert_eq!(
        value_to_time_style(&JsValue::String(JsString::new("full")), &mut context),
        Some(length::Time::Full)
    );
    assert_eq!(
        value_to_time_style(&JsValue::String(JsString::new("long")), &mut context),
        Some(length::Time::Long)
    );
    assert_eq!(
        value_to_time_style(&JsValue::String(JsString::new("medium")), &mut context),
        Some(length::Time::Medium)
    );
    assert_eq!(
        value_to_time_style(&JsValue::String(JsString::new("short")), &mut context),
        Some(length::Time::Short)
    );
    assert_eq!(
        value_to_time_style(&JsValue::String(JsString::new("narrow")), &mut context),
        None
    );

    assert_eq!(
        text_to_value(Some(components::Text::Long)),
        JsValue::String(JsString::from("long"))
    );
    assert_eq!(
        text_to_value(Some(components::Text::Short)),
        JsValue::String(JsString::from("short"))
    );
    assert_eq!(
        text_to_value(Some(components::Text::Narrow)),
        JsValue::String(JsString::from("narrow"))
    );
    assert_eq!(text_to_value(None), JsValue::undefined());

    assert_eq!(
        year_to_value(Some(components::Year::Numeric)),
        JsValue::String(JsString::from("numeric"))
    );
    assert_eq!(
        year_to_value(Some(components::Year::TwoDigit)),
        JsValue::String(JsString::from("2-digit"))
    );
    assert_eq!(
        year_to_value(Some(components::Year::NumericWeekOf)),
        JsValue::String(JsString::from("numericWeek"))
    );
    assert_eq!(
        year_to_value(Some(components::Year::TwoDigitWeekOf)),
        JsValue::String(JsString::from("2-digitWeek"))
    );
    assert_eq!(year_to_value(None), JsValue::undefined());

    assert_eq!(
        month_to_value(Some(components::Month::Numeric)),
        JsValue::String(JsString::from("numeric"))
    );
    assert_eq!(
        month_to_value(Some(components::Month::TwoDigit)),
        JsValue::String(JsString::from("2-digit"))
    );
    assert_eq!(
        month_to_value(Some(components::Month::Long)),
        JsValue::String(JsString::from("long"))
    );
    assert_eq!(
        month_to_value(Some(components::Month::Short)),
        JsValue::String(JsString::from("short"))
    );
    assert_eq!(
        month_to_value(Some(components::Month::Narrow)),
        JsValue::String(JsString::from("narrow"))
    );
    assert_eq!(month_to_value(None), JsValue::undefined());

    assert_eq!(
        numeric_to_value(Some(components::Numeric::Numeric)),
        JsValue::String(JsString::from("numeric"))
    );
    assert_eq!(
        numeric_to_value(Some(components::Numeric::TwoDigit)),
        JsValue::String(JsString::from("2-digit"))
    );
    assert_eq!(numeric_to_value(None), JsValue::undefined());

    assert_eq!(
        time_zone_to_value(Some(components::TimeZoneName::ShortSpecific)),
        JsValue::String(JsString::from("short"))
    );
    assert_eq!(
        time_zone_to_value(Some(components::TimeZoneName::LongSpecific)),
        JsValue::String(JsString::from("long"))
    );
    assert_eq!(
        time_zone_to_value(Some(components::TimeZoneName::GmtOffset)),
        JsValue::String(JsString::from("gmt"))
    );
    assert_eq!(
        time_zone_to_value(Some(components::TimeZoneName::ShortGeneric)),
        JsValue::String(JsString::from("shortGeneric"))
    );
    assert_eq!(
        time_zone_to_value(Some(components::TimeZoneName::LongGeneric)),
        JsValue::String(JsString::from("longGeneric"))
    );
    assert_eq!(time_zone_to_value(None), JsValue::undefined());
}

#[test]
fn build_fmts() {
    let mut context = Context::default();

    let formats = build_formats(&JsString::new("fr"), &JsString::new("gregory"));
    assert_eq!(formats.is_empty(), false);

    let formats = build_formats(&JsString::new("de-DE"), &JsString::new("buddhist"));
    assert_eq!(formats.is_empty(), false);

    let formats = build_formats(&JsString::new("ja-Kana-JP"), &JsString::new("japanese"));
    assert_eq!(formats.is_empty(), false);

    let formats = build_formats(&JsString::new("it"), &JsString::new("julian"));
    assert_eq!(formats.is_empty(), true);

    let formats = build_formats(&JsString::new("en-US"), &JsString::new("gregory"));
    assert_eq!(formats.is_empty(), false);

    let format_options = FormatOptionsRecord {
        date_time_format_opts: DateTimeFormatOptions::default(),
        components: FxHashMap::default(),
    };
    assert_eq!(
        basic_format_matcher(&format_options, &formats).is_none(),
        false
    );

    let styles = StylesRecord {
        locale: JsString::new("en-US"),
        calendar: JsString::new("gregory"),
    };

    let date_style = JsValue::String(JsString::from("full"));
    let time_style = JsValue::String(JsString::from("full"));
    assert_eq!(
        date_time_style_format(&date_style, &time_style, &styles, &mut context).is_none(),
        false
    );
}
