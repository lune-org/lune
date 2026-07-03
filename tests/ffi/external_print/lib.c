#include<stdio.h>

#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

EXPORT void hello_world() {
    printf("Hello world from external function!");
}
