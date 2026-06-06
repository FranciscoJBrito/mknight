/*
 * grow.c — a *bounded* memory grower for safely testing MemoryKnight's monitor.
 *
 * Allocates up to N MB (default 200, or argv[1]) in 10 MB steps. Each step it
 * fills the new block with an incompressible pattern and re-touches every block
 * allocated so far, keeping the whole working set "hot" and resident — so RSS
 * tracks real usage even on macOS, whose compressor/pager would otherwise evict
 * cold, write-once pages and hide the growth. Holds for ~1s, then frees and
 * exits cleanly. Safe to run anywhere — unlike examples/leak.c it never runs away.
 *
 * Build:  gcc examples/grow.c -o examples/grow
 * Run:    mknight run examples/grow 200
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
    size_t target_mb = (argc > 1) ? strtoul(argv[1], NULL, 10) : 200;
    const size_t CHUNK = 10UL * 1024 * 1024; /* 10 MB */
    size_t n = target_mb / 10;

    char **blocks = malloc(n * sizeof(char *));
    if (blocks == NULL) {
        fprintf(stderr, "bookkeeping malloc failed\n");
        return 1;
    }

    for (size_t i = 0; i < n; i++) {
        blocks[i] = malloc(CHUNK);
        if (blocks[i] == NULL) {
            fprintf(stderr, "malloc failed at %zu MB\n", i * 10);
            return 1;
        }
        /* Fill with an incompressible xorshift pattern so the pages stay
         * resident (a constant memset would be compressed away by macOS). */
        unsigned int x = (unsigned int)(i * 2654435761u) | 1u;
        unsigned int *words = (unsigned int *)blocks[i];
        for (size_t w = 0; w < CHUNK / sizeof(unsigned int); w++) {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            words[w] = x;
        }
        /* Re-touch every block allocated so far (one byte per 4 KB page) so the
         * OS keeps them resident instead of paging them out. */
        for (size_t j = 0; j <= i; j++) {
            for (size_t off = 0; off < CHUNK; off += 4096) {
                blocks[j][off] ^= 0xAAu;
            }
        }

        printf("allocated %zu MB\n", (i + 1) * 10);
        fflush(stdout);
        usleep(100 * 1000); /* 100 ms between steps -> several samples each */
    }

    printf("holding %zu MB for ~1s\n", n * 10);
    fflush(stdout);
    sleep(1);

    for (size_t i = 0; i < n; i++) {
        free(blocks[i]);
    }
    free(blocks);
    return 0;
}
