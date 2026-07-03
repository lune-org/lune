--!nocheck
--!nolint
-- readCString reads a null-terminated string and stops at the data boundary
-- when there is no terminator.
local ffi = require("@lune/ffi")

local function shouldError(fn: () -> (), msg: string)
	assert(not (pcall(fn)), msg)
end

-- Basic null-terminated read
local box = ffi.box(16):zero()
box:writeString("hello")
assert(box:readCString() == "hello", "readCString should stop at the null terminator")

-- Reading from an offset
box:writeString("world", nil, 6)
assert(box:readCString(6) == "world", "readCString with an offset failed")

-- readString still reads the exact requested length, terminator or not
assert(box:readString(5) == "hello", "readString should read a fixed length")

-- A reference into the box reads the same string
assert(box:ref():readCString() == "hello", "readCString through a ref failed")

-- No terminator within the boundary is an error, not an over-read
local full = ffi.box(4)
full:writeString("aaaa")
shouldError(function()
	full:readCString()
end, "readCString without a terminator in bounds should error")

-- An empty string (immediate terminator) reads as ""
local empty = ffi.box(4):zero()
assert(empty:readCString() == "", "an immediate terminator should read as an empty string")

print("readCString: null-terminated string reading behaves correctly")
