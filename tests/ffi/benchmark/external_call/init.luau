local ffi = require("@lune/ffi")
local lib = require("../../utility/compile")("./tests/ffi/benchmark/external_call/lib.c")
local process = require("@lune/process")
local c = ffi.c
local BENCH_SCALE: number = tonumber(process.env.BENCH_SCALE) or 1000000

-- Get clock provider
local procClock = require("../../utility/proc_clock")
local before, after = procClock.newBox()
local getClock = procClock.getClock

local add = c.fn({ c.int, c.int }, c.int):callable(lib:find("add"))

local a = c.int:box(0)
local delta = c.int:box(1)
local a_ref = a:ref()
local delta_ref = delta:ref()

getClock(before)
for i = 1, BENCH_SCALE do
	add(a, a_ref, delta_ref)
end
getClock(after)

print(procClock.getOffset(before, after))
local result = c.int:readData(a)
assert(result == BENCH_SCALE, `bench_add failed. result expected {BENCH_SCALE}, got {result}`)
