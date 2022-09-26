#include <stdio.h>

#include "boa.h"

int main(void)
{
    const char* return_value = NULL;
    return_value = boa_exec("console.log('hello from C from Rust from JavaScript!');");

    printf("Got back: \"%s\"\n", return_value);
    return 0;
}
