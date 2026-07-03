local callableWrapper = require("../utils/callableWrapper")
local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_struct/lib.c")
local c = ffi.c

local argStructInfo = c.struct({ c.int, c.int:ptr() })
local resultStructInfo = c.struct({ c.int, c.int })

local ab = callableWrapper(lib:find("ab"), { argStructInfo }, resultStructInfo)
local result = ab({ 100, c.int:box(200):ref() } :: { any })
assert(result[1] == 300, `ab failed. result expected 300, got {result[1]}`)
assert(result[2] == 20000, `ab failed. result expected 300, got {result[2]}`)
