--!nocheck
--!nolint

-- A library stringifies to the exact path it was opened with
local lib, path = require("../utils/compile")("./tests/ffi/external_math/lib.c")
assert(tostring(lib) == path, `lib should stringify to {path}, got {tostring(lib)}`)

print(lib)
