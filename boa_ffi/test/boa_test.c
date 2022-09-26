#include "boa.h"

int main(void)
{
    boa_exec("console.log('hello from C from Rust from JavaScript!');");
    return 0;
}
