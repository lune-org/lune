local ffi = require("@lune/ffi")
local c = ffi.c

assert(typeof(c.int) :: string == "CType")
assert(tostring(c.int) == "int")

assert(typeof(c.int:ptr()) :: string == "CPtr")
assert(tostring(c.int:ptr()) == "int")
assert(tostring(c.int:arr(5):ptr()) == " <CArr( int, length = 5 )> ")

assert(typeof(c.int:arr(5)) :: string == "CArr")
assert(tostring(c.int:arr(5)) == " int, length = 5 ")
assert(tostring(c.int:ptr():arr(5)) == " <CPtr(int)>, length = 5 ")

assert(typeof(c.fn({ c.int }, c.int)) :: string == "CFn")
assert(tostring(c.fn({ c.int }, c.int)) == " (int) -> int ")
assert(tostring(c.fn({ c.int, ffi.f32 }, c.int)) == " (int, f32) -> int ")
assert(tostring(c.fn({ c.int:ptr() }, c.int)) == " (<CPtr(int)>) -> int ")
assert(tostring(c.fn({ c.int }, c.int:ptr())) == " (int) -> <CPtr(int)> ")
assert(tostring(c.fn({ c.int:ptr() }, c.int:ptr())) == " (<CPtr(int)>) -> <CPtr(int)> ")
assert(
	tostring(c.fn({ c.int:ptr(), c.int:ptr() }, c.int:ptr()))
		== " (<CPtr(int)>, <CPtr(int)>) -> <CPtr(int)> "
)

assert(typeof(c.struct({ c.int, c.char })) :: string == "CStruct")
assert(
	tostring(c.struct({ c.int, c.char:ptr() }))
		== ` int, <CPtr(char)>, size = {c.struct({ c.int, c.char:ptr() }).size} `
)

-- FIXME: add box, ref pretty-print test
