local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_math"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_add_int()
	local add_int_info = c.fn({ c.int, c.int }, c.int)

	local add_int_callable = add_int_info:callable(lib:find("add_int"))

	local result_box = ffi.box(c.int.size)
	local arg1 = c.int:box(100)
	local arg2 = c.int:box(200)

	add_int_callable(result_box, arg1:ref(), arg2:ref())
	local result = c.int:readData(result_box)

	assert(result == 300, `add_int failed. result expected 300, got {result}`)
end

test_add_int()

local function test_mul_int()
	local mul_int = c.fn({ c.int, c.int }, c.int)

	local mul_int_caller = mul_int:callable(lib:find("mul_int"))

	local resultBox = ffi.box(c.int.size)
	local arg1 = c.int:box(100)
	local arg2 = c.int:box(200)

	mul_int_caller(resultBox, arg1:ref(), arg2:ref())
	local result = c.int:readData(resultBox)

	assert(result == 20000, `mul_int failed. result expected 20000, got {result}`)
end

test_mul_int()
