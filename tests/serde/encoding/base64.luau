local serde = require("@lune/serde")

local source = "\0Hello, world!\255"
local encoded = "AEhlbGxvLCB3b3JsZCH/"

assert(serde.encode("base64", source) == encoded)
assert(serde.encode("base64", buffer.fromstring(source)) == encoded)
assert(serde.decode("base64", encoded) == source)
assert(serde.decode("base64", buffer.fromstring(encoded)) == source)

assert(not pcall(serde.decode, "base64", "invalid"))
