#include <time.h>

int add(int a, int b) {
    return a + b;
}

double c_test() {
    clock_t before = clock();

    int a = 0;
    for (int i=0; i<1000000; i++) {
        a = add(a, 1);
    }

    clock_t after = clock();

    return (double)(after - before) / CLOCKS_PER_SEC;
}
