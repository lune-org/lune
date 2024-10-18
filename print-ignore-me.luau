local ffi = require("@lune/ffi")

print(ffi.int)
print(ffi.int:ptr())
print(ffi.int:arr(5):ptr())
print(ffi.int:arr(5))

print(ffi.funcInfo({ ffi.int }, ffi.int))
print(ffi.funcInfo({ ffi.int, ffi.double }, ffi.int:ptr()))
print(ffi.funcInfo({ ffi.int, ffi.double }, ffi.int:ptr():ptr()))

print(ffi.structInfo({ ffi.int, ffi.char }))
