local ffi = require("@lune/ffi")
local lib = require("../utils/compile")("./tests/ffi/external_print/lib.c")
local c = ffi.c

c.fn({}, c.void):callable(lib:find("hello_world"))(nil)
