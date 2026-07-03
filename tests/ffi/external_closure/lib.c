#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

typedef int (*lua_closure_t)(int, int);
EXPORT int call_closure(lua_closure_t lua_closure) {
    return lua_closure(12, 24) * 2;
}

typedef void (*lua_hello_world_t)();
EXPORT void call_hello_world(lua_hello_world_t lua_closure) {
    lua_closure();
}

typedef int (*lua_closure_with_pointer_t)(int, int*);
EXPORT int call_closure_with_pointer(lua_closure_with_pointer_t lua_closure) {
    int b = 24;
    return lua_closure(12, &b) * 2;
}
