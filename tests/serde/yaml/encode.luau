local serde = require("@lune/serde")

local encoded = serde.encode("yaml", {
	integerString = "1",
	booleanString = "true",
	nullString = "null",
})
local decoded = serde.decode("yaml", encoded)

assert(typeof(decoded.integerString) == "string" and decoded.integerString == "1")
assert(typeof(decoded.booleanString) == "string" and decoded.booleanString == "true")
assert(typeof(decoded.nullString) == "string" and decoded.nullString == "null")
