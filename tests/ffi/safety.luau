--!nocheck
--!nolint
-- Negative tests for the bounds and permission guards. Each of these must
-- raise a clean Lua error rather than read/write out of bounds.
local ffi = require("@lune/ffi")

local function shouldError(fn: () -> (), msg: string)
	assert(not (pcall(fn)), msg)
end

-- Reading a type larger than the target
shouldError(function()
	ffi.u32:readData(ffi.box(2))
end, "reading u32 from a 2-byte box should error")

-- Negative offsets are rejected
shouldError(function()
	ffi.u8:readData(ffi.box(4), -1)
end, "a negative offset should error")

-- A huge offset must be rejected without integer underflow
shouldError(function()
	ffi.u8:readData(ffi.box(4), 999999999)
end, "a huge offset should error, not underflow")
shouldError(function()
	ffi.u8:writeData(ffi.box(4), 1, 999999999)
end, "a huge write offset should error, not underflow")

-- Writing a type larger than the target
shouldError(function()
	ffi.i64:writeData(ffi.box(4), 1)
end, "writing i64 into a 4-byte box should error")

-- copyData bounds are checked on both sides
shouldError(function()
	ffi.u32:copyData(ffi.box(2), ffi.box(4))
end, "copyData into a too-small destination should error")
shouldError(function()
	ffi.u32:copyData(ffi.box(4), ffi.box(2))
end, "copyData from a too-small source should error")

-- void cannot be used as a function argument or struct field
shouldError(function()
	ffi.c.fn({ ffi.c.void }, ffi.c.int)
end, "void as a function argument should error")

-- variadic fixed-argument counts are validated
shouldError(function()
	ffi.c.fn({ ffi.c.int, ffi.c.int }, ffi.c.int, 0)
end, "a variadic function with zero fixed arguments should error")
shouldError(function()
	ffi.c.fn({ ffi.c.int }, ffi.c.int, 5)
end, "a fixed-argument count larger than the argument list should error")

-- Struct/array writes require a table value
shouldError(function()
	ffi.c.struct({ ffi.i32, ffi.i32 }):box(5)
end, "boxing a struct from a non-table should error")

-- cast bounds are validated
shouldError(function()
	-- source is only 1 byte but we cast from a 4-byte type
	ffi.i32:cast(ffi.i32, ffi.box(1), ffi.box(4))
end, "casting out of the source bounds should error")

-- Writing a string past the destination boundary is rejected
shouldError(function()
	ffi.box(4):writeString("this is longer than four bytes")
end, "writing a string past the box should error")

print("safety: bounds and permission guards reject invalid access")
