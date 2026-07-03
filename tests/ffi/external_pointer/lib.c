#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

EXPORT void pointer_write(int *a) {
    *a = 123;
}

EXPORT int pointer_read(int *a) {
    return *a;
}
