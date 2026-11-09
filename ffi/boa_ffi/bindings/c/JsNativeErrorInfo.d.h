#ifndef JsNativeErrorInfo_D_H
#define JsNativeErrorInfo_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "JsNativeErrorKind.d.h"




typedef struct JsNativeErrorInfo {
  JsNativeErrorKind kind;
  DiplomatStringView message;
} JsNativeErrorInfo;

typedef struct JsNativeErrorInfo_option {union { JsNativeErrorInfo ok; }; bool is_ok; } JsNativeErrorInfo_option;



#endif // JsNativeErrorInfo_D_H
