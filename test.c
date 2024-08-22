
#include <stdio.h>

typedef struct
{
    char t; // 1
    // .
    // .
    // .
    // .
    // .
    // .
    // .
    long long tt; // 8
    char ttt;
    char tttt;
    char ttttt;
    char tttttt;
    // .
    // .
    // .
    // .
} a; // 24 만약 가장 큰것의 정렬을 따른다면

int main()
{
    // 2400
    printf("%d\n", sizeof(a[100]));
}
