local ffi = require("@lune/ffi")
local c = ffi.c

assert(typeof(c.int) :: string == "CTypeInfo")
assert(tostring(c.int) == "int")
assert(tostring(ffi.i32) == "i32")
