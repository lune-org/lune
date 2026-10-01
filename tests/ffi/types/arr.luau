local ffi = require("@lune/ffi")
local ok

local arr = ffi.i32:arr(4)

assert(rawequal(arr.length, 4), "length assersion failed, arr.length should be 4")
assert(rawequal(arr.inner, ffi.i32), "inner assersion failed, arr.inner should be ffi.i32")

-- offset(2) success
ok = pcall(function()
	arr:offset(2)
end)
assert(ok, "assersion failed, arr:offset(2) should success")

-- offset(4) success
ok = pcall(function()
	arr:offset(4)
end)
assert(not ok, "assersion failed, arr:offset(4) should fail")
