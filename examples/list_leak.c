/* Unbounded linked-list leak — a common student bug: a list that grows forever
 * with no exit condition and no free(). Each ~4 KB node is touched so RSS grows.
 * Best run on Linux (the setrlimit wall is Linux-only). With mknight's monitor,
 * it should be killed at --max-ram with a post-mortem, not hang at the wall.
 * Build: gcc examples/list_leak.c -o examples/list_leak */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct node {
    int value;
    char payload[4096];
    struct node *next;
};

int main(void) {
    struct node *head = NULL;
    unsigned long count = 0;

    for (;;) {
        struct node *n = malloc(sizeof(struct node));
        if (n == NULL) {
            fprintf(stderr,
                    "malloc failed after %lu nodes - did you forget to free()?\n",
                    count);
            return 1;
        }
        n->value = (int)count;
        memset(n->payload, 1, sizeof(n->payload)); /* touch the pages */
        n->next = head;
        head = n;
        count++;

        if (count % 10000 == 0) {
            printf("list has %lu nodes\n", count);
            fflush(stdout);
        }
    }

    return 0; /* never reached */
}
