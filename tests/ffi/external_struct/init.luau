local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_struct"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_AB()
	local argStructInfo = c.struct({ c.int, c.int:ptr() })
	local resultStructInfo = c.struct({ c.int, c.int })

	local AB = c.fn({ argStructInfo }, resultStructInfo)

	local AB_callable = AB:callable(lib:find("AB"))

	local resultBox = ffi.box(resultStructInfo.size)
	local b = c.int:box(200)
	local argStruct = argStructInfo:box({ 100, b:ref() })

	AB_callable(resultBox, argStruct:ref())
	local result = resultStructInfo:readData(resultBox)

	assert(result[1] == 300, `AB failed. result expected 300, got {result[1]}`)
	assert(result[2] == 20000, `AB failed. result expected 300, got {result[2]}`)
end

test_AB()
