// This is a string type that can be given to an external function.
// To be exact, it converts the Lua string into a c_char array and puts it in the box.
// For this part, initially, i wanted to allow box("lua string"),
// but separated it for clarity.
// This also allows operations such as ffi.string:intoBox().
// (Write a string to an already existing box)
