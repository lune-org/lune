local roblox = require("@lune/roblox") :: any
local Content = roblox.Content
local Instance = roblox.Instance
local Enum = roblox.Enum

assert(Content.none, "Content.none did not exist")
assert(
	Content.none.SourceType == Enum.ContentSourceType.None,
	"Content.none's SourceType was wrong"
)
assert(Content.none.Uri == nil, "Content.none's Uri field was wrong")
assert(Content.none.Object == nil, "Content.none's Object field was wrong")

local uri = Content.fromUri("test uri")
assert(uri.SourceType == Enum.ContentSourceType.Uri, "URI Content's SourceType was wrong")
assert(uri.Uri == "test uri", "URI Content's Uri field was wrong")
assert(uri.Object == nil, "URI Content's Object field was wrong")

assert(not pcall(Content.fromUri), "Content.fromUri accepted no argument")
assert(not pcall(Content.fromUri, false), "Content.fromUri accepted a boolean argument")
assert(not pcall(Content.fromUri, Enum), "Content.fromUri accepted a UserData as an argument")
assert(
	not pcall(Content.fromUri, buffer.create(0), "Content.fromUri accepted a buffer as an argument")
)

-- It feels weird that this is allowed because `EditableImage` is very much
-- not an Instance. But what can you do?
local target = Instance.new("EditableImage")
local object = Content.fromObject(target)
assert(object.SourceType == Enum.ContentSourceType.Object, "Object Content's SourceType was wrong")
assert(object.Uri == nil, "Object Content's Uri field was wrong")
assert(object.Object == target, "Object Content's Object field was wrong")

assert(not pcall(Content.fromObject), "Content.fromObject accepted no argument")
assert(not pcall(Content.fromObject, false), "Content.fromObject accepted a boolean argument")
assert(
	not pcall(Content.fromObject, Enum),
	"Content.fromObject accepted a non-Instance/Object UserData as an argument"
)
assert(
	not pcall(
		Content.fromObject,
		buffer.create(0),
		"Content.fromObject accepted a buffer as an argument"
	)
)

assert(
	not pcall(Content.fromObject, Instance.new("Folder")),
	"Content.fromObject accepted an Instance as an argument"
)

assert(
	tostring(Content.none) == "Content(None)",
	`expected tostring(Content.none) to be Content(None), it was actually {Content.none}`
)
assert(
	tostring(uri) == "Content(Uri=test uri)",
	`expected tostring(URI Content) to be Content(Uri=...), it was actually {uri}`
)
assert(
	tostring(object) == "Content(Object)",
	`expected tostring(Object Content) to be Content(Object), it was actually {object}`
)
