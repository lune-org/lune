
local ffi = require("@lune/ffi")

-- ptr size test
assert(
    ffi.i32:ptr().size == ffi.i64:ptr().size,
    "All of Ptr.size must be same.\n"..
    "ffi.i32:ptr().size == ffi.i64:ptr().size failed"
)

-- inner test
local i32ptr = ffi.i32:ptr()
assert(
    rawequal(ffi.i32, i32ptr.inner),
    "Ptr.inner must be same with their parent\n"..
    "raweq ffi.i32 == ffi.i32:ptr().inner failed"
)
assert(
    rawequal(i32ptr, i32ptr:ptr().inner),
    "Ptr.inner must be same with their parent\n"..
    "raweq i32ptr == i32ptr:ptr().inner failed"
)
assert(
    rawequal(i32ptr, i32ptr:ptr().inner:ptr().inner:ptr().inner),
    "Ptr.inner must be same with their parent\n"..
    "raweq i32ptr == i32ptr:ptr().inner:ptr().inner:ptr().inner failed"
)

-- deep ptr test
local ok,err = pcall(function()
    i32ptr:ptr():ptr():ptr():ptr():ptr():ptr():ptr()
end)
assert(ok,`Deep ptr test failed.\n{err}`)
