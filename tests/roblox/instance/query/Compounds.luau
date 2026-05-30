local query = require("./tree")
local root = query.build()

-- A compound selector is a run of simple selectors that must all match the same
-- instance (logical AND). Whitespace between simple selectors is insignificant.

query.check(root, "Model.Apple", { "ModelA" })
query.check(root, "Part#RedTree", { "RedTree" })
query.check(root, "Part #RedTree", { "RedTree" })

-- "Part Part" is just "IsA Part AND IsA Part"
query.check(root, "Part Part", { "RedTree", "PartC", "PartD" })

-- "Model Part" is "IsA Model AND IsA Part", which nothing satisfies
query.check(root, "Model Part", {})

-- A compound combining several simple selector kinds
query.check(root, "Part.Fruit#RedTree", { "RedTree" })
