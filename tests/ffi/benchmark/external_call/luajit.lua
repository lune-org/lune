--!nolint
--!nocheck

local ffi = require("ffi")

local function bench_add(bench_scale)
	ffi.cdef([[
		int add(int a, int b);
    ]])
	local lib = ffi.load("./tests/ffi/benchmark/external_call/lib.so")
	local add = lib.add
	local a = 0

	local before = os.clock()
	for i = 1, bench_scale do
		a = add(a, 1)
	end
	local after = os.clock()

	print(after - before)
	assert(
		a == bench_scale,
		string.format("bench_add failed. result expected %d, got %d", bench_scale, a)
	)
end

bench_add(1000000)
