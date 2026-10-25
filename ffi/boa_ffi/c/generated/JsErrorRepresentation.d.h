#ifndef JsErrorRepresentation_D_H
#define JsErrorRepresentation_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef enum JsErrorRepresentation {
  JsErrorRepresentation_Native = 0,
  JsErrorRepresentation_Opaque = 1,
  JsErrorRepresentation_Engine = 2,
} JsErrorRepresentation;

typedef struct JsErrorRepresentation_option {union { JsErrorRepresentation ok; }; bool is_ok; } JsErrorRepresentation_option;



#endif // JsErrorRepresentation_D_H
