local serde = require("@lune/serde")
local source = require("./source")

local decoded = serde.decode("json", source.pretty)
local encoded = serde.encode("json", decoded, false)
assert(encoded == source.encoded, "JSON round-trip did not produce the same result")

local encodedPretty = serde.encode("json", decoded, true)
assert(encodedPretty == source.pretty, "JSON round-trip did not produce the same result (pretty)")
