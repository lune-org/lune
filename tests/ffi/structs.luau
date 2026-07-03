--!nocheck
--!nolint
-- Struct field layout with padding, read/write, nesting, introspection, and
-- rejected shapes.
local ffi = require("@lune/ffi")

local function shouldError(fn: () -> (), msg: string)
	assert(not (pcall(fn)), msg)
end

-- Padding: u8 then i32 -> i32 is aligned to offset 4, total size 8
local mixed = ffi.c.struct({ ffi.u8, ffi.i32 })
assert(mixed:offset(0) == 0, "field 0 should be at offset 0")
assert(mixed:offset(1) == 4, `field 1 should be padded to offset 4, got {mixed:offset(1)}`)
assert(mixed.size == 8, `struct size should be 8 with padding, got {mixed.size}`)

-- Field introspection is 0-based and returns the original type userdata
assert(rawequal(mixed:field(0), ffi.u8), "field(0) should be u8")
assert(rawequal(mixed:field(1), ffi.i32), "field(1) should be i32")

-- Read/write roundtrip
local box = mixed:box({ 200, -12345 })
local out = mixed:readData(box)
assert(out[1] == 200, `field 0 expected 200, got {out[1]}`)
assert(out[2] == -12345, `field 1 expected -12345, got {out[2]}`)

-- Writing a single field by its offset does not disturb the other
ffi.i32:writeData(box, 777, mixed:offset(1))
local out2 = mixed:readData(box)
assert(out2[1] == 200 and out2[2] == 777, "targeted field write disturbed the struct")

-- Nested struct
local inner = ffi.c.struct({ ffi.i32, ffi.i32 })
local outer = ffi.c.struct({ ffi.u8, inner })
local nestedBox = outer:box({ 9, { 100, 200 } })
local nested = outer:readData(nestedBox)
assert(nested[1] == 9, "outer field mismatch")
assert(nested[2][1] == 100 and nested[2][2] == 200, "inner struct fields mismatch")

-- copyData duplicates the whole struct
local copy = ffi.box(outer.size)
outer:copyData(copy, nestedBox)
local copied = outer:readData(copy)
assert(copied[2][2] == 200, "copyData did not duplicate nested struct")

-- Introspection out of range
shouldError(function()
	mixed:field(2)
end, "field(2) on a 2-field struct should error")
shouldError(function()
	mixed:offset(2)
end, "offset(2) on a 2-field struct should error")

-- Illegal struct shapes
shouldError(function()
	ffi.c.struct({})
end, "empty struct should error")
shouldError(function()
	ffi.c.struct({ ffi.i32, ffi.c.void })
end, "struct with a void field should error")

print("structs: layout, nesting, and introspection behave correctly")
