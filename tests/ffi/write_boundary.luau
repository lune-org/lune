local ffi = require("@lune/ffi")
local c = ffi.c

local ok

-- Case1: Fail
ok = pcall(function()
	local box = ffi.box(c.int.size - 1)
	c.int:writeData(box, 10)
end)
assert(not ok, "assersion failed, Case1 should fail")

-- Case2: Success
ok = pcall(function()
	local box = ffi.box(c.int.size)
	c.int:writeData(box, 10)
end)
assert(ok, "assersion failed, Case2 should success")

-- Case3: Success
ok = pcall(function()
	local box = ffi.box(c.int.size * 2)
	c.int:writeData(box, 10, c.int.size)
end)
assert(ok, "assersion failed, Case3 should success")

-- Case4: Fail
ok = pcall(function()
	local box = ffi.box(c.int.size * 2)
	c.int:writeData(box, 10, c.int.size * 2)
end)
assert(not ok, "assersion failed, Case4 should fail")

-- Case5: Success
ok = pcall(function()
	local box = ffi.box(c.int.size * 2):ref()
	c.int:writeData(box, 10, c.int.size)
end)
assert(ok, "assersion failed, Case5 should success")

-- Case6: Fail
ok = pcall(function()
	local box = ffi.box(c.int.size * 2):ref()
	c.int:writeData(box, 10, c.int.size * 2)
end)
assert(not ok, "assersion failed, Case6 should fail")
