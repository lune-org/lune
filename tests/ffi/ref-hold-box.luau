--!nocheck
--!nolint

local ffi = require("@lune/ffi")
local box = ffi.box(ffi.i32.size)
local ref = box:ref()

local wt = setmetatable({}, { __mode = "v" })

wt[1] = box
box = nll

collectgarbage("collect")

assert(wt[1] ~= nil, "ref hold box failed")
