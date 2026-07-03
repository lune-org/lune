--!nocheck
--!nolint
-- Array layout, whole-array and per-element access, bounds, large arrays,
-- and arrays of structs.
local ffi = require("@lune/ffi")

local function shouldError(fn: () -> (), msg: string)
	assert(not (pcall(fn)), msg)
end

-- Layout
local arr = ffi.i32:arr(4)
assert(arr.length == 4, `arr.length should be 4, got {arr.length}`)
assert(arr.size == 16, `arr.size should be 16, got {arr.size}`)
assert(rawequal(arr.inner, ffi.i32), "arr.inner should be the element type")
assert(arr:offset(0) == 0, "element 0 offset should be 0")
assert(arr:offset(3) == 12, "element 3 offset should be 12")

-- Whole-array roundtrip
local box = arr:box({ 11, 22, 33, 44 })
local out = ffi.i32:arr(4):readData(box)
assert(#out == 4, `read array should have 4 elements, got {#out}`)
for i, expected in { 11, 22, 33, 44 } do
	assert(out[i] == expected, `element {i} expected {expected}, got {out[i]}`)
end

-- Per-element access through offsets
assert(ffi.i32:readData(box, arr:offset(2)) == 33, "element 2 via offset failed")
ffi.i32:writeData(box, 99, arr:offset(1))
assert(ffi.i32:readData(box, arr:offset(1)) == 99, "per-element write failed")

-- Index bounds
shouldError(function()
	arr:offset(4)
end, "offset(4) on a length-4 array should error")
shouldError(function()
	arr:offset(-1)
end, "negative index should error")
shouldError(function()
	ffi.u8:arr(0)
end, "zero-length array should error")

-- Large array roundtrip with pseudo-random data
local COUNT = 512
local big = ffi.u32:arr(COUNT)
local values = table.create(COUNT)
local seed = 2463534242
for i = 1, COUNT do
	-- xorshift for repeatable pseudo-random values
	seed = bit32.bxor(seed, bit32.lshift(seed, 13))
	seed = bit32.bxor(seed, bit32.rshift(seed, 17))
	seed = bit32.bxor(seed, bit32.lshift(seed, 5))
	values[i] = seed
end
local bigBox = big:box(values)
local bigOut = big:readData(bigBox)
for i = 1, COUNT do
	assert(bigOut[i] == values[i], `large array mismatch at {i}`)
end

-- Whole-array copyData duplicates every element
local copy = ffi.box(arr.size)
arr:copyData(copy, box)
local copyOut = ffi.i32:arr(4):readData(copy)
assert(copyOut[1] == 11 and copyOut[3] == 33, "array copyData did not duplicate elements")

-- Array of structs
local point = ffi.c.struct({ ffi.i32, ffi.i32 })
local points = point:arr(3)
assert(points.size == point.size * 3, "struct array size should be 3 * struct size")
local pointsBox = points:box({ { 1, 2 }, { 3, 4 }, { 5, 6 } })
local pointsOut = points:readData(pointsBox)
assert(pointsOut[2][1] == 3 and pointsOut[2][2] == 4, "struct array element 2 mismatch")
assert(pointsOut[3][1] == 5 and pointsOut[3][2] == 6, "struct array element 3 mismatch")

print(`arrays: {COUNT}-element and struct arrays roundtrip correctly`)
