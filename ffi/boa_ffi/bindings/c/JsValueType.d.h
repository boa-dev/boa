#ifndef JsValueType_D_H
#define JsValueType_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum JsValueType {
  JsValueType_Undefined = 0,
  JsValueType_Null = 1,
  JsValueType_Boolean = 2,
  JsValueType_Number = 3,
  JsValueType_String = 4,
  JsValueType_Symbol = 5,
  JsValueType_BigInt = 6,
  JsValueType_Object = 7,
} JsValueType;

typedef struct JsValueType_option {union { JsValueType ok; }; bool is_ok; } JsValueType_option;



#endif // JsValueType_D_H
