-- FIXME: in windows, we need another library to get process cpu time

local ffi = require("@lune/ffi")
local process = require("@lune/process")
local isWindows = process.os == "windows"
local c = ffi.c

local procClock = {}

local lib = require("../compile")("./tests/ffi/utils/proc_clock/lib.c")

-- sizeof_clock
local sizeofClock = c.fn({}, c.int):callable(lib:find("sizeof_clock"))
function procClock.sizeofClock(): number
	local result = ffi.box(c.int.size)
	sizeofClock(result)
	return c.int:readData(result)
end
-- get_clock
local clock_t = if isWindows then ffi.f32 else ffi["u" .. (procClock.sizeofClock() * 8)]
assert(clock_t, "clock_t is unknown type")
procClock.getClock = (
	if isWindows
		then function(clock: ffi.BoxData | ffi.RefData)
			ffi.f32:writeData(clock, os.clock())
		end
		else c.fn({}, clock_t):callable(lib:find("get_clock"))
) :: (ffi.BoxData | ffi.RefData) -> ()

-- get_offset
local getOffset: (ffi.BoxData, ffi.RefData, ffi.RefData) -> () = if isWindows
	then function(result: ffi.BoxData, before: ffi.RefData, after: ffi.RefData)
		ffi.f64:writeData(result, (ffi.f32:readData(after) - ffi.f32:readData(before)))
	end
	else c.fn({ clock_t, clock_t }, ffi.f64):callable(lib:find("get_offset"))
function procClock.getOffset(before: ffi.BoxData, after: ffi.BoxData): number
	local result = ffi.box(ffi.f64.size)
	getOffset(result, before:ref(), after:ref())
	return ffi.f64:readData(result)
end

function procClock.newBox(): (ffi.BoxData, ffi.BoxData)
	return ffi.box(clock_t.size), ffi.box(clock_t.size)
end

return procClock
