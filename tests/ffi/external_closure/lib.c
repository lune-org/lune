#include<stdio.h>

typedef int (*lua_callback_t)(int, int);
typedef void (*lua_hello_world_callback_t)();

int closure(lua_callback_t lua_closure) {
    return lua_closure(12, 24) * 2;
}

void hello_world(lua_hello_world_callback_t lua_closure) {
    lua_closure();
}
