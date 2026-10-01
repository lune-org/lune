local ffi = require("@lune/ffi")
local ok

local str = "hello world"
local strbox = ffi.box(#str):writeString(str)
assert(strbox:readString(#str) == str, "String read write assersion failed")

-- Case1: Fail
ok = pcall(function()
	local box = ffi.box(2)
	box:readString(10)
end)
assert(not ok, "assersion failed, Case1 should fail")

-- Case2: Fail
ok = pcall(function()
	local box = ffi.box(2)
	box:writeString("hello world")
end)
assert(not ok, "assersion failed, Case2 should fail")

-- Case3: Fail
ok = pcall(function()
	local box = ffi.box(2):ref()
	box:readString(10)
end)
assert(not ok, "assersion failed, Case3 should fail")

-- Case4: Fail
ok = pcall(function()
	local box = ffi.box(2):ref()
	box:writeString("hello world")
end)
assert(not ok, "assersion failed, Case4 should fail")
