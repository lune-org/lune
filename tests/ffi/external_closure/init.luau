local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_closure"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_closure()
	local callback_info = c.fn({ c.int, c.int }, c.int)
	local callback_closure = callback_info:closure(function(ret, a, b)
		c.int:writeData(ret, c.int:readData(a) + c.int:readData(b))
	end)

	local closure_test_info = c.fn({ callback_info }, c.int)

	local closure_test_callable = closure_test_info:callable(lib:find("closure"))

	local result_box = ffi.box(c.int.size)
	closure_test_callable(result_box, callback_closure:ref())
	local result = c.int:readData(result_box)
	assert(result == 72, `test_closure failed. result expected 20000, got {result}`)
end

test_closure()

local function test_hello_world()
	local callback_info = c.fn({}, c.void)
	local callback_closure = callback_info:closure(function()
		print("Hello world in lua closure!")
	end)

	local closure_test_info = c.fn({ callback_info }, c.void)

	local closure_test_callable = closure_test_info:callable(lib:find("hello_world"))

	closure_test_callable(nil, callback_closure:ref())
end

test_hello_world()
