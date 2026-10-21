local serde = require("@lune/serde")

local source = "\0Hello, world!\255"
local encoded = "0048656c6c6f2c20776f726c6421ff"

assert(serde.encode("hex", source) == encoded)
assert(serde.encode("hex", buffer.fromstring(source)) == encoded)
assert(serde.decode("hex", encoded) == source)
assert(serde.decode("hex", buffer.fromstring(string.upper(encoded))) == source)

assert(not pcall(serde.decode, "hex", "invalid"))
