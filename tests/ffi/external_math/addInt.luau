local callableWrapper = require("../utils/callableWrapper")
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_math/lib.c")
local c = ffi.c

local addInt = callableWrapper(lib:find("add_int"), { c.int, c.int }, c.int)
local result = addInt(100, 200)

assert(result == 300, `test_addInt failed. result expected 300, got {result}`)
