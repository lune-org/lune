#include<stdio.h>

typedef int (*lua_callback_t)(int a, int b);

int closure_test(lua_callback_t callback) {
    printf("%p\n", callback);
    printf("%d\n", (*callback)(12, 24));

    return (*callback)(12, 24) * 2;
}

int closure(int a, int b) {
    return a+b;
}
