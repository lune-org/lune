local ffi = require("@lune/ffi")

local int = 0b1
local float = 0.5

assert(ffi.isInteger(int) == true, "ffi.isInteger(int) == true assersion failed")
assert(ffi.isInteger(float) == false, "ffi.isInteger(float) == false assersion failed")
