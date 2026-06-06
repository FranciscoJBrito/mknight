/* Bounded memory grower for testing MemoryKnight's monitor. Allocates up to
 * N MB (default 200, or argv[1]), keeps its working set hot/resident, holds for
 * ~1s, then frees and exits. Safe to run anywhere. Build: gcc -O2 examples/grow.c -o examples/grow */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
    size_t target_mb = (argc > 1) ? strtoul(argv[1], NULL, 10) : 200;
    const size_t CHUNK = 10UL * 1024 * 1024;
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
        unsigned int x = (unsigned int)(i * 2654435761u) | 1u;
        unsigned int *words = (unsigned int *)blocks[i];
        for (size_t w = 0; w < CHUNK / sizeof(unsigned int); w++) {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            words[w] = x;
        }
        for (size_t j = 0; j <= i; j++) {
            for (size_t off = 0; off < CHUNK; off += 4096) {
                blocks[j][off] ^= 0xAAu;
            }
        }

        printf("allocated %zu MB\n", (i + 1) * 10);
        fflush(stdout);
        usleep(100 * 1000);
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
