local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_pointer"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_pointerWrite()
	local pointerWrite = c.fn({ c.int:ptr() }, c.void):callable(lib:find("pointer_write"))

	local aBox = ffi.box(c.int.size)
	pointerWrite(nil, aBox:ref():ref())

	local result = c.int:readData(aBox)

	assert(result == 123, `pointer failed. result expected 123, got {result}`)
end

test_pointerWrite()
