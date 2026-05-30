local query = require("./tree")
local root = query.build()

-- A selector list (the `,` operator) is a union of independent selectors. The
-- matches of each selector are appended in order, and results are NOT
-- de-duplicated across the list.

query.check(root, ".Fruit, .SwordPart", { "RedTree", "MeshA" })

-- The same instance matched by two selectors appears once per selector
query.check(root, "Part, Part", { "RedTree", "PartC", "PartD", "RedTree", "PartC", "PartD" })

-- Per-selector grouping: all of the first selector's matches, then the second's
query.check(root, "MeshPart, Part", { "MeshA", "RedTree", "PartC", "PartD" })
