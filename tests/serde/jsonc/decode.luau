local serde = require("@lune/serde")
local source = require("./source")

local decoded = serde.decode("jsonc", source.encoded)

assert(type(decoded) == "table", "Decoded payload was not a table")
assert(decoded.Hello == "World", "Decoded payload Hello was not World")
assert(type(decoded.Inner) == "table", "Decoded payload Inner was not a table")
assert(type(decoded.Inner.Array) == "table", "Decoded payload Inner.Array was not a table")
assert(type(decoded.Inner.Array[1]) == "number", "Decoded payload Inner.Array[1] was not a number")
assert(type(decoded.Inner.Array[2]) == "number", "Decoded payload Inner.Array[2] was not a number")
assert(type(decoded.Inner.Array[3]) == "number", "Decoded payload Inner.Array[3] was not a number")
assert(decoded.Foo == "Bar", "Decoded payload Foo was not Bar")
