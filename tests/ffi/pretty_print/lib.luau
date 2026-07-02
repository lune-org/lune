--!nocheck
--!nolint
local ffi = require("@lune/ffi")
local process = require("@lune/process")

-- Make sure the shared library is built, then open it by a known path
require("../utils/compile")("./tests/ffi/external_math/lib.c")

local suffix = ({ linux = "so", macos = "dylib", windows = "dll" })[process.os]
local path = `./tests/ffi/external_math/lib.{suffix}`
local lib = ffi.open(path)

-- A library stringifies to the exact path it was opened with
assert(tostring(lib) == path, `lib should stringify to {path}, got {tostring(lib)}`)

print(lib)
