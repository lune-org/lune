--!nocheck
--!nolint
-- Pointer null checks, layout, and reading/writing addresses with readRef,
-- writeRef, and the pointer box helper.
local ffi = require("@lune/ffi")

local function shouldError(fn: () -> (), msg: string)
	assert(not (pcall(fn)), msg)
end

-- Null references
assert(ffi.nullRef():isNull(), "nullRef() should be null")
assert(not ffi.box(8):ref():isNull(), "a real box ref should not be null")

-- Pointer type layout
local intPtr = ffi.i32:ptr()
assert(intPtr.size == ffi.usize.size, "pointer size should equal usize size")
assert(rawequal(intPtr.inner, ffi.i32), "pointer inner type should be i32")

-- Store an address with the pointer box helper, then read it back
local target = ffi.i32:box(0x1234)
local pointerBox = intPtr:box(target:ref())
local readBack = intPtr:readRef(pointerBox)
assert(ffi.i32:readData(readBack) == 0x1234, "address roundtrip through :box failed")

-- writeRef overwrites the stored address, readRef into an existing ref reuses it
local other = ffi.i32:box(0x5678)
intPtr:writeRef(pointerBox, other:ref())
local reused = ffi.nullRef()
local returned = intPtr:readRef(pointerBox, 0, reused)
assert(rawequal(returned, reused), "readRef with a ref argument should reuse it")
assert(ffi.i32:readData(reused) == 0x5678, "writeRef/readRef roundtrip failed")

-- A box can hold several pointers; index them by offset
local table3 = ffi.box(intPtr.size * 3)
local a, b, c = ffi.i32:box(10), ffi.i32:box(20), ffi.i32:box(30)
intPtr:writeRef(table3, a:ref(), intPtr.size * 0)
intPtr:writeRef(table3, b:ref(), intPtr.size * 1)
intPtr:writeRef(table3, c:ref(), intPtr.size * 2)
assert(ffi.i32:readData(intPtr:readRef(table3, intPtr.size * 0)) == 10, "pointer table [0] failed")
assert(ffi.i32:readData(intPtr:readRef(table3, intPtr.size * 1)) == 20, "pointer table [1] failed")
assert(ffi.i32:readData(intPtr:readRef(table3, intPtr.size * 2)) == 30, "pointer table [2] failed")

-- readRef with a non-RefData ref argument is rejected
shouldError(function()
	intPtr:readRef(pointerBox, 0, ffi.box(8))
end, "readRef with a non-ref 'ref' argument should error")

print("pointers: address read/write and null handling behave correctly")
