local ffi = require("@lune/ffi")
local c = ffi.c

local proc_clock = require("../../utility/proc_clock")
local before, after = proc_clock.newBox()
local get_clock = proc_clock.getClock

local testdir = "./tests/ffi/benchmark/external_call"
local compile = require("../../utility/compile")
compile(`{testdir}/lib.c`, `{testdir}/lib.so`)
local lib = ffi.open(`{testdir}/lib.so`)

local function bench_add(bench_scale: number)
	local add_info = c.fn({ c.int, c.int }, c.int)

	local add_callable = add_info:callable(lib:find("add"))

	local a = c.int:box(0)
	local delta = c.int:box(1)

	local a_ref = a:ref()
	local delta_ref = delta:ref()

	get_clock(before)
	for i = 1, bench_scale do
		add_callable(a, a_ref, delta_ref)
	end
	get_clock(after)
	print(proc_clock.getOffset(before, after))

	local result = c.int:readData(a)
	assert(result == bench_scale, `bench_add failed. result expected {bench_scale}, got {result}`)
end

bench_add(1000000)
