#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "BoaContext.h"
#include "JsError.h"
#include "JsValue.h"

static DiplomatStringView view(const char *source) {
    return (DiplomatStringView){.data = source, .len = strlen(source)};
}

static JsValue *eval_value(BoaContext *context, DiplomatStringView source) {
    boa_BoaContext_eval_result result = boa_BoaContext_eval(context, source);
    if (result.is_ok) {
        return result.ok;
    }
    boa_JsError_destroy(result.err);
    return NULL;
}

static JsError *eval_error(BoaContext *context, DiplomatStringView source) {
    boa_BoaContext_eval_result result = boa_BoaContext_eval(context, source);
    if (!result.is_ok) {
        return result.err;
    }
    if (result.ok != NULL) {
        boa_JsValue_destroy(result.ok);
    }
    return NULL;
}

static bool has_type(BoaContext *context, DiplomatStringView source, JsValueType type) {
    JsValue *value = eval_value(context, source);
    if (value == NULL) {
        return false;
    }
    bool valid = boa_JsValue_get_type(value) == type;
    boa_JsValue_destroy(value);
    return valid;
}

static bool has_number(BoaContext *context, const char *source, double expected) {
    JsValue *value = eval_value(context, view(source));
    if (value == NULL) {
        return false;
    }
    boa_JsValue_as_number_result number = boa_JsValue_as_number(value);
    boa_JsValue_destroy(value);
    return number.is_ok && number.ok == expected;
}

static bool throws(BoaContext *context, const char *source, JsValueType type) {
    JsError *error = eval_error(context, view(source));
    if (error == NULL) {
        return false;
    }
    JsValue *payload = boa_JsError_as_opaque(error);
    bool valid = boa_JsError_get_representation(error) == JsErrorRepresentation_Opaque &&
        !boa_JsError_as_native(error).is_ok &&
        payload != NULL && boa_JsValue_get_type(payload) == type;
    if (valid && type == JsValueType_Number) {
        boa_JsValue_as_number_result number = boa_JsValue_as_number(payload);
        valid = number.is_ok && number.ok == 7.0;
    }
    if (payload != NULL) {
        boa_JsValue_destroy(payload);
    }
    boa_JsError_destroy(error);
    return valid;
}

// `kind` is checked only for native errors.
static bool fails(BoaContext *context, const char *source,
                  JsErrorRepresentation representation, JsNativeErrorKind kind) {
    JsError *error = eval_error(context, view(source));
    if (error == NULL) {
        return false;
    }
    boa_JsError_as_native_result info = boa_JsError_as_native(error);
    JsValue *payload = boa_JsError_as_opaque(error);
    bool valid = boa_JsError_get_representation(error) == representation && payload == NULL &&
        (representation == JsErrorRepresentation_Native
            ? info.is_ok && info.ok.kind == kind
            : !info.is_ok);
    if (payload != NULL) {
        boa_JsValue_destroy(payload);
    }
    boa_JsError_destroy(error);
    return valid;
}

static bool results(BoaContext *context) {
    return has_type(context, view("undefined"), JsValueType_Undefined) &&
        throws(context, "throw 7", JsValueType_Number) &&
        throws(context, "throw new Error('boom')", JsValueType_Object) &&
        throws(context, "throw undefined", JsValueType_Undefined) &&
        throws(context, "throw null", JsValueType_Null) &&
        has_number(context, "1 + 2", 3.0) &&
        has_type(context, (DiplomatStringView){.data = NULL, .len = 0}, JsValueType_Undefined) &&
        fails(context, "1 +", JsErrorRepresentation_Native, JsNativeErrorKind_Syntax) &&
        fails(context, "missing", JsErrorRepresentation_Native, JsNativeErrorKind_Reference) &&
        fails(context, "null.x", JsErrorRepresentation_Native, JsNativeErrorKind_Type) &&
        throws(context, "try { null.x } catch (error) { throw error; }", JsValueType_Object);
}

static bool invalid_utf8(BoaContext *context) {
    const char *const sources[] = {
        "ran++; \xFF", "ran++; '\xF0\x9F\xA6'", "ran++; '\xED\xA0\x80'",
    };
    if (!has_type(context, view("var ran = 0;"), JsValueType_Undefined)) {
        return false;
    }
    for (size_t i = 0; i < sizeof(sources) / sizeof(sources[0]); ++i) {
        if (!fails(context, sources[i], JsErrorRepresentation_Native, JsNativeErrorKind_Type)) {
            return false;
        }
    }
    return has_number(context, "ran", 0.0);
}

