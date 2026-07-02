--!nocheck
--!nolint
local ffi = require("@lune/ffi")

-- A null reference stringifies to its address, which is 0x0
assert(
	tostring(ffi.nullRef()) == "0x0",
	`nullRef should stringify to 0x0, got {tostring(ffi.nullRef())}`
)

-- A real reference stringifies to its hex address (value is not deterministic)
local ref = ffi.i32:box(1):ref()
assert(
	tostring(ref):match("^0x%x+$") ~= nil,
	`ref should stringify as a hex address, got {tostring(ref)}`
)

print(ref)
