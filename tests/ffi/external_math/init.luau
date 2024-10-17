local ffi = require("@lune/ffi")

local testdir = "./tests/ffi/external_math"

local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)

local lib = ffi.open(`{testdir}/lib.so`)

local function test_add_int()
	local add_int = ffi.fnInfo({ ffi.int, ffi.int }, ffi.int)

	local add_int_caller = add_int:callable(lib:find("add_int"))

	local resultBox = ffi.box(ffi.int.size)
	local arg1 = ffi.int:box(100)
	local arg2 = ffi.int:box(200)

	add_int_caller:call(resultBox, arg1, arg2)
	local result = ffi.int:readData(resultBox)

	assert(result == 300, `add_int failed. result expected 300, got {result}`)
end

test_add_int()

local function test_mul_int()
	local mul_int = ffi.fnInfo({ ffi.int, ffi.int }, ffi.int)

	local mul_int_caller = mul_int:callable(lib:find("mul_int"))

	local resultBox = ffi.box(ffi.int.size)
	local arg1 = ffi.int:box(100)
	local arg2 = ffi.int:box(200)

	mul_int_caller:call(resultBox, arg1, arg2)
	local result = ffi.int:readData(resultBox)

	assert(result == 20000, `mul_int failed. result expected 20000, got {result}`)
end

test_mul_int()
