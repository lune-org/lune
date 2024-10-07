--!nocheck
--!nolint

local ffi = require("@lune/ffi")

local i32ptr = ffi.i32:ptr()
local struct = ffi.struct({ i32ptr, ffi.i32 })

assert(rawequal(struct:field(0), i32ptr), "Struct get field failed")
assert(rawequal(struct:field(1), ffi.i32), "Struct get field failed")

-- offset(2) should fail
