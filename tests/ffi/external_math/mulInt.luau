local callableWrapper = require("../utils/callableWrapper")
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_math/lib.c")
local c = ffi.c

local mulInt = callableWrapper(lib:find("mul_int"), { c.int, c.int }, c.int)
local result = mulInt(100, 200)
assert(result == 20000, `test_mulInt failed. result expected 20000, got {result}`)
