--!nocheck
--!nolint
-- A Lua closure that errors must not unwind across the C caller and abort the
-- process. The callback catches it, leaves a zeroed result, and returns.
-- Regression guard: the old code unwrapped the error and panicked.
local callableWrapper = require("../utils/callableWrapper")
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_closure/lib.c")
local c = ffi.c

local erroring = c.fn({ c.int, c.int }, c.int):closure(function()
	error("intentional error from a closure")
end)

-- call_closure returns lua_closure(12, 24) * 2; a zeroed result means 0 * 2
local callClosure = callableWrapper(lib:find("call_closure"), { c.void:ptr() }, c.int)
local result = callClosure(erroring:ref())
assert(result == 0, `an erroring closure should yield a zeroed result, got {result}`)

-- A closure that writes only part of the result still cannot corrupt the host
local partial = c.fn({ c.int, c.int }, c.int):closure(function(ret)
	c.int:writeData(ret, 21)
end)
local resultTwo = callClosure(partial:ref())
assert(resultTwo == 42, `a normal closure after an error should still work, got {resultTwo}`)

print("external_closure/errors: an erroring closure does not crash the host")
