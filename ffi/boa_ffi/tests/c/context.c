#include <stdio.h>
#include <stdlib.h>

#include "BoaContext.h"

int main(void) {
    for (size_t i = 0; i < 32; ++i) {
        BoaContext *first = boa_BoaContext_new();
        if (first == NULL) {
            fputs("first context failed to initialize\n", stderr);
            return EXIT_FAILURE;
        }

        BoaContext *second = boa_BoaContext_new();
        if (second == NULL) {
            boa_BoaContext_destroy(first);
            fputs("second context failed to initialize\n", stderr);
            return EXIT_FAILURE;
        }

        boa_BoaContext_destroy(first);

        BoaContext *third = boa_BoaContext_new();
        if (third == NULL) {
            boa_BoaContext_destroy(second);
            fputs("third context failed to initialize\n", stderr);
            return EXIT_FAILURE;
        }

        boa_BoaContext_destroy(second);
        boa_BoaContext_destroy(third);
    }

    puts("C context lifecycle tests passed.");
    return EXIT_SUCCESS;
}
