#include <stdio.h>
#include <string.h>

#include "../boa.h"

int main(void)
{
    char* return_value = NULL;
    char buffer[4096] = {0};
    size_t bytes_read = 0;

    // Read up to sizeof(buffer) from stdin
    bytes_read = fread(buffer, 1, sizeof(buffer), stdin);

    // Bail on error
    if (ferror(stdin))
    {
        perror("boa_test");
        return 1;
    }

    // If it's more than our buffer size, sorry
    if (bytes_read >= sizeof(buffer))
    {
        return 2;
    }

    // Get a zero-terminated result
    return_value = boa_exec(buffer);

    // Put it on stdout
    fwrite(return_value, 1, strlen(return_value), stdout);

    // Free the result
    boa_free_string(return_value);

    return 0;
}
