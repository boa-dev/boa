#ifndef JsNativeErrorKind_D_H
#define JsNativeErrorKind_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum JsNativeErrorKind {
  JsNativeErrorKind_Error = 0,
  JsNativeErrorKind_Eval = 1,
  JsNativeErrorKind_Range = 2,
  JsNativeErrorKind_Reference = 3,
  JsNativeErrorKind_Syntax = 4,
  JsNativeErrorKind_Type = 5,
  JsNativeErrorKind_Uri = 6,
  JsNativeErrorKind_Aggregate = 7,
  JsNativeErrorKind_Other = 8,
} JsNativeErrorKind;

typedef struct JsNativeErrorKind_option {union { JsNativeErrorKind ok; }; bool is_ok; } JsNativeErrorKind_option;



#endif // JsNativeErrorKind_D_H
