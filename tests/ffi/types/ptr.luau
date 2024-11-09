local ffi = require("@lune/ffi")

-- ptr size test
assert(
	ffi.i32:ptr().size == ffi.i64:ptr().size,
	"size assersion failed, size of pointer should be same with each other (ffi.i32:ptr().size == ffi.i64:ptr().size)"
)

-- inner test
local i32ptr = ffi.i32:ptr()
assert(
	rawequal(ffi.i32, i32ptr.inner),
	"inner assersion failed, inner field must be same with their parent"
		.. " (ffi.i32 == ffi.i32:ptr().inner)"
)
assert(
	rawequal(i32ptr, i32ptr:ptr().inner),
	"inner assersion failed, inner field must be same with their parent"
		.. " (i32ptr == i32ptr:ptr().inner)"
)
assert(
	rawequal(i32ptr, i32ptr:ptr().inner:ptr().inner:ptr().inner),
	"inner assersion failed, inner field must be same with their parent"
		.. " (i32ptr == i32ptr:ptr().inner:ptr().inner:ptr().inner)"
)

-- deep ptr test
local ok, err = pcall(function()
	i32ptr:ptr():ptr():ptr():ptr():ptr():ptr():ptr()
end)
assert(ok, "deep ptr assersion failed\n" .. (err or ""))
