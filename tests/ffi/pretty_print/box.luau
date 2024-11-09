local ffi = require("@lune/ffi")
local half_back = 0b_0000_0000_0000_0000_0000_0000_1111_1111

print(ffi.i32:box(half_back))
