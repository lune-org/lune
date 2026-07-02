local process = require("@lune/process")

assert(type(process.env) == "table", "Process env should be a plain table")

local randomKey = string.format("LUNE_TEST_%d", math.random(1, 999_999))

assert(process.env[randomKey] == nil, "Unset variable returned a non-nil value")

process.env[randomKey] = "abc"
assert(process.env[randomKey] == "abc", "Failed to set environment variable")

process.env[randomKey] = nil
assert(process.env[randomKey] == nil, "Failed to set environment variable")
