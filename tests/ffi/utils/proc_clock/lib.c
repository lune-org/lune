#include <time.h>
clock_t get_clock() {
    return clock();
}

int sizeof_clock() {
    return sizeof(clock_t);
}

double get_offset(clock_t before, clock_t after) {
    return (double)(after - before) / CLOCKS_PER_SEC;
}
