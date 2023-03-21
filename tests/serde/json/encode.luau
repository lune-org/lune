local serde = require("@lune/serde") :: any
local source = require("./source")

local decoded = serde.decode("json", source.pretty)

assert(type(decoded) == "table", "Decoded payload was not a table")
assert(decoded.Hello == "World", "Decoded payload Hello was not World")
assert(type(decoded.Inner) == "table", "Decoded payload Inner was not a table")
assert(type(decoded.Inner.Array) == "table", "Decoded payload Inner.Array was not a table")
assert(type(decoded.Inner.Array[1]) == "number", "Decoded payload Inner.Array[1] was not a number")
assert(type(decoded.Inner.Array[2]) == "number", "Decoded payload Inner.Array[2] was not a number")
assert(type(decoded.Inner.Array[3]) == "number", "Decoded payload Inner.Array[3] was not a number")
assert(decoded.Foo == "Bar", "Decoded payload Foo was not Bar")

local encoded = serde.encode("json", decoded, false)
assert(encoded == source.encoded, "JSON round-trip did not produce the same result")

local encodedPretty = serde.encode("json", decoded, true)
assert(encodedPretty == source.pretty, "JSON round-trip did not produce the same result (pretty)")
