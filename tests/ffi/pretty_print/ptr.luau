local ffi = require("@lune/ffi")
local c = ffi.c

assert(typeof(c.int:ptr()) :: string == "CPtrInfo")
assert(tostring(c.int:ptr()) == "int")
assert(tostring(c.int:arr(5):ptr()) == " <CArrInfo( int, length = 5 )> ")
