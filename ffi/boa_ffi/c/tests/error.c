#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "JsError.h"
#include "JsValue.h"

static bool native_errors(void) {
    const JsNativeErrorKind kinds[] = {
        JsNativeErrorKind_Error, JsNativeErrorKind_Eval,
        JsNativeErrorKind_Range, JsNativeErrorKind_Reference,
        JsNativeErrorKind_Syntax, JsNativeErrorKind_Type,
        JsNativeErrorKind_Uri, JsNativeErrorKind_Aggregate,
    };
    const char expected[] = "bad \xF0\x9F\xA6\x80\0tail";
    const size_t lengths[] = {0, 3, sizeof(expected) - 1};

    for (size_t i = 0; i < sizeof(kinds) / sizeof(kinds[0]); ++i) {
        for (size_t j = 0; j < sizeof(lengths) / sizeof(lengths[0]); ++j) {
            char message[sizeof(expected)];
            memcpy(message, expected, sizeof(message));
            JsError *error = JsError_from_native(kinds[i],
                (DiplomatStringView){.data = message, .len = lengths[j]});
            if (error == NULL) {
                return false;
            }
            memset(message, 'x', sizeof(message));

            JsError_as_native_result info = JsError_as_native(error);
            JsValue *payload = JsError_as_opaque(error);
            bool valid = JsError_get_representation(error) == JsErrorRepresentation_Native &&
                info.is_ok && info.ok.kind == kinds[i] &&
                info.ok.message.len == lengths[j] &&
                (lengths[j] == 0 ||
                 memcmp(info.ok.message.data, expected, lengths[j]) == 0) &&
                payload == NULL;
            if (payload != NULL) {
                JsValue_destroy(payload);
            }
            JsError_destroy(error);
            if (!valid) {
                return false;
            }
        }
    }

    JsError *empty = JsError_from_native(JsNativeErrorKind_Error,
        (DiplomatStringView){.data = NULL, .len = 0});
    if (empty == NULL) {
        return false;
    }
    JsError_as_native_result empty_info = JsError_as_native(empty);
    bool empty_valid = empty_info.is_ok &&
        empty_info.ok.kind == JsNativeErrorKind_Error &&
        empty_info.ok.message.len == 0;
    JsError_destroy(empty);
    if (!empty_valid) {
        return false;
    }

    const DiplomatStringView rejected_messages[] = {
        {.data = "\xFF", .len = 1},
        {.data = expected, .len = 7},
        {.data = "\xED\xA0\x80", .len = 3},
    };
    for (size_t i = 0; i < sizeof(rejected_messages) / sizeof(rejected_messages[0]); ++i) {
        JsError *invalid = JsError_from_native(JsNativeErrorKind_Type, rejected_messages[i]);
        if (invalid != NULL) {
            JsError_destroy(invalid);
            return false;
        }
    }

    JsError *other = JsError_from_native(JsNativeErrorKind_Other,
        (DiplomatStringView){.data = "", .len = 0});
    if (other != NULL) {
        JsError_destroy(other);
        return false;
    }
    return true;
}

static bool payload_matches(JsValue *value, JsValueType kind) {
    if (value == NULL || JsValue_get_type(value) != kind) {
        return false;
    }
    JsValue_as_number_result number = JsValue_as_number(value);
    return kind == JsValueType_Number
        ? number.is_ok && number.ok == 7.0
        : !number.is_ok;
}

static bool opaque_error(JsValue *input, JsValueType kind, bool error_first) {
    JsError *error = JsError_from_opaque(input);
    JsValue_destroy(input);

    bool valid = JsError_get_representation(error) == JsErrorRepresentation_Opaque &&
        !JsError_as_native(error).is_ok;
    JsValue *first = JsError_as_opaque(error);
    JsValue *second = JsError_as_opaque(error);
    if (error_first) {
        JsError_destroy(error);
    }

    valid = valid && payload_matches(first, kind) && payload_matches(second, kind);
    if (first != NULL) {
        JsValue_destroy(first);
    }
    valid = valid && payload_matches(second, kind);
    if (second != NULL) {
        JsValue_destroy(second);
    }
    if (!error_first) {
        valid = valid && JsError_get_representation(error) == JsErrorRepresentation_Opaque;
        JsError_destroy(error);
    }
    return valid;
}

int main(void) {
    if (!native_errors()) {
        fputs("native error details or message ownership failed\n", stderr);
        return EXIT_FAILURE;
    }
    for (size_t order = 0; order < 2; ++order) {
        bool error_first = order == 0;
        if (!opaque_error(JsValue_from_number(7.0), JsValueType_Number, error_first) ||
            !opaque_error(JsValue_from_null(), JsValueType_Null, error_first) ||
            !opaque_error(JsValue_from_undefined(), JsValueType_Undefined, error_first)) {
            fputs("opaque error payload ownership failed\n", stderr);
            return EXIT_FAILURE;
        }
    }
    puts("C error tests passed.");
    return EXIT_SUCCESS;
}
