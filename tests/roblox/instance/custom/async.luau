local roblox = require("@lune/roblox")

local game = roblox.Instance.new("DataModel")
local http = game:GetService("HttpService") :: any

roblox.implementMethod("HttpService", "GetAsync", function()
	-- TODO: Fill in method body
end)

-- TODO: Fill in rest of test cases here

http:GetAsync()
