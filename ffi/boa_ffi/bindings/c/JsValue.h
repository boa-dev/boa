#ifndef JsValue_H
#define JsValue_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "JsValueType.d.h"

#include "JsValue.d.h"






JsValue* boa_JsValue_from_number(double number);

JsValue* boa_JsValue_from_null(void);

JsValue* boa_JsValue_from_undefined(void);

JsValueType boa_JsValue_get_type(const JsValue* self);

typedef struct boa_JsValue_as_number_result {union {double ok; }; bool is_ok;} boa_JsValue_as_number_result;
boa_JsValue_as_number_result boa_JsValue_as_number(const JsValue* self);

void boa_JsValue_destroy(JsValue* self);





#endif // JsValue_H
