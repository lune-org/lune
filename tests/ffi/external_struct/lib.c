typedef struct {
    int a;
    int* b;
} ArgStruct;

typedef struct {
    int sum;
    int mul;
} ResultStruct;

ResultStruct ab(ArgStruct t) {
    ResultStruct result = { t.a+ * t.b, t.a * (*t.b) };
    return result;
}
