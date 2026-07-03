local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_pointer/lib.c")
local c = ffi.c

local pointerWrite = c.fn({ c.int:ptr() }, c.void):callable(lib:find("pointer_write"))
local aBox = ffi.box(c.int.size)
pointerWrite(nil, aBox:ref():ref())
local result = c.int:readData(aBox)
assert(result == 123, `pointerWrite failed. result expected 123, got {result}`)
