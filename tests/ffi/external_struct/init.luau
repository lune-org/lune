local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_struct"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_AB()
	local ArgStruct = c.struct({ c.int, c.int:ptr() })
	local ResultStruct = c.struct({ c.int, c.int })

	local AB = c.fn({ ArgStruct }, ResultStruct)

	local AB_callable = AB:callable(lib:find("AB"))

	local resultBox = ffi.box(ResultStruct.size)
	local b = c.int:box(200)
	local arg = ArgStruct:box({ 100, b:ref() })

	AB_callable(resultBox, arg:ref())
	local result = ResultStruct:readData(resultBox)

	assert(result[1] == 300, `AB failed. result expected 300, got {result[1]}`)
	assert(result[2] == 20000, `AB failed. result expected 300, got {result[2]}`)
end

test_AB()
