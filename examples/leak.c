/*
 * leak.c — a deliberate memory leaker for testing MemoryKnight.
 *
 * It allocates 10 MB per iteration in an unbounded loop and *touches* every
 * page (via memset) so that resident memory (RSS) actually grows — not just
 * virtual memory. This exercises both protection layers:
 *
 *   - Layer 1 (Linux setrlimit wall): malloc eventually returns NULL, and this
 *     program prints a clean message and exits 1 — exactly the lesson students
 *     should learn (always check malloc's return value).
 *   - Layer 2 (monitor): mknight detects the runaway growth and kills it.
 *
 * Build:  gcc examples/leak.c -o examples/leak
 * Run:    mknight run --max-ram 200MB examples/leak
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    const size_t CHUNK = 10UL * 1024 * 1024; /* 10 MB */
    size_t total_mb = 0;

    for (;;) {
        char *p = malloc(CHUNK);
        if (p == NULL) {
            fprintf(stderr,
                    "malloc failed after %zu MB — did you forget to free()?\n",
                    total_mb);
            return 1;
        }
        memset(p, 1, CHUNK); /* touch the pages so RSS grows */
        total_mb += CHUNK / (1024 * 1024);
        printf("allocated %zu MB\n", total_mb);
        fflush(stdout);
    }

    return 0; /* never reached */
}
