local ffi = require("@lune/ffi")
local c = ffi.c

assert(typeof(c.fn({ c.int }, c.int)) :: string == "CFnInfo")
assert(tostring(c.fn({ c.int }, c.int)) == " (int) -> int ")
assert(tostring(c.fn({ c.int, ffi.f32 }, c.int)) == " (int, f32) -> int ")
assert(tostring(c.fn({ c.int:ptr() }, c.int)) == " (<CPtrInfo(int)>) -> int ")
assert(tostring(c.fn({ c.int }, c.int:ptr())) == " (int) -> <CPtrInfo(int)> ")
assert(tostring(c.fn({ c.int:ptr() }, c.int:ptr())) == " (<CPtrInfo(int)>) -> <CPtrInfo(int)> ")
assert(
	tostring(c.fn({ c.int:ptr(), c.int:ptr() }, c.int:ptr()))
		== " (<CPtrInfo(int)>, <CPtrInfo(int)>) -> <CPtrInfo(int)> "
)
