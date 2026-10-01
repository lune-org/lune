local ffi = require("@lune/ffi")
local c = ffi.c

assert(typeof(c.int:arr(5)) :: string == "CArrInfo")
assert(tostring(c.int:arr(5)) == " int, length = 5 ")
assert(tostring(c.int:ptr():arr(5)) == " <CPtrInfo(int)>, length = 5 ")
