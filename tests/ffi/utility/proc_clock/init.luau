local ffi = require("@lune/ffi")
local c = ffi.c

local proc_clock = {}

local libdir = "./tests/ffi/utility/proc_clock"
local compile = require("../compile")
compile(`{libdir}/lib.c`, `{libdir}/lib.so`)
local lib = ffi.open(`{libdir}/lib.so`)

-- sizeof_clock
local sizeof_clock = c.fn({}, c.int):callable(lib:find("sizeof_clock"))
function proc_clock.sizeof_clock()
	local result = ffi.box(c.int.size)
	sizeof_clock(result)
	return c.int:readData(result)
end

-- get_clock
local clock_t = ffi["u" .. (proc_clock.sizeof_clock() * 8)]
assert(clock_t, "clock_t is unknown type")
local get_clock = c.fn({}, clock_t):callable(lib:find("get_clock"))
proc_clock.get_clock = get_clock

-- get_offset
local get_offset = c.fn({ clock_t, clock_t }, ffi.f64):callable(lib:find("get_offset"))
function proc_clock.get_offset(before, after)
	local result = ffi.box(ffi.f64.size)
	get_offset(result, before:ref(), after:ref())
	return ffi.f64:readData(result)
end

function proc_clock.new_box()
	return ffi.box(clock_t.size), ffi.box(clock_t.size)
end

return proc_clock
