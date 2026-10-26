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
        JsValue *value = JsValue_from_number(numbers[i]);
        for (size_t read = 0; read < 2; ++read) {
            JsValue_as_number_result result = JsValue_as_number(value);
            if (JsValue_get_type(value) != JsValueType_Number ||
                !result.is_ok || !same_number(result.ok, numbers[i])) {
                JsValue_destroy(value);
                fprintf(stderr, "number round trip failed at index %zu\n", i);
                return EXIT_FAILURE;
            }
        }
        JsValue_destroy(value);
    }

    BoaContext *context = BoaContext_new();
    if (context == NULL) {
        fputs("context failed to initialize\n", stderr);
        return EXIT_FAILURE;
    }
    JsValue *null_value = JsValue_from_null();
    JsValue *undefined_value = JsValue_from_undefined();
    JsValue *number = JsValue_from_number(42.5);
    BoaContext_destroy(context);

    JsValue_as_number_result result = JsValue_as_number(number);
    bool valid = JsValue_get_type(null_value) == JsValueType_Null &&
        JsValue_get_type(undefined_value) == JsValueType_Undefined &&
        !JsValue_as_number(null_value).is_ok &&
        !JsValue_as_number(undefined_value).is_ok &&
        JsValue_get_type(number) == JsValueType_Number &&
        result.is_ok && same_number(result.ok, 42.5);
    JsValue_destroy(null_value);
    JsValue_destroy(undefined_value);
    JsValue_destroy(number);
    if (!valid) {
        fputs("primitive values failed after context destruction\n", stderr);
        return EXIT_FAILURE;
    }

    puts("C value tests passed.");
    return EXIT_SUCCESS;
}
