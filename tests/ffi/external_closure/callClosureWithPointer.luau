local callableWrapper = require("../utils/callableWrapper")
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_closure/lib.c")
local c = ffi.c

-- Create closure
local closureWithPointerInfo = c.fn({ c.int, c.int:ptr() }, c.int)
local closureWithPointer = closureWithPointerInfo:closure(function(returnRef, aRef, bRef)
	c.int:writeData(returnRef, c.int:readData(aRef) + c.int:readData(bRef:deref()))
end)

local callClosureWithPointer =
	callableWrapper(lib:find("call_closure_with_pointer"), { closureWithPointerInfo }, c.int)
local result = callClosureWithPointer(closureWithPointer:ref())
assert(result == 72, `closureWithPointer failed. result expected 20000, got {result}`)
