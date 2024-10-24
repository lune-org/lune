local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_math"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_addInt()
	local addInt = c.fn({ c.int, c.int }, c.int):callable(lib:find("add_int"))

	local resultBox = ffi.box(c.int.size)
	local arg1 = c.int:box(100)
	local arg2 = c.int:box(200)

	addInt(resultBox, arg1:ref(), arg2:ref())
	local result = c.int:readData(resultBox)

	assert(result == 300, `test_addInt failed. result expected 300, got {result}`)
end

test_addInt()

local function test_mulInt()
	local mulInt = c.fn({ c.int, c.int }, c.int):callable(lib:find("mul_int"))

	local resultBox = ffi.box(c.int.size)
	local arg1 = c.int:box(100)
	local arg2 = c.int:box(200)

	mulInt(resultBox, arg1:ref(), arg2:ref())
	local result = c.int:readData(resultBox)

	assert(result == 20000, `test_mulInt failed. result expected 20000, got {result}`)
end

test_mulInt()
