local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_closure/lib.c")
local c = ffi.c

-- Create closure
local helloWorldInfo = c.fn({}, c.void)
local helloWorld = helloWorldInfo:closure(function()
	print("Hello world in lua closure!")
end)

local callHelloWorld = c.fn({ helloWorldInfo }, c.void):callable(lib:find("call_hello_world"))
callHelloWorld(nil, helloWorld:ref())
