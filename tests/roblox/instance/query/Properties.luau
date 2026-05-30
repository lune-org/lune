local query = require("./tree")
local root = query.build()

-- Property selectors compare against the effective value, falling back to the
-- class default when the property has not been explicitly set.

query.check(root, "[CanCollide = false]", { "RedTree" })
-- MeshA / PartC / PartD all keep the default CanCollide = true
query.check(root, "[CanCollide = true]", { "MeshA", "PartC", "PartD" })

-- Default numeric property value
query.check(root, "[Transparency = 0]", { "RedTree", "MeshA", "PartC", "PartD" })

-- Enum properties are matched by enum item name (using the default, Plastic)
query.check(root, "[Material = Plastic]", { "RedTree", "MeshA", "PartC", "PartD" })
