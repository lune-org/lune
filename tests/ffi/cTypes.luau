--!nocheck
--!nolint
-- The compiler-defined C types exist, have standard-conforming sizes and
-- signedness, and roundtrip values through a box.
local ffi = require("@lune/ffi")
local c = ffi.c

-- Sizes fixed or bounded by the C standard (portable across platforms)
assert(c.char.size == 1, `char should be 1 byte, got {c.char.size}`)
assert(c.uchar.size == 1 and c.schar.size == 1, "uchar and schar should be 1 byte")
assert(c.short.size >= 2, "short should be at least 2 bytes")
assert(c.int.size >= 2, "int should be at least 2 bytes")
assert(c.long.size >= 4, "long should be at least 4 bytes")
assert(c.longlong.size >= 8, "longlong should be at least 8 bytes")

-- Non-decreasing size ordering
assert(c.char.size <= c.short.size, "char should not be larger than short")
assert(c.short.size <= c.int.size, "short should not be larger than int")
assert(c.int.size <= c.long.size, "int should not be larger than long")
assert(c.long.size <= c.longlong.size, "long should not be larger than longlong")

-- Signed and unsigned variants share a size
assert(c.short.size == c.ushort.size, "short and ushort should share a size")
assert(c.int.size == c.uint.size, "int and uint should share a size")
assert(c.long.size == c.ulong.size, "long and ulong should share a size")
assert(c.longlong.size == c.ulonglong.size, "longlong and ulonglong should share a size")

-- Signedness (char itself is platform-defined, so it is not asserted here)
assert(c.uchar.signedness == false and c.schar.signedness == true, "schar/uchar signedness")
assert(c.ushort.signedness == false and c.short.signedness == true, "short/ushort signedness")
assert(c.uint.signedness == false and c.int.signedness == true, "int/uint signedness")
assert(c.ulong.signedness == false and c.long.signedness == true, "long/ulong signedness")
assert(
	c.ulonglong.signedness == false and c.longlong.signedness == true,
	"longlong/ulonglong signedness"
)

-- A small positive value roundtrips through every type
local all = {
	char = c.char,
	uchar = c.uchar,
	schar = c.schar,
	short = c.short,
	ushort = c.ushort,
	int = c.int,
	uint = c.uint,
	long = c.long,
	ulong = c.ulong,
	longlong = c.longlong,
	ulonglong = c.ulonglong,
}
for name, ctype in all do
	local box = ctype:box(7)
	assert(ctype:readData(box) == 7, `{name} did not roundtrip 7`)
end

-- Negatives roundtrip through the signed types
local signed = { schar = c.schar, short = c.short, int = c.int, long = c.long, longlong = c.longlong }
for name, ctype in signed do
	local box = ctype:box(-2)
	assert(ctype:readData(box) == -2, `{name} did not roundtrip -2`)
end

print("cTypes: compiler-defined C types have sane sizes and roundtrip")
