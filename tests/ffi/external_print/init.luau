local ffi = require("@lune/ffi")
local c = ffi.c

local testdir = "./tests/ffi/external_print"
local compile = require("../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function test_hello_world()
	local hello_world_info = c.fn({}, c.void)

	local hello_world_callable = hello_world_info:callable(lib:find("hello_world"))

	hello_world_callable(nil)
end

test_hello_world()