static bool source_is_only_borrowed(BoaContext *context) {
    char source[] = "function seven() { return 3 + 4; } seven";
    JsValue *function = eval_value(context, view(source));
    memset(source, 'x', sizeof(source) - 1);
    if (function == NULL) {
        return false;
    }
    bool valid = boa_JsValue_get_type(function) == JsValueType_Object &&
        has_number(context, "seven()", 7.0);
    boa_JsValue_destroy(function);
    return valid;
}

static bool state_and_limits(BoaContext *context) {
    return has_type(context, view("var count = 1;"), JsValueType_Undefined) &&
        has_number(context, "count + 1", 2.0) &&
        throws(context, "count = 9; throw 7", JsValueType_Number) &&
        has_number(context, "count", 9.0) &&
        fails(context, "function recurse() { recurse(); } recurse()",
              JsErrorRepresentation_Engine, JsNativeErrorKind_Other) &&
        has_number(context, "1 + 2", 3.0);
}

static bool contexts_are_isolated(void) {
    BoaContext *first = boa_BoaContext_new();
    BoaContext *second = boa_BoaContext_new();
    bool valid = first != NULL && second != NULL &&
        has_type(first, view("var only_first = 1;"), JsValueType_Undefined) &&
        fails(second, "only_first", JsErrorRepresentation_Native, JsNativeErrorKind_Reference) &&
        has_number(first, "only_first", 1.0);
    if (first != NULL) {
        boa_BoaContext_destroy(first);
    }
    if (second != NULL) {
        boa_BoaContext_destroy(second);
    }
    return valid;
}

static bool converts(BoaContext *context, const char *source, JsNativeErrorKind kind,
                     const char *message, bool original_first) {
    JsError *error = eval_error(context, view(source));
    if (error == NULL) {
        return false;
    }
    boa_JsError_try_native_result converted = boa_JsError_try_native(error, context);
    if (!converted.is_ok || converted.ok == NULL) {
        boa_JsError_destroy(error);
        return false;
    }
    bool valid = boa_JsError_get_representation(error) == JsErrorRepresentation_Opaque;
    if (original_first) {
        boa_JsError_destroy(error);
    }

    boa_JsError_as_native_result info = boa_JsError_as_native(converted.ok);
    valid = valid &&
        boa_JsError_get_representation(converted.ok) == JsErrorRepresentation_Native &&
        info.is_ok && info.ok.kind == kind && info.ok.message.len == strlen(message) &&
        (info.ok.message.len == 0 ||
         memcmp(info.ok.message.data, message, info.ok.message.len) == 0);
    boa_JsError_destroy(converted.ok);
    if (!original_first) {
        valid = valid && boa_JsError_get_representation(error) == JsErrorRepresentation_Opaque;
        boa_JsError_destroy(error);
    }
    return valid;
}

static bool conversions(BoaContext *context) {
    return converts(context, "throw new Error('boom')", JsNativeErrorKind_Error, "boom", true) &&
        converts(context, "throw new Error('boom')", JsNativeErrorKind_Error, "boom", false) &&
        converts(context, "throw new TypeError('bad type')",
                 JsNativeErrorKind_Type, "bad type", false) &&
        converts(context, "class Sub extends Error {} throw new Sub('sub')",
                 JsNativeErrorKind_Error, "sub", false);
}

// A failed conversion must leave the error with its representation and payload.
static bool conversion_fails(BoaContext *context, const char *source,
                             JsErrorRepresentation representation, JsTryNativeError reason) {
    JsError *error = eval_error(context, view(source));
    if (error == NULL) {
        return false;
    }
    boa_JsError_try_native_result converted = boa_JsError_try_native(error, context);
    JsValue *payload = boa_JsError_as_opaque(error);
    bool valid = !converted.is_ok && converted.err == reason &&
        boa_JsError_get_representation(error) == representation &&
        (payload != NULL) == (representation == JsErrorRepresentation_Opaque);
    if (converted.is_ok && converted.ok != NULL) {
        boa_JsError_destroy(converted.ok);
    }
    if (payload != NULL) {
        boa_JsValue_destroy(payload);
    }
    boa_JsError_destroy(error);
    return valid;
}

