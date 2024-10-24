local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_closure"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_callClosure()
	local closureInfo = c.fn({ c.int, c.int }, c.int)
	local closure = closureInfo:closure(function(ret, a, b)
		c.int:writeData(ret, c.int:readData(a) + c.int:readData(b))
	end)

	local callClosure = c.fn({ closureInfo }, c.int):callable(lib:find("call_closure"))

	local resultBox = ffi.box(c.int.size)
	callClosure(resultBox, closure:ref())
	local result = c.int:readData(resultBox)
	assert(result == 72, `test_callClosure failed. result expected 20000, got {result}`)
end

test_callClosure()

local function test_helloWorld()
	local helloWorldInfo = c.fn({}, c.void)
	local helloWorld = helloWorldInfo:closure(function()
		print("Hello world in lua closure!")
	end)

	local callHelloWorld = c.fn({ helloWorldInfo }, c.void):callable(lib:find("call_hello_world"))
	callHelloWorld(nil, helloWorld:ref())
end

test_helloWorld()

local function test_closureWithPointer()
	local closureWithPointerInfo = c.fn({ c.int, c.int:ptr() }, c.int)
	local closureWithPointer = closureWithPointerInfo:closure(function(returnRef, aRef, bRef)
		c.int:writeData(returnRef, c.int:readData(aRef) + c.int:readData(bRef:deref()))
	end)

	local callClosureWithPointer = c.fn({ closureWithPointerInfo }, c.int)
		:callable(lib:find("call_closure_with_pointer"))

	local resultBox = ffi.box(c.int.size)
	callClosureWithPointer(resultBox, closureWithPointer:ref())
	local result = c.int:readData(resultBox)
	assert(result == 72, `test_closureWithPointer failed. result expected 20000, got {result}`)
end

test_closureWithPointer()
