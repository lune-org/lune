--!nocheck
--!nolint
local ffi = require("@lune/ffi")

local box = ffi.box(ffi.i32.size)
local ref = box:leak()

box = nil

collectgarbage("collect")
collectgarbage("collect")
collectgarbage("collect")

ffi.free(ref)

collectgarbage("collect")
collectgarbage("collect")
collectgarbage("collect")
