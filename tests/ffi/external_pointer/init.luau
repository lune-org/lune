local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_pointer"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_pointer()
	local pointer_info = c.fn({ c.int:ptr() }, c.void)

	local pointer_callable = pointer_info:callable(lib:find("pointer"))

	local a = ffi.box(c.int.size)

	pointer_callable(nil, a:ref():ref())

	local result = c.int:readData(a)

	assert(result == 123, `pointer failed. result expected 123, got {result}`)
end

test_pointer()
