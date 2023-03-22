<!-- markdownlint-disable MD041 -->

# API Reference

Welcome to the API reference page for the built-in `roblox` library!

All of the following static functions, classes, and datatypes can be imported using `require("@lune/roblox")`.

## Static Functions

### `readPlaceFile`

Reads a place file into a DataModel instance.

```lua
local roblox = require("@lune/roblox")
local game = roblox.readPlaceFile("filePath.rbxl")
```

### `readModelFile`

Reads a model file into a table of instances.

```lua
local roblox = require("@lune/roblox")
local instances = roblox.readModelFile("filePath.rbxm")
```

### `writePlaceFile`

Writes a DataModel instance to a place file.

```lua
local roblox = require("@lune/roblox")
roblox.writePlaceFile("filePath.rbxl", game)
```

### `writeModelFile`

Writes one or more instances to a model file.

```lua
local roblox = require("@lune/roblox")
roblox.writeModelFile("filePath.rbxm", { instance1, instance2, ... })
```

## Classes

### `Instance`

Currently implemented APIs:

-   [`new`](https://create.roblox.com/docs/reference/engine/datatypes/Instance#new) - note that this does not include the second `parent` argument
-   [`Clone`](https://create.roblox.com/docs/reference/engine/classes/Instance#Clone)
-   [`Destroy`](https://create.roblox.com/docs/reference/engine/classes/Instance#Destroy)
-   [`ClearAllChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#ClearAllChildren)
-   [`FindFirstAncestor`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestor)
-   [`FindFirstAncestorOfClass`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestorOfClass)
-   [`FindFirstAncestorWhichIsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstAncestorWhichIsA)
-   [`FindFirstChild`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChild)
-   [`FindFirstChildOfClass`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChildOfClass)
-   [`FindFirstChildWhichIsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstChildWhichIsA)
-   [`FindFirstDescendant`](https://create.roblox.com/docs/reference/engine/classes/Instance#FindFirstDescendant)
-   [`GetChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetChildren)
-   [`GetDescendants`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetDescendants)
-   [`GetFullName`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetFullName)
-   [`IsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsA)
-   [`IsAncestorOf`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsAncestorOf)
-   [`IsDescendantOf`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsDescendantOf)

Not yet implemented, but planned:

-   [`GetAttribute`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetAttribute)
-   [`GetAttributes`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetAttributes)
-   [`SetAttribute`](https://create.roblox.com/docs/reference/engine/classes/Instance#SetAttribute)

### `DataModel`

Currently implemented APIs:

-   [`GetService`](https://create.roblox.com/docs/reference/engine/classes/ServiceProvider#GetService)
-   [`FindService`](https://create.roblox.com/docs/reference/engine/classes/ServiceProvider#FindService)

## Datatypes

Currently implemented datatypes:

-   [`Axes`](https://create.roblox.com/docs/reference/engine/datatypes/Axes)
-   [`BrickColor`](https://create.roblox.com/docs/reference/engine/datatypes/BrickColor)
-   [`CFrame`](https://create.roblox.com/docs/reference/engine/datatypes/CFrame)
-   [`Color3`](https://create.roblox.com/docs/reference/engine/datatypes/Color3)
-   [`ColorSequence`](https://create.roblox.com/docs/reference/engine/datatypes/ColorSequence)
-   [`ColorSequenceKeypoint`](https://create.roblox.com/docs/reference/engine/datatypes/ColorSequenceKeypoint)
-   [`Enum`](https://create.roblox.com/docs/reference/engine/datatypes/Enum)
-   [`Faces`](https://create.roblox.com/docs/reference/engine/datatypes/Faces)
-   [`Font`](https://create.roblox.com/docs/reference/engine/datatypes/Font)
-   [`NumberRange`](https://create.roblox.com/docs/reference/engine/datatypes/NumberRange)
-   [`NumberSequence`](https://create.roblox.com/docs/reference/engine/datatypes/NumberSequence)
-   [`NumberSequenceKeypoint`](https://create.roblox.com/docs/reference/engine/datatypes/NumberSequenceKeypoint)
-   [`PhysicalProperties`](https://create.roblox.com/docs/reference/engine/datatypes/PhysicalProperties)
-   [`Ray`](https://create.roblox.com/docs/reference/engine/datatypes/Ray)
-   [`Rect`](https://create.roblox.com/docs/reference/engine/datatypes/Rect)
-   [`Region3`](https://create.roblox.com/docs/reference/engine/datatypes/Region3)
-   [`Region3int16`](https://create.roblox.com/docs/reference/engine/datatypes/Region3int16)
-   [`UDim`](https://create.roblox.com/docs/reference/engine/datatypes/UDim)
-   [`UDim2`](https://create.roblox.com/docs/reference/engine/datatypes/UDim2)
-   [`Vector2`](https://create.roblox.com/docs/reference/engine/datatypes/Vector2)
-   [`Vector2int16`](https://create.roblox.com/docs/reference/engine/datatypes/Vector2int16)
-   [`Vector3`](https://create.roblox.com/docs/reference/engine/datatypes/Vector3)
-   [`Vector3int16`](https://create.roblox.com/docs/reference/engine/datatypes/Vector3int16)

Note that these datatypes are kept as up-to-date as possible, but some very new members may be missing.
