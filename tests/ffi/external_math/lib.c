#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

EXPORT int add_int(int a, int b) {
    return a + b;
}

EXPORT int mul_int(int a, int b) {
    return a * b;
}
