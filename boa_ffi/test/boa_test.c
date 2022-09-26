#include <stdio.h>

#include "boa.h"

int main(void)
{
    char* return_value = NULL;
    return_value = boa_exec("console.log('hello from C from Rust from JavaScript!');");

    printf("Got back: \"%s\"\n", return_value);

    boa_free_string(return_value);
    printf("All freed!\n");

    return 0;
}
