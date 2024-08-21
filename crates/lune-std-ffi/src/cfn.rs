// cfn is a type declaration for a function.
// Basically, when calling an external function, this type declaration
// is referred to and type conversion is automatically assisted.
// However, in order to save on type conversion costs,
// users keep values ​​they will use continuously in a box and use them multiple times.
// Alternatively, if the types are the same,you can save the cost of creating
// a new space by directly passing FfiRaw,
// the result value of another function or the argument value of the callback.
// The name cfn is intentional. This is because any *c_void is
// moved to a Lua function or vice versa.
