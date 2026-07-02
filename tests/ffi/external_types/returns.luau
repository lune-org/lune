--!nocheck
--!nolint
-- Calls external functions returning every common integer width and verifies
-- both the value and that no memory past the result slot was written. libffi
-- widens narrow integral returns to a full word, so a naive result buffer
-- would overflow; the canary bytes catch that regression.
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_types/lib.c")

local CANARY = 0x7E -- a byte pattern placed after the result slot

-- Call a zero-argument function whose return type is `retType`, checking the
-- returned value equals `expected` and that the bytes after the result are
-- left untouched.
local function checkReturn(name: string, retType, expected: number)
	-- Result slot + 8 canary bytes
	local resultBox = ffi.box(retType.size + 8)
	for i = 0, 7 do
		ffi.u8:writeData(resultBox, CANARY, retType.size + i)
	end

	local callable = ffi.c.fn({}, retType):callable(lib:find(name))
	callable(resultBox)

	local got = retType:readData(resultBox)
	assert(got == expected, `{name}: expected {expected}, got {got}`)

	for i = 0, 7 do
		assert(
			ffi.u8:readData(resultBox, retType.size + i) == CANARY,
			`{name}: return value overflowed into adjacent memory (byte {i})`
		)
	end
end

checkReturn("ret_i8", ffi.i8, -8)
checkReturn("ret_u8", ffi.u8, 200)
checkReturn("ret_i16", ffi.i16, -1600)
checkReturn("ret_u16", ffi.u16, 60000)
checkReturn("ret_i32", ffi.i32, -32000000)
checkReturn("ret_u32", ffi.u32, 4000000000)
checkReturn("ret_i64", ffi.i64, -9000000000)
checkReturn("ret_u64", ffi.u64, 18000000000)

-- Multiple arguments through the sized-caller fast path
local sum4 = ffi.c.fn({ ffi.c.int, ffi.c.int, ffi.c.int, ffi.c.int }, ffi.c.int)
	:callable(lib:find("sum4"))
local sumResult = ffi.box(ffi.c.int.size)
sum4(
	sumResult,
	ffi.c.int:box(1):ref(),
	ffi.c.int:box(2):ref(),
	ffi.c.int:box(3):ref(),
	ffi.c.int:box(4):ref()
)
assert(ffi.c.int:readData(sumResult) == 10, "sum4 returned the wrong result")

-- A void function that writes through a pointer argument. A pointer argument
-- is passed as a reference-to-reference, so the callee receives the address.
local writeByte = ffi.c.fn({ ffi.u8:ptr(), ffi.u8 }, ffi.c.void):callable(lib:find("write_byte"))
local outByte = ffi.box(1):zero()
writeByte(nil, outByte:ref():ref(), ffi.u8:box(42):ref())
assert(ffi.u8:readData(outByte) == 42, "write_byte out-parameter failed")

-- An array passed by value as an argument (ABI-equivalent to a struct of its
-- elements)
local tripleType = ffi.i32:arr(3)
local sumTriple = ffi.c.fn({ tripleType }, ffi.c.int):callable(lib:find("sum_triple"))
local tripleResult = ffi.box(ffi.c.int.size)
sumTriple(tripleResult, tripleType:box({ 10, 20, 30 }):ref())
assert(ffi.c.int:readData(tripleResult) == 60, "array-by-value argument failed")

print("external_types: integer-width returns and arguments behave correctly")
