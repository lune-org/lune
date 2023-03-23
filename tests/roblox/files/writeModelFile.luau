local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local instances = {
	Instance.new("Model"),
	Instance.new("Part"),
}

roblox.writeModelFile("bin/temp-model.rbxm", instances)
roblox.writeModelFile("bin/temp-model.rbxmx", instances)

local savedBinary = roblox.readModelFile("bin/temp-model.rbxm")
local savedXml = roblox.readModelFile("bin/temp-model.rbxmx")

assert(savedBinary[1].Name ~= "ROOT")
assert(savedXml[1].Name ~= "ROOT")

assert(savedBinary[1].Name ~= "DataModel")
assert(savedXml[1].Name ~= "DataModel")

assert(savedBinary[1].ClassName == "Model")
assert(savedBinary[2].ClassName == "Part")

assert(savedXml[1].ClassName == "Model")
assert(savedXml[2].ClassName == "Part")
