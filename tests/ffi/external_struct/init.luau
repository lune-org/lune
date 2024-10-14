local ffi = require("@lune/ffi")

local testdir = "./tests/ffi/external_struct"

local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)

local lib = ffi.open(`{testdir}/lib.so`)

local function test_AB()
	local ArgStruct = ffi.struct({ ffi.int, ffi.int:ptr() })
	local ResultStruct = ffi.struct({ ffi.int, ffi.int })

	local AB = ffi.fn({ ArgStruct }, ResultStruct)

	local AB_caller = AB:caller(lib:find("AB"))

	local resultBox = ffi.box(ffi.int.size)
	local a = ffi.int:box(100)
	local b = ffi.int:box(200)
	local arg = ArgStruct:box({ a, b:leak() })

	AB_caller:call(resultBox, arg)
	local result = ResultStruct:from(resultBox)

	assert(result[0] == 300, `AB failed. result expected 300, got {result}`)
	assert(result[1] == 20000, `AB failed. result expected 300, got {result}`)
end

test_AB()
