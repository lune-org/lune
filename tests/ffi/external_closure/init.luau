local ffi = require("@lune/ffi")

local testdir = "./tests/ffi/external_closure"

local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)

local lib = ffi.open(`{testdir}/lib.so`)

local function test_closure()
	local callback_info = ffi.c.fn({ ffi.c.int, ffi.c.int }, ffi.c.int)
	local callback_closure = callback_info:closure(function(ret, a, b)
		ffi.c.int:writeData(ret, ffi.c.int:readData(a) + ffi.c.int:readData(b))
	end)

	local closure_test_info = ffi.c.fn({ callback_info }, ffi.c.int)

	local closure_test_callable = closure_test_info:callable(lib:find("closure_test"))

	local result_box = ffi.box(ffi.c.int.size)
	closure_test_callable(result_box, callback_closure:ref())
end

test_closure()
