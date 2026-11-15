/* Plan-34: the TSC rate (invariant TSC, nominal, not the core clock).
 * Three 1 s sleeps; prints "<hz>" per line. */
#include <stdio.h>
#include <time.h>
#include <x86intrin.h>

int main(void) {
    for (int i = 0; i < 3; i++) {
        struct timespec t0, t1, req = {1, 0};
        unsigned long long a = __rdtsc();
        clock_gettime(CLOCK_MONOTONIC, &t0);
        nanosleep(&req, 0);
        clock_gettime(CLOCK_MONOTONIC, &t1);
        unsigned long long b = __rdtsc();
        double s = (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec) / 1e9;
        printf("%.0f\n", (b - a) / s);
    }
    return 0;
}
