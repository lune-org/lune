--!nocheck
--!nolint
-- Roundtrip every numeric ctype through a box, checking sizes, signedness,
-- and read/write correctness (including the fixed 16-byte u128/i128 layout).
local ffi = require("@lune/ffi")

type Case = { type: any, size: number, signed: boolean, values: { number } }

local cases: { Case } = {
	{ type = ffi.u8, size = 1, signed = false, values = { 0, 1, 127, 255 } },
	{ type = ffi.i8, size = 1, signed = true, values = { -128, -1, 0, 127 } },
	{ type = ffi.u16, size = 2, signed = false, values = { 0, 255, 65535 } },
	{ type = ffi.i16, size = 2, signed = true, values = { -32768, -1, 32767 } },
	{ type = ffi.u32, size = 4, signed = false, values = { 0, 65536, 4294967295 } },
	{ type = ffi.i32, size = 4, signed = true, values = { -2147483648, -1, 2147483647 } },
	{ type = ffi.u64, size = 8, signed = false, values = { 0, 1, 4294967296 } },
	{ type = ffi.i64, size = 8, signed = true, values = { -4294967296, -1, 4294967296 } },
	{ type = ffi.f32, size = 4, signed = true, values = { 0, 1, -1, 0.5 } },
	{ type = ffi.f64, size = 8, signed = true, values = { 0, 3.5, -2.25, 1234.5 } },
	{ type = ffi.usize, size = ffi.usize.size, signed = false, values = { 0, 1, 4096 } },
	{ type = ffi.isize, size = ffi.isize.size, signed = true, values = { -4096, 0, 4096 } },
}

for _, case in cases do
	assert(
		case.type.size == case.size,
		`size mismatch: expected {case.size}, got {case.type.size}`
	)
	assert(
		case.type.signedness == case.signed,
		`signedness mismatch for a size-{case.size} type`
	)

	local box = ffi.box(case.type.size)
	for _, value in case.values do
		case.type:writeData(box, value)
		local readBack = case.type:readData(box)
		assert(readBack == value, `roundtrip failed: wrote {value}, read {readBack}`)
	end
end

-- The 128-bit integer types must be a full 16 bytes wide. Their exact value is
-- validated through stringifyData, since Luau numbers cannot hold 128 bits.
assert(ffi.u128.size == 16, `u128 size should be 16, got {ffi.u128.size}`)
assert(ffi.i128.size == 16, `i128 size should be 16, got {ffi.i128.size}`)

local u128box = ffi.box(ffi.u128.size)
ffi.u128:writeData(u128box, "340282366920938463463374607431768211455") -- 2^128 - 1
assert(
	ffi.u128:stringifyData(u128box) == "340282366920938463463374607431768211455",
	"u128 max value did not roundtrip through 16-byte storage"
)

local i128box = ffi.box(ffi.i128.size)
ffi.i128:writeData(i128box, "-170141183460469231731687303715884105728") -- -2^127
assert(
	ffi.i128:stringifyData(i128box) == "-170141183460469231731687303715884105728",
	"i128 min value did not roundtrip through 16-byte storage"
)

-- Writing past the low half of a 128-bit box must not corrupt the high half:
-- a naive 8-byte-wide type would have overflowed here before the fix.
local probe = ffi.box(ffi.u128.size)
probe:writeString(string.rep("\255", 16))
ffi.u128:writeData(probe, "0")
assert(ffi.u128:stringifyData(probe) == "0", "u128 write did not clear all 16 bytes")

print("numbers: all numeric types roundtrip correctly")
