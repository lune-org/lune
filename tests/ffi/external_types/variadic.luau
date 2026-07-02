--!nocheck
--!nolint
-- Calls variadic external functions. `ffi.c.fn(args, ret, fixedCount)` marks a
-- signature as variadic: the first `fixedCount` arguments are named and the
-- rest are variadic. Every argument type (fixed and variadic) is listed.
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_types/lib.c")
local c = ffi.c

-- sum_variadic(int count, ...): count is the single fixed argument
local sumInts = c.fn({ c.int, c.int, c.int, c.int }, c.int, 1):callable(lib:find("sum_variadic"))
local intResult = ffi.box(c.int.size)
sumInts(
	intResult,
	c.int:box(3):ref(), -- count
	c.int:box(10):ref(),
	c.int:box(20):ref(),
	c.int:box(30):ref()
)
assert(c.int:readData(intResult) == 60, "variadic int sum failed")

-- A different variadic arity reuses the same pattern with its own signature
local sumTwo = c.fn({ c.int, c.int, c.int }, c.int, 1):callable(lib:find("sum_variadic"))
local twoResult = ffi.box(c.int.size)
sumTwo(twoResult, c.int:box(2):ref(), c.int:box(100):ref(), c.int:box(23):ref())
assert(c.int:readData(twoResult) == 123, "variadic int sum (arity 2) failed")

-- Variadic doubles (C promotes variadic floats to double)
local sumDoubles = c.fn({ c.int, ffi.f64, ffi.f64 }, ffi.f64, 1)
	:callable(lib:find("sum_variadic_double"))
local doubleResult = ffi.box(ffi.f64.size)
sumDoubles(doubleResult, c.int:box(2):ref(), ffi.f64:box(1.5):ref(), ffi.f64:box(2.25):ref())
assert(ffi.f64:readData(doubleResult) == 3.75, "variadic double sum failed")

-- A variadic signature stringifies with a `...` marker at the boundary
local signature = tostring(c.fn({ c.int, c.int }, c.int, 1))
assert(signature:find("%.%.%.") ~= nil, `variadic signature should show '...', got {signature}`)

print("external_types/variadic: variadic calls behave correctly")
