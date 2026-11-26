#ifndef JsValue_H
#define JsValue_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "JsStringError.d.h"
#include "JsValueType.d.h"

#include "JsValue.d.h"






JsValue* boa_JsValue_from_boolean(bool value);

JsValue* boa_JsValue_from_number(double number);

JsValue* boa_JsValue_from_string(DiplomatStringView value);

JsValue* boa_JsValue_from_null(void);

JsValue* boa_JsValue_from_undefined(void);

JsValueType boa_JsValue_get_type(const JsValue* self);

typedef struct boa_JsValue_as_boolean_result {union {bool ok; }; bool is_ok;} boa_JsValue_as_boolean_result;
boa_JsValue_as_boolean_result boa_JsValue_as_boolean(const JsValue* self);

typedef struct boa_JsValue_as_number_result {union {double ok; }; bool is_ok;} boa_JsValue_as_number_result;
boa_JsValue_as_number_result boa_JsValue_as_number(const JsValue* self);

typedef struct boa_JsValue_as_string_result {union { JsStringError err;}; bool is_ok;} boa_JsValue_as_string_result;
boa_JsValue_as_string_result boa_JsValue_as_string(const JsValue* self, DiplomatWrite* write);

typedef struct boa_JsValue_as_string_lossy_result { bool is_ok;} boa_JsValue_as_string_lossy_result;
boa_JsValue_as_string_lossy_result boa_JsValue_as_string_lossy(const JsValue* self, DiplomatWrite* write);

void boa_JsValue_destroy(JsValue* self);





#endif // JsValue_H
