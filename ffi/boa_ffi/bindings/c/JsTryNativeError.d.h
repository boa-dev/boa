#ifndef JsTryNativeError_D_H
#define JsTryNativeError_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum JsTryNativeError {
  JsTryNativeError_NotAnErrorObject = 0,
  JsTryNativeError_InvalidPropertyType = 1,
  JsTryNativeError_InvalidMessageEncoding = 2,
  JsTryNativeError_InvalidConstructor = 3,
  JsTryNativeError_InaccessibleProperty = 4,
  JsTryNativeError_InvalidErrorsIndex = 5,
  JsTryNativeError_InaccessibleRealm = 6,
  JsTryNativeError_EngineError = 7,
} JsTryNativeError;

typedef struct JsTryNativeError_option {union { JsTryNativeError ok; }; bool is_ok; } JsTryNativeError_option;



#endif // JsTryNativeError_D_H
