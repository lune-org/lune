local serde = require("@lune/serde")
local source = require("./source")

-- NOTE: Our json format (intentionally) does not round-trip or keep
-- comments, so we will strip those for the purposes of this test
local original = string.gsub(source.encoded, "%s*//[^\n]+", "")

local decoded = serde.decode("jsonc", original)
local encoded = serde.encode("jsonc", decoded, true)
assert(encoded == original, "JSONC round-trip did not produce the same result")
