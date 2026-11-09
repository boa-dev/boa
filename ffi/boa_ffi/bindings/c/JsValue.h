#ifndef JsValue_H
#define JsValue_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "JsValueType.d.h"

#include "JsValue.d.h"






JsValue* JsValue_from_number(double number);

JsValue* JsValue_from_null(void);

JsValue* JsValue_from_undefined(void);

JsValueType JsValue_get_type(const JsValue* self);

typedef struct JsValue_as_number_result {union {double ok; }; bool is_ok;} JsValue_as_number_result;
JsValue_as_number_result JsValue_as_number(const JsValue* self);

void JsValue_destroy(JsValue* self);





#endif // JsValue_H
