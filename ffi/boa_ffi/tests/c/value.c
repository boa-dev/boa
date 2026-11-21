#include <float.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

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
                !result.is_ok || !same_number(result.ok, numbers[i])) {
                boa_JsValue_destroy(value);
                fprintf(stderr, "number round trip failed at index %zu\n", i);
                return EXIT_FAILURE;
            }
        }
        boa_JsValue_destroy(value);
    }

    BoaContext *context = boa_BoaContext_new();
    if (context == NULL) {
        fputs("context failed to initialize\n", stderr);
        return EXIT_FAILURE;
    }
    JsValue *null_value = boa_JsValue_from_null();
    JsValue *undefined_value = boa_JsValue_from_undefined();
    JsValue *number = boa_JsValue_from_number(42.5);
    boa_BoaContext_destroy(context);

    boa_JsValue_as_number_result result = boa_JsValue_as_number(number);
    bool valid = boa_JsValue_get_type(null_value) == JsValueType_Null &&
        boa_JsValue_get_type(undefined_value) == JsValueType_Undefined &&
        !boa_JsValue_as_number(null_value).is_ok &&
        !boa_JsValue_as_number(undefined_value).is_ok &&
        boa_JsValue_get_type(number) == JsValueType_Number &&
        result.is_ok && same_number(result.ok, 42.5);
    boa_JsValue_destroy(null_value);
    boa_JsValue_destroy(undefined_value);
    boa_JsValue_destroy(number);
    if (!valid) {
        fputs("primitive values failed after context destruction\n", stderr);
        return EXIT_FAILURE;
    }

    puts("C value tests passed.");
    return EXIT_SUCCESS;
}
