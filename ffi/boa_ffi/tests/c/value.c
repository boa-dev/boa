#include <float.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "BoaContext.h"
#include "JsValue.h"

static bool same_number(double actual, double expected) {
    if (isnan(expected)) {
        return isnan(actual);
    }
    if (actual != expected) {
        return false;
    }
    return expected != 0.0 || (signbit(actual) != 0) == (signbit(expected) != 0);
}

static bool string_matches(const JsValue *value, const char *expected, size_t expected_len) {
    DiplomatWrite *strict = diplomat_buffer_write_create(0);
    boa_JsValue_as_string_result strict_result = boa_JsValue_as_string(value, strict);
    const char *strict_bytes = diplomat_buffer_write_get_bytes(strict);
    size_t strict_len = diplomat_buffer_write_len(strict);
    bool strict_ok = strict_result.is_ok && strict_len == expected_len &&
        (expected_len == 0 ||
         (strict_bytes != NULL && memcmp(strict_bytes, expected, expected_len) == 0));
    diplomat_buffer_write_destroy(strict);

    DiplomatWrite *lossy = diplomat_buffer_write_create(0);
    boa_JsValue_as_string_lossy_result lossy_result = boa_JsValue_as_string_lossy(value, lossy);
    const char *lossy_bytes = diplomat_buffer_write_get_bytes(lossy);
    size_t lossy_len = diplomat_buffer_write_len(lossy);
    bool lossy_ok = lossy_result.is_ok && lossy_len == expected_len &&
        (expected_len == 0 ||
         (lossy_bytes != NULL && memcmp(lossy_bytes, expected, expected_len) == 0));
    diplomat_buffer_write_destroy(lossy);

    return strict_ok && lossy_ok;
}

static bool not_a_string(const JsValue *value) {
    DiplomatWrite *strict = diplomat_buffer_write_create(0);
    boa_JsValue_as_string_result strict_result = boa_JsValue_as_string(value, strict);
    bool strict_ok = !strict_result.is_ok &&
        strict_result.err == JsStringError_NotAString &&
        diplomat_buffer_write_len(strict) == 0;
    diplomat_buffer_write_destroy(strict);

    DiplomatWrite *lossy = diplomat_buffer_write_create(0);
    boa_JsValue_as_string_lossy_result lossy_result = boa_JsValue_as_string_lossy(value, lossy);
    bool lossy_ok = !lossy_result.is_ok && diplomat_buffer_write_len(lossy) == 0;
    diplomat_buffer_write_destroy(lossy);

    return strict_ok && lossy_ok;
}

static bool booleans(void) {
    const bool values[] = {false, true};
    for (size_t i = 0; i < sizeof(values) / sizeof(values[0]); ++i) {
        JsValue *value = boa_JsValue_from_boolean(values[i]);
        for (size_t read = 0; read < 2; ++read) {
            boa_JsValue_as_boolean_result result = boa_JsValue_as_boolean(value);
            if (boa_JsValue_get_type(value) != JsValueType_Boolean ||
                !result.is_ok || result.ok != values[i] ||
                boa_JsValue_as_number(value).is_ok || !not_a_string(value)) {
                boa_JsValue_destroy(value);
                return false;
            }
        }
        boa_JsValue_destroy(value);
    }

    JsValue *non_booleans[] = {
        boa_JsValue_from_number(0.0),
        boa_JsValue_from_number(1.0),
        boa_JsValue_from_string((DiplomatStringView){.data = "", .len = 0}),
        boa_JsValue_from_string((DiplomatStringView){.data = "true", .len = 4}),
    };
    bool valid = true;
    for (size_t i = 0; i < sizeof(non_booleans) / sizeof(non_booleans[0]); ++i) {
        if (non_booleans[i] == NULL || boa_JsValue_as_boolean(non_booleans[i]).is_ok) {
            valid = false;
        }
        if (non_booleans[i] != NULL) {
            boa_JsValue_destroy(non_booleans[i]);
        }
    }
    return valid;
}

