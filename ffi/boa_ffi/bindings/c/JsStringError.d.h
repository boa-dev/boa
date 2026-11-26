#ifndef JsStringError_D_H
#define JsStringError_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum JsStringError {
  JsStringError_NotAString = 0,
  JsStringError_InvalidEncoding = 1,
} JsStringError;

typedef struct JsStringError_option {union { JsStringError ok; }; bool is_ok; } JsStringError_option;



#endif // JsStringError_D_H
