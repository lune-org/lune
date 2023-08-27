local DateTime = require("@lune/datetime")

local TIME = "2023-08-27T05:54:19Z"
local dateTime = DateTime.fromIsoDate(TIME):toLocalTime()
local expectedDateTimeValues =
	table.pack(string.match(TIME, "(%d%d%d%d)-?(%d?%d?)-?(%d?%d?)T(%d?%d?):(%d?%d?):(%d?%d?)Z$"))

-- TODO: Implement the test
