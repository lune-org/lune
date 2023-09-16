local DateTime = require("@lune/datetime")

local TYPE = "DateTime"

local now = DateTime.now()

assert(typeof(now) == TYPE, `dateTime.now() should return a {TYPE}, returned {typeof(now)}`)
