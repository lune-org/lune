local callableWrapper = require("../utils/callableWrapper")
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_closure/lib.c")
local c = ffi.c

-- Create closure
local closureInfo = c.fn({ c.int, c.int }, c.int)
local closure = closureInfo:closure(function(ret, a, b)
	c.int:writeData(ret, c.int:readData(a) + c.int:readData(b))
end)

local callClosure = callableWrapper(lib:find("call_closure"), { closureInfo }, c.int)
local result = callClosure(closure:ref())
assert(result == 72, `callClosure failed. result expected 20000, got {result}`)
