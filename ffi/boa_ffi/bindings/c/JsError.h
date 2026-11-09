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






JsError* boa_JsError_from_native(JsNativeErrorKind kind, DiplomatStringView message);

JsError* boa_JsError_from_opaque(const JsValue* value);

JsErrorRepresentation boa_JsError_get_representation(const JsError* self);

typedef struct boa_JsError_as_native_result {union {JsNativeErrorInfo ok; }; bool is_ok;} boa_JsError_as_native_result;
boa_JsError_as_native_result boa_JsError_as_native(const JsError* self);

JsValue* boa_JsError_as_opaque(const JsError* self);

void boa_JsError_destroy(JsError* self);





#endif // JsError_H
