local ffi = require("@lune/ffi")

assert(typeof(ffi.int) == "CType")
assert(tostring(ffi.int) == "int")

assert(typeof(ffi.int:ptr()) == "CPtr")
assert(tostring(ffi.int:ptr()) == "int")
assert(tostring(ffi.int:arr(5):ptr()) == " <CArr( int, length = 5 )> ")

assert(typeof(ffi.int:arr(5)) == "CArr")
assert(tostring(ffi.int:arr(5)) == " int, length = 5 ")
assert(tostring(ffi.int:ptr():arr(5)) == " <CPtr(int)>, length = 5 ")

assert(typeof(ffi.funcInfo({ ffi.int }, ffi.int)) == "CFunc")
assert(tostring(ffi.funcInfo({ ffi.int }, ffi.int)) == " (int) -> int ")
assert(tostring(ffi.funcInfo({ ffi.int, ffi.double }, ffi.int)) == " (int, double) -> int ")
assert(tostring(ffi.funcInfo({ ffi.int:ptr() }, ffi.int)) == " (<CPtr(int)>) -> int ")
assert(tostring(ffi.funcInfo({ ffi.int }, ffi.int:ptr())) == " (int) -> <CPtr(int)> ")
assert(tostring(ffi.funcInfo({ ffi.int:ptr() }, ffi.int:ptr())) == " (<CPtr(int)>) -> <CPtr(int)> ")
assert(
	tostring(ffi.funcInfo({ ffi.int:ptr(), ffi.int:ptr() }, ffi.int:ptr()))
		== " (<CPtr(int)>, <CPtr(int)>) -> <CPtr(int)> "
)

assert(typeof(ffi.structInfo({ ffi.int, ffi.char })) == "CStruct")
assert(
	tostring(ffi.structInfo({ ffi.int, ffi.char:ptr() }))
		== ` int, <CPtr(char)>, size = {ffi.structInfo({ ffi.int, ffi.char:ptr() }).size} `
)
