local serde = require("@lune/serde")
local source = require("./source")

local toml = serde.decode("toml", source.encoded)

assert(toml.package.name == "my-cool-toml-package")
assert(toml.package.version == "0.1.0")
assert(toml.values.epic == true)
