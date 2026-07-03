--!nocheck
--!nolint
-- Numeric casts across type pairs: widening, narrowing/truncation, sign
-- changes, and float/int conversion. Cast follows Rust `as` semantics.
local ffi = require("@lune/ffi")

local function castValue(fromType, intoType, value): any
	local fromBox = fromType:box(value)
	local intoBox = ffi.box(intoType.size)
	fromType:cast(intoType, fromBox, intoBox)
	return intoType:readData(intoBox)
end

-- Integer widening keeps the value
assert(castValue(ffi.u8, ffi.u32, 200) == 200, "u8 -> u32 should widen")
assert(castValue(ffi.i32, ffi.i64, -5) == -5, "i32 -> i64 should sign-extend")

-- Integer narrowing truncates (wraps modulo the target width)
assert(castValue(ffi.u32, ffi.u8, 300) == 300 % 256, "u32 -> u8 should truncate to 44")
assert(castValue(ffi.i32, ffi.u8, 257) == 1, "i32 -> u8 should keep the low byte")

-- Sign reinterpretation follows two's complement
assert(castValue(ffi.i32, ffi.u32, -1) == 4294967295, "i32 -1 -> u32 wraps to max")

-- Float to int truncates toward zero
assert(castValue(ffi.f32, ffi.i32, 3.9) == 3, "f32 3.9 -> i32 truncates to 3")
assert(castValue(ffi.f64, ffi.i32, -2.9) == -2, "f64 -2.9 -> i32 truncates to -2")

-- Int to float, and float narrowing that stays exact
assert(castValue(ffi.i32, ffi.f64, 42) == 42, "i32 -> f64")
assert(castValue(ffi.f64, ffi.f32, 1.5) == 1.5, "f64 -> f32 exact for 1.5")

-- Casting out of the source or destination bounds is rejected
assert(not pcall(function()
	ffi.i32:cast(ffi.i32, ffi.box(1), ffi.box(4))
end), "casting from a too-small source should error")
assert(not pcall(function()
	ffi.i32:cast(ffi.i32, ffi.box(4), ffi.box(1))
end), "casting into a too-small destination should error")

print("casts: numeric conversions follow Rust as-semantics")
