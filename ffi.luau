-- b = ffi.box(50);

-- buffer.create(1000)


-- ffi.i32:intoBox(b,100);

local ffi = require"@lune/ffi"

local libzip = ffi.dlopen("libzip.so")

print(libzip)

local myst = ffi.struct({ ffi.i32, ffi.i64 })
print(myst)
print(myst.inner[1])
print(ffi.i32.size,ffi.i64.size,ffi.i32:ptr().size)

local box = ffi.box(ffi.i64.size)
print(box,tostring(box))


-- print(myst:offset(0))
-- print(myst:offset(1))



-- ffi.i32:ptr().inner:ptr():ptr()

-- local lib = ffi.dlopen("asdf.so")

-- lib:dlsym("asdf")


-- local mybox = ffi.box(30)
-- mybox:zero()
-- print(#mybox)
-- print(tostring(mybox))

-- local myptrptr =
-- ffi.i32:ptr():ptr()

-- print(myptrptr)
-- print(myptrptr.inner)
-- print(myptrptr.inner.inner)
-- print(myptrptr.inner.inner.inner)
