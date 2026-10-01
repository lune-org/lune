local callableWrapper = require("../utils/callableWrapper")
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_pointer/lib.c")
local c = ffi.c

local pointerRead = callableWrapper(lib:find("pointer_read"), { c.int:ptr() }, c.int)
local result = pointerRead(c.int:box(123):ref():ref())
assert(result == 123, `pointerRead failed. result expected 123, got {result}`)
