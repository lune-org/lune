--!nolint
--!nocheck

local ffi = require("ffi")
local BENCH_SCALE = 1000000

ffi.cdef([[
	int add(int a, int b);
]])
local lib = ffi.load("./tests/ffi/benchmark/external_call/lib.so")
local add = lib.add
local a = 0

local before = os.clock()
for i = 1, BENCH_SCALE do
	a = add(a, 1)
end
local after = os.clock()

print(after - before)
assert(
	a == BENCH_SCALE,
	string.format("bench_add failed. result expected %d, got %d", BENCH_SCALE, a)
)
