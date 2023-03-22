local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local instances = {
	Instance.new("Model"),
	Instance.new("Part"),
}

roblox.writeModelFile("bin/temp-model.rbxm", instances)
roblox.writeModelFile("bin/temp-model.rbxmx", instances)
