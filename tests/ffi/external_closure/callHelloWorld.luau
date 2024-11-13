local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_closure/lib.c")
local c = ffi.c

-- Create closure
local helloWorld = c.fn({}, c.void):closure(function()
	print("Hello world in lua closure!")
end)

local callHelloWorld = c.fn({ c.void:ptr() }, c.void):callable(lib:find("call_hello_world"))
callHelloWorld(nil, helloWorld:ref())
