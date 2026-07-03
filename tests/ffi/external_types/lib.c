#include <stdarg.h>
#include <stdint.h>

#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

// Return values of every common integer width. libffi widens any integral
// return narrower than a machine word, so these exercise the result-buffer
// handling that must not overflow the caller's result storage.
EXPORT int8_t ret_i8(void) { return -8; }
EXPORT uint8_t ret_u8(void) { return 200; }
EXPORT int16_t ret_i16(void) { return -1600; }
EXPORT uint16_t ret_u16(void) { return 60000; }
EXPORT int32_t ret_i32(void) { return -32000000; }
EXPORT uint32_t ret_u32(void) { return 4000000000u; }
EXPORT int64_t ret_i64(void) { return -9000000000LL; }
EXPORT uint64_t ret_u64(void) { return 18000000000ULL; }

// Multiple arguments, exercising the sized-caller fast path.
EXPORT int sum4(int a, int b, int c, int d) { return a + b + c + d; }

// A byte the caller can use as an out-parameter through a pointer argument.
EXPORT void write_byte(uint8_t *out, uint8_t value) { *out = value; }

// Takes a fixed-size array wrapped in a struct, passed by value. This matches
// the ABI of an ffi array argument (a struct of N elements).
typedef struct {
    int v[3];
} Triple;
EXPORT int sum_triple(Triple t) { return t.v[0] + t.v[1] + t.v[2]; }

// Variadic functions. The first argument says how many follow.
EXPORT int sum_variadic(int count, ...) {
    va_list args;
    va_start(args, count);
    int total = 0;
    for (int i = 0; i < count; i++) {
        total += va_arg(args, int);
    }
    va_end(args);
    return total;
}

EXPORT double sum_variadic_double(int count, ...) {
    va_list args;
    va_start(args, count);
    double total = 0;
    for (int i = 0; i < count; i++) {
        total += va_arg(args, double);
    }
    va_end(args);
    return total;
}
