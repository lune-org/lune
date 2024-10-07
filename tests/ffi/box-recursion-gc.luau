--!nocheck
--!nolint

local ffi = require("@lune/ffi")

local box = ffi.box(ffi.u8:ptr().size)
local ref = box:ref()
ffi.u8:ptr():into(box, ref)

local wt = setmetatable({}, { __mode = "v" })

wt[1] = box
wt[2] = ref

box = nil
ref = nil

collectgarbage("collect")

assert(wt[1] == nil and wt[2] == nil, "Box - ref recursion GC test failed")
