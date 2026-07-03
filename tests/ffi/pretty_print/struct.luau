local ffi = require("@lune/ffi")
local c = ffi.c

assert(typeof(c.struct({ c.int, c.char })) :: string == "CStructInfo")
assert(
	tostring(c.struct({ c.int, c.char:ptr() }))
		== ` int, <CPtrInfo(char)>, size = {c.struct({ c.int, c.char:ptr() }).size} `
)
