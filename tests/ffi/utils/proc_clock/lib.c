#include <time.h>

#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

EXPORT clock_t get_clock() {
    return clock();
}

EXPORT int sizeof_clock() {
    return sizeof(clock_t);
}

EXPORT double get_offset(clock_t before, clock_t after) {
    return (double)(after - before) / CLOCKS_PER_SEC;
}
