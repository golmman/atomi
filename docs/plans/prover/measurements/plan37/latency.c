/* Plan 37 — measured penalties for the class model (not part of the engine).
 *
 *   gcc -O2 -o /tmp/p37-lat latency.c && /tmp/p37-lat
 *
 * Pointer chase: dependent-load latency by working set (L1/L2/L3/DRAM).
 * Branch loop: predictable vs data-dependent branch; the delta divided by
 * the measured branch-misses gives the mispredict penalty.
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <x86intrin.h>

static uint64_t rdtsc(void) {
    unsigned lo, hi;
    __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
}

static void chase(size_t bytes, int steps) {
    size_t n = bytes / 64;
    size_t *idx = malloc(n * sizeof(size_t));
    char *buf = aligned_alloc(64, n * 64);
    memset(buf, 0, n * 64);
    for (size_t i = 0; i < n; i++) idx[i] = i;
    for (size_t i = n - 1; i > 0; i--) {
        size_t j = rand() % (i + 1);
        size_t t = idx[i]; idx[i] = idx[j]; idx[j] = t;
    }
    for (size_t i = 0; i < n; i++) *(size_t *)(buf + idx[i] * 64) = idx[(i + 1) % n];
    volatile size_t p = 0;
    for (int i = 0; i < steps / 10 + 10; i++) p = *(size_t *)(buf + p * 64);
    uint64_t t0 = rdtsc();
    for (int i = 0; i < steps; i++) p = *(size_t *)(buf + p * 64);
    uint64_t t1 = rdtsc();
    printf("chase %8zu KB: %.2f cycles/load (p=%zu)\n", bytes / 1024, (double)(t1 - t0) / steps, p);
    free(idx); free(buf);
}

int main(void) {
    srand(1);
    chase(16 * 1024, 2000000);
    chase(256 * 1024, 2000000);
    chase(4 * 1024 * 1024, 1000000);
    chase(64 * 1024 * 1024, 300000);

    int n = 200000000;
    unsigned *bits = malloc(n * sizeof(unsigned));
    for (int i = 0; i < n; i++) bits[i] = rand() & 1;
    volatile long s = 0;
    uint64_t t0 = rdtsc();
    for (int i = 0; i < n; i++) { if (1) s += i; }
    uint64_t t1 = rdtsc();
    uint64_t t2 = rdtsc();
    for (int i = 0; i < n; i++) { if (bits[i]) s += i; }
    uint64_t t3 = rdtsc();
    printf("branch predictable %.3f cycles/iter, unpredictable %.3f, delta %.3f\n",
           (double)(t1 - t0) / n, (double)(t3 - t2) / n,
           (double)((t3 - t2) - (t1 - t0)) / n);
    return 0;
}
