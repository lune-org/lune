local ffi = require("@lune/ffi")
local ok

-- Case1: Success
ok = pcall(function()
	local box = ffi.u8:box(1)
	ffi.u8:readData(box)
end)
assert(ok, "assersion failed, Case1 should success")

-- Case2: Fail
ok = pcall(function()
	local box = ffi.u8:box(1)
	ffi.u16:readData(box)
end)
assert(not ok, "assersion failed, Case2 should fail")

-- Case3: Success
ok = pcall(function()
	local box = ffi.box(ffi.u8.size * 2)
	ffi.u16:readData(box)
end)
assert(ok, "assersion failed, Case3 should success")

-- Case4: Success
ok = pcall(function()
	local box = ffi.box(ffi.u8.size * 2)
	ffi.u8:readData(box, ffi.u8.size)
end)
assert(ok, "assersion failed, Case4 should success")

-- Case5: Fail
ok = pcall(function()
	local box = ffi.u8:box(1):ref()
	ffi.u16:readData(box)
end)
assert(not ok, "assersion failed, Case5 should fail")

-- Case6: Success
ok = pcall(function()
	local box = ffi.box(ffi.u8.size * 2):ref()
	ffi.u16:readData(box)
end)
assert(ok, "assersion failed, Case6 should success")

-- Case7: Fail
ok = pcall(function()
	local box = ffi.box(ffi.u8.size * 2):ref(ffi.u16.size)
	ffi.u16:readData(box)
end)
assert(not ok, "assersion failed, Case7 should fail")
