#ifndef BoaContext_H
#define BoaContext_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "JsError.d.h"
#include "JsValue.d.h"

#include "BoaContext.d.h"






BoaContext* boa_BoaContext_new(void);

typedef struct boa_BoaContext_eval_result {union {JsValue* ok; JsError* err;}; bool is_ok;} boa_BoaContext_eval_result;
boa_BoaContext_eval_result boa_BoaContext_eval(BoaContext* self, DiplomatStringView source);

void boa_BoaContext_destroy(BoaContext* self);





#endif // BoaContext_H
