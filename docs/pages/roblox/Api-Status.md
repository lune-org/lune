<!-- markdownlint-disable MD041 -->
<!-- markdownlint-disable MD033 -->

# API Status

This is a page indicating the current implementation status for instance methods and datatypes in the `roblox` library.

If an API on a class is not listed here it may not be within the scope for Lune and may not be implemented in the future. <br />
However, if a recently added datatype is missing, and it can be used as an instance property, it is likely that it will be implemented.

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
-   [`GetAttribute`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetAttribute)
-   [`GetAttributes`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetAttributes)
-   [`GetChildren`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetChildren)
-   [`GetDescendants`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetDescendants)
-   [`GetFullName`](https://create.roblox.com/docs/reference/engine/classes/Instance#GetFullName)
-   [`IsA`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsA)
-   [`IsAncestorOf`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsAncestorOf)
-   [`IsDescendantOf`](https://create.roblox.com/docs/reference/engine/classes/Instance#IsDescendantOf)
-   [`SetAttribute`](https://create.roblox.com/docs/reference/engine/classes/Instance#SetAttribute)

### `DataModel`

Currently implemented APIs:

-   [`GetService`](https://create.roblox.com/docs/reference/engine/classes/ServiceProvider#GetService)
-   [`FindService`](https://create.roblox.com/docs/reference/engine/classes/ServiceProvider#FindService)

### `CollectionService`

Currently implemented APIs:

-   [`AddTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#AddTag)
-   [`GetTags`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#GetTags)
-   [`HasTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#HasTag)
-   [`RemoveTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#RemoveTag)

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

Note that these datatypes are kept as up-to-date as possible, but recently added members & methods may be missing.
