local ffi = require("@lune/ffi")
local ok

local i32ptr = ffi.i32:ptr()
local struct = ffi.c.struct({ i32ptr, ffi.i32 })

assert(rawequal(struct:field(0), i32ptr), "Struct get field failed")
assert(rawequal(struct:field(1), ffi.i32), "Struct get field failed")

-- offset(2) should fail
ok = pcall(function()
	struct:offset(2)
end)
assert(not ok, "assersion failed, struct:offset(2) should fail")

-- field(2) should fail
ok = pcall(function()
	struct:field(2)
end)
assert(not ok, "assersion failed, struct:field(2) should fail")
