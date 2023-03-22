<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD026 -->

# ✏️ Writing Lune Scripts for Roblox

Lune has a powerful built-in library and set of APIs for manipulating Roblox place files and model files. It contains APIs for reading & writing files, and gives you instances to use, just as if you were scripting inside of the Roblox engine, albeit with a more limited API.

For a full list of the currently implemented APIs, check out the [API Reference](https://github.com/filiptibell/lune/wiki/Roblox---API-Reference) page.

## Example Scripts

### `1` - Make all parts anchored in a place file

```lua
local roblox = require("@lune/roblox")

-- Read the place file called myPlaceFile.rbxl into a DataModel called "game"
-- This works exactly the same as in Roblox, except "game" does not exist by default - you have to load it from a file!
local game = roblox.readPlaceFile("myPlaceFile.rbxl")
local workspace = game:GetService("Workspace")

-- Make all of the parts in the workspace anchored
for _, descendant in workspace:GetDescendants() do
	if descendant:IsA("BasePart") then
		descendant.Anchored = true
	end
end

-- Save the DataModel (game) back to the file that we read it from
roblox.writePlaceFile("myPlaceFile.rbxl")
```

---

### `2` - Save instances in a place as individual model files

```lua
local roblox = require("@lune/roblox")
local fs = require("@lune/fs")

-- Here we load a file just like in the first example
local game = roblox.readPlaceFile("myPlaceFile.rbxl")
local workspace = game:GetService("Workspace")

-- We use a normal Lune API to make sure a directory exists to save our models in
fs.writeDir("models")

-- Then we save all of our instances in Workspace as model files, in our new directory
-- Note that a model file can actually contain several instances at once, so we pass a table here
for _, child in workspace:GetChildren() do
	roblox.writeModelFile("models/" .. child.Name, { child })
end
```

---

### `3` - Make a new place from scratch

```lua
local roblox = require("@lune/roblox")
local Instance = roblox.Instance

-- You can even create a new DataModel using Instance.new, which is not normally possible in Roblox
-- This is normal - most instances that are not normally accessible in Roblox can be manipulated using Lune!
local game = Instance.new("DataModel")
local workspace = game:GetService("Workspace")

-- Here we just make a bunch of models with parts in them for demonstration purposes
for i = 1, 50 do
	local model = Instance.new("Model")
	model.Name = "Model #" .. tostring(i)
	model.Parent = workspace
	for j = 1, 4 do
		local part = Instance.new("Part")
		part.Name = "Part #" .. tostring(j)
		part.Parent = model
	end
end

-- As always, we have to save the DataModel (game) to a file when we're done
roblox.writePlaceFile("myPlaceWithLotsOfModels.rbxl")
```
