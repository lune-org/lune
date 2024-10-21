local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_pointer"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_pointer_write()
	local pointer_write_info = c.fn({ c.int:ptr() }, c.void)

	local pointer_write_callable = pointer_write_info:callable(lib:find("pointer_write"))

	local a = ffi.box(c.int.size)

	pointer_write_callable(nil, a:ref():ref())

	local result = c.int:readData(a)

	assert(result == 123, `pointer failed. result expected 123, got {result}`)
end

test_pointer_write()
