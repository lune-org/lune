--!nocheck
--!nolint
local ffi = require("@lune/ffi")

-- A box stringifies to the little-endian hex of its content, so it is
-- deterministic and can be asserted directly.
local box = ffi.i32:box(255)
assert(tostring(box) == "0xff000000", `unexpected box stringify: {tostring(box)}`)

local zero = ffi.i32:box(0)
assert(tostring(zero) == "0x00000000", `unexpected zeroed box stringify: {tostring(zero)}`)

print(box)
