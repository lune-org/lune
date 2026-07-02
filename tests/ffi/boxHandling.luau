--!nocheck
--!nolint
-- Box allocation, zeroing, refs, and the freeing lifecycle.
local ffi = require("@lune/ffi")

local function shouldError(fn: () -> (), msg: string)
	assert(not (pcall(fn)), msg)
end

-- Basic allocation and size
local box = ffi.box(64)
assert(box.size == 64, `box.size should be 64, got {box.size}`)

-- zero() returns the box itself and clears memory
assert(rawequal(box:zero(), box), "box:zero() should return the box itself")
assert(ffi.u32:readData(box) == 0, "zeroed box should read 0")

-- Writing then reading at an offset
ffi.u32:writeData(box, 0xDEADBEEF, 8)
assert(ffi.u32:readData(box, 8) == 0xDEADBEEF, "offset write/read failed")
assert(ffi.u32:readData(box, 0) == 0, "unrelated bytes should stay zero")

-- ref() keeps the box alive and points at the same memory
local ref = box:ref()
assert(ffi.u32:readData(ref, 8) == 0xDEADBEEF, "ref should see the box's data")

-- A zero-sized box is not allowed
shouldError(function()
	ffi.box(0)
end, "ffi.box(0) should error")

-- Offsetting a ref past the box is rejected
shouldError(function()
	box:ref(65)
end, "ref offset past the box should error")

-- Freeing a leaked box, then using it, should error
local leaked = ffi.box(16)
local leakedRef = leaked:leak()
ffi.free(leakedRef)

-- Freeing a plain (GC-managed) box prevents the destructor from double-freeing
local managed = ffi.box(16)
ffi.free(managed)
shouldError(function()
	managed:zero()
end, "using a freed box should error")
shouldError(function()
	ffi.u8:readData(managed)
end, "reading a freed box should error")

-- Double free is rejected instead of corrupting the heap
shouldError(function()
	ffi.free(managed)
end, "double free should error")

-- Stringifying a freed box must not read freed memory
assert(tostring(managed):find("freed") ~= nil, "a freed box should stringify as freed")

-- free() rejects things that are not a box or reference
shouldError(function()
	ffi.free(ffi.c.int)
end, "free of a non-data value should error")

print("boxHandling: allocation and freeing lifecycle behaves correctly")
