local query = require("./tree")
local root = query.build()

-- Class selectors match via IsA, are case-sensitive, and return results in
-- preorder depth-first order. The receiver itself is never included.

query.check(root, "MeshPart", { "MeshA" })
query.check(root, "Part", { "RedTree", "PartC", "PartD" })

-- Unknown class names match nothing (no error)
query.check(root, "Paart", {})

-- Class matching is case-sensitive
query.check(root, "meshpart", {})

-- The receiver (a Folder) is excluded from its own results
query.check(root, "Folder", { "FolderB" })
