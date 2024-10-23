local ffi = require("@lune/ffi")
local c = ffi.c

assert(typeof(c.int) :: string == "CTypeInfo")
assert(tostring(c.int) == "int")

assert(typeof(c.int:ptr()) :: string == "CPtrInfo")
assert(tostring(c.int:ptr()) == "int")
assert(tostring(c.int:arr(5):ptr()) == " <CArrInfo( int, length = 5 )> ")

assert(typeof(c.int:arr(5)) :: string == "CArrInfo")
assert(tostring(c.int:arr(5)) == " int, length = 5 ")
assert(tostring(c.int:ptr():arr(5)) == " <CPtrInfo(int)>, length = 5 ")

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

assert(typeof(c.struct({ c.int, c.char })) :: string == "CStructInfo")
assert(
	tostring(c.struct({ c.int, c.char:ptr() }))
		== ` int, <CPtrInfo(char)>, size = {c.struct({ c.int, c.char:ptr() }).size} `
)

-- FIXME: add box, ref pretty-print test
