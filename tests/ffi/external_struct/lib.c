#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

typedef struct {
    int a;
    int* b;
} ArgStruct;

typedef struct {
    int sum;
    int mul;
} ResultStruct;

EXPORT ResultStruct ab(ArgStruct t) {
    ResultStruct result = { t.a+ * t.b, t.a * (*t.b) };
    return result;
}
