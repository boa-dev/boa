#ifndef JsError_H
#define JsError_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "JsErrorRepresentation.d.h"
#include "JsNativeErrorInfo.d.h"
#include "JsNativeErrorKind.d.h"
#include "JsValue.d.h"

#include "JsError.d.h"






JsError* JsError_from_native(JsNativeErrorKind kind, DiplomatStringView message);

JsError* JsError_from_opaque(const JsValue* value);

JsErrorRepresentation JsError_get_representation(const JsError* self);

typedef struct JsError_as_native_result {union {JsNativeErrorInfo ok; }; bool is_ok;} JsError_as_native_result;
JsError_as_native_result JsError_as_native(const JsError* self);

JsValue* JsError_as_opaque(const JsError* self);

void JsError_destroy(JsError* self);





#endif // JsError_H
