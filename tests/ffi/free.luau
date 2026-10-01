--!nocheck
--!nolint
local ffi = require("@lune/ffi")

-- NOTE: Scripts cannot force a full GC cycle in Lune, since sandboxed Luau
-- only allows collectgarbage("count"), so we allocate a bunch of short-lived
-- values to apply GC pressure instead
local function gcPressure()
	for _ = 1, 64 do
		local t = table.create(1024)
		for i = 1, 1024 do
			t[i] = { i }
		end
	end
end

local box = ffi.box(ffi.i32.size)
local ref = box:leak()

box = nil

gcPressure()

ffi.free(ref)

gcPressure()