static bool strings(void) {
    const char nul_sample[] = "before\0after";
    const DiplomatStringView samples[] = {
        {.data = "", .len = 0},
        {.data = "hello", .len = 5},
        {.data = "latin1 \xC3\xA9", .len = 9},
        {.data = "crab \xF0\x9F\xA6\x80", .len = 9},
        {.data = nul_sample, .len = sizeof(nul_sample) - 1},
    };

    for (size_t i = 0; i < sizeof(samples) / sizeof(samples[0]); ++i) {
        char copy[32];
        if (samples[i].len > 0) {
            memcpy(copy, samples[i].data, samples[i].len);
        }
        JsValue *value = boa_JsValue_from_string(
            (DiplomatStringView){.data = copy, .len = samples[i].len});
        if (value == NULL) {
            return false;
        }
        memset(copy, 'x', sizeof(copy));

        for (size_t read = 0; read < 2; ++read) {
            if (boa_JsValue_get_type(value) != JsValueType_String ||
                boa_JsValue_as_boolean(value).is_ok ||
                boa_JsValue_as_number(value).is_ok ||
                !string_matches(value, samples[i].data, samples[i].len)) {
                boa_JsValue_destroy(value);
                return false;
            }
        }
        boa_JsValue_destroy(value);
    }

    // Empty string with a NULL data pointer.
    JsValue *empty = boa_JsValue_from_string((DiplomatStringView){.data = NULL, .len = 0});
    if (empty == NULL) {
        return false;
    }
    bool empty_ok = boa_JsValue_get_type(empty) == JsValueType_String &&
        string_matches(empty, "", 0);
    boa_JsValue_destroy(empty);
    if (!empty_ok) {
        return false;
    }

    // Fixed stack buffer: exact fit and overflow handling, plus a shorter view.
    const char slice_buf[] = "hello!";
    JsValue *hello = boa_JsValue_from_string((DiplomatStringView){.data = slice_buf, .len = 5});
    if (hello == NULL) {
        return false;
    }
    char exact_buf[6];
    DiplomatWrite exact_write = diplomat_simple_write(exact_buf, sizeof(exact_buf));
    boa_JsValue_as_string_result exact_res = boa_JsValue_as_string(hello, &exact_write);

    char small_buf[4] = {'a', 'b', 'c', 'd'};
    DiplomatWrite small_write = diplomat_simple_write(small_buf, sizeof(small_buf));
    boa_JsValue_as_string_result small_res = boa_JsValue_as_string(hello, &small_write);

    bool simple_ok = exact_res.is_ok && !exact_write.grow_failed && exact_write.len == 5 &&
        memcmp(exact_buf, "hello\0", 6) == 0 &&
        small_res.is_ok && small_write.grow_failed && small_write.len == 0 &&
        small_buf[0] == '\0';
    boa_JsValue_destroy(hello);
    if (!simple_ok) {
        return false;
    }

    // Invalid UTF-8 must be rejected with NULL.
    const DiplomatStringView invalid[] = {
        {.data = "\xFF", .len = 1},
        {.data = "\xED\xA0\x80", .len = 3},
    };
    for (size_t i = 0; i < sizeof(invalid) / sizeof(invalid[0]); ++i) {
        JsValue *bad = boa_JsValue_from_string(invalid[i]);
        if (bad != NULL) {
            boa_JsValue_destroy(bad);
            return false;
        }
    }
    return true;
}

int main(void) {
    const double numbers[] = {
        0.0, -0.0, 1.5, -2.75,
        -2147483649.0, -2147483648.0, 2147483647.0, 2147483648.0,
        9007199254740991.0, NAN, INFINITY, -INFINITY,
        DBL_MIN, DBL_TRUE_MIN, DBL_MAX, -DBL_MAX,
    };

    for (size_t i = 0; i < sizeof(numbers) / sizeof(numbers[0]); ++i) {
        JsValue *value = boa_JsValue_from_number(numbers[i]);
        for (size_t read = 0; read < 2; ++read) {
            boa_JsValue_as_number_result result = boa_JsValue_as_number(value);
            if (boa_JsValue_get_type(value) != JsValueType_Number ||
                !result.is_ok || !same_number(result.ok, numbers[i]) ||
                boa_JsValue_as_boolean(value).is_ok || !not_a_string(value)) {
                boa_JsValue_destroy(value);
                fprintf(stderr, "number round trip failed at index %zu\n", i);
                return EXIT_FAILURE;
            }
        }
        boa_JsValue_destroy(value);
    }

    if (!booleans()) {
        fputs("boolean round trip or non-coercion check failed\n", stderr);
        return EXIT_FAILURE;
    }
    if (!strings()) {
        fputs("string round trip or UTF-8 validation failed\n", stderr);
        return EXIT_FAILURE;
    }

    BoaContext *context = boa_BoaContext_new();
    if (context == NULL) {
        fputs("context failed to initialize\n", stderr);
        return EXIT_FAILURE;
    }
    const char kept[] = "kept\0value";
    JsValue *null_value = boa_JsValue_from_null();
    JsValue *undefined_value = boa_JsValue_from_undefined();
    JsValue *boolean = boa_JsValue_from_boolean(true);
    JsValue *number = boa_JsValue_from_number(42.5);
    JsValue *string = boa_JsValue_from_string(
        (DiplomatStringView){.data = kept, .len = sizeof(kept) - 1});
    boa_BoaContext_destroy(context);

    boa_JsValue_as_boolean_result bool_result = boa_JsValue_as_boolean(boolean);
    boa_JsValue_as_number_result num_result = boa_JsValue_as_number(number);
    bool valid = boa_JsValue_get_type(null_value) == JsValueType_Null &&
        boa_JsValue_get_type(undefined_value) == JsValueType_Undefined &&
        !boa_JsValue_as_boolean(null_value).is_ok &&
        !boa_JsValue_as_boolean(undefined_value).is_ok &&
        !boa_JsValue_as_number(null_value).is_ok &&
        !boa_JsValue_as_number(undefined_value).is_ok &&
        not_a_string(null_value) && not_a_string(undefined_value) &&
        boa_JsValue_get_type(boolean) == JsValueType_Boolean &&
        bool_result.is_ok && bool_result.ok &&
        boa_JsValue_get_type(number) == JsValueType_Number &&
        num_result.is_ok && same_number(num_result.ok, 42.5) &&
        string != NULL && boa_JsValue_get_type(string) == JsValueType_String &&
        string_matches(string, kept, sizeof(kept) - 1);
    boa_JsValue_destroy(null_value);
    boa_JsValue_destroy(undefined_value);
    boa_JsValue_destroy(boolean);
    boa_JsValue_destroy(number);
    if (string != NULL) {
        boa_JsValue_destroy(string);
    }
    if (!valid) {
        fputs("primitive values failed after context destruction\n", stderr);
        return EXIT_FAILURE;
    }

    puts("C value tests passed.");
    return EXIT_SUCCESS;
}
