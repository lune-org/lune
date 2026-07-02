local query = require("./tree")
local root = query.build()

-- Tag selectors (`.Tag`) match CollectionService tags
query.check(root, ".Fruit", { "RedTree" })
query.check(root, ".SwordPart", { "MeshA" })

-- Name selectors (`#Name`) match the exact Instance.Name
query.check(root, "#RedTree", { "RedTree" })
query.check(root, "#PartC", { "PartC" })
query.check(root, "#Nonexistent", {})
