local serde = require("@lune/serde")
local source = require("./source")

local str = serde.encode("toml", source.decoded)
assert(str == source.encoded)