static bool failed_conversions(BoaContext *context) {
    const JsErrorRepresentation opaque = JsErrorRepresentation_Opaque;
    return conversion_fails(context, "throw 7", opaque, JsTryNativeError_NotAnErrorObject) &&
        conversion_fails(context, "throw { message: 'x' }",
                         opaque, JsTryNativeError_NotAnErrorObject) &&
        conversion_fails(context, "var e = new Error(); e.message = 7; throw e",
                         opaque, JsTryNativeError_InvalidPropertyType) &&
        conversion_fails(context, "throw new Error('\\ud800')",
                         opaque, JsTryNativeError_InvalidMessageEncoding) &&
        conversion_fails(context, "var e = new Error(); e.constructor = 7; throw e",
                         opaque, JsTryNativeError_InvalidConstructor) &&
        conversion_fails(context,
                         "var e = new Error();"
                         "Object.defineProperty(e, 'message', { get() { throw 1; } });"
                         "throw e",
                         opaque, JsTryNativeError_InaccessibleProperty) &&
        conversion_fails(context,
                         "var e = new AggregateError([1]);"
                         "Object.defineProperty(e.errors, 0, { get() { throw 1; } });"
                         "throw e",
                         opaque, JsTryNativeError_InvalidErrorsIndex) &&
        conversion_fails(context,
                         "var revocable = Proxy.revocable(function () {}, {});"
                         "var e = new Error();"
                         "e.constructor = revocable.proxy;"
                         "revocable.revoke();"
                         "throw e",
                         opaque, JsTryNativeError_InaccessibleRealm) &&
        conversion_fails(context, "function recurse() { recurse(); } recurse()",
                         JsErrorRepresentation_Engine, JsTryNativeError_EngineError);
}

static bool destruction_order(bool context_first) {
    BoaContext *context = boa_BoaContext_new();
    if (context == NULL) {
        return false;
    }
    JsValue *value = eval_value(context, view("({ payload: 7 })"));
    JsError *error = eval_error(context, view("throw new Error('boom')"));
    JsValue *payload = error == NULL ? NULL : boa_JsError_as_opaque(error);
    JsError *converted = NULL;
    if (error != NULL) {
        boa_JsError_try_native_result result = boa_JsError_try_native(error, context);
        converted = result.is_ok ? result.ok : NULL;
    }
    if (context_first) {
        boa_BoaContext_destroy(context);
    }

    bool valid = value != NULL && error != NULL && payload != NULL && converted != NULL &&
        boa_JsValue_get_type(value) == JsValueType_Object &&
        boa_JsError_get_representation(error) == JsErrorRepresentation_Opaque &&
        boa_JsValue_get_type(payload) == JsValueType_Object &&
        boa_JsError_get_representation(converted) == JsErrorRepresentation_Native;
    if (converted != NULL) {
        boa_JsError_destroy(converted);
    }
    if (value != NULL) {
        boa_JsValue_destroy(value);
    }
    if (error != NULL) {
        boa_JsError_destroy(error);
    }
    if (payload != NULL) {
        boa_JsValue_destroy(payload);
    }
    if (!context_first) {
        boa_BoaContext_destroy(context);
    }
    return valid;
}

int main(void) {
    BoaContext *context = boa_BoaContext_new();
    if (context == NULL) {
        fputs("context failed to initialize\n", stderr);
        return EXIT_FAILURE;
    }
    const char *failure = NULL;
    if (!results(context)) {
        failure = "eval results were not distinguished";
    } else if (!invalid_utf8(context)) {
        failure = "invalid UTF-8 source was not rejected before running";
    } else if (!source_is_only_borrowed(context)) {
        failure = "eval results depended on the source buffer";
    } else if (!state_and_limits(context)) {
        failure = "eval state or engine limit handling failed";
    } else if (!conversions(context)) {
        failure = "thrown Error objects were not converted to native errors";
    } else if (!failed_conversions(context)) {
        failure = "failed conversions did not report their reason";
    }
    boa_BoaContext_destroy(context);

    if (failure == NULL && !contexts_are_isolated()) {
        failure = "contexts shared globals";
    }
    if (failure == NULL && (!destruction_order(true) || !destruction_order(false))) {
        failure = "handles did not survive either destruction order";
    }
    if (failure != NULL) {
        fprintf(stderr, "%s\n", failure);
        return EXIT_FAILURE;
    }
    puts("C eval tests passed.");
    return EXIT_SUCCESS;
}
