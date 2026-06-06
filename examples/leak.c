/* Unbounded memory leaker for testing MemoryKnight. Best run on Linux, where
 * the wall makes malloc() return NULL cleanly. Build: gcc examples/leak.c -o examples/leak */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    const size_t CHUNK = 10UL * 1024 * 1024;
    size_t total_mb = 0;

    for (;;) {
        char *p = malloc(CHUNK);
        if (p == NULL) {
            fprintf(stderr,
                    "malloc failed after %zu MB — did you forget to free()?\n",
                    total_mb);
            return 1;
        }
        memset(p, 1, CHUNK);
        total_mb += CHUNK / (1024 * 1024);
        printf("allocated %zu MB\n", total_mb);
        fflush(stdout);
    }

    return 0;
}
