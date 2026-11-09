#ifndef BoaContext_H
#define BoaContext_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"


#include "BoaContext.d.h"






BoaContext* boa_BoaContext_new(void);

void boa_BoaContext_destroy(BoaContext* self);





#endif // BoaContext_H
