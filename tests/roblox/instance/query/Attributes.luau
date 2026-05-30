local query = require("./tree")
local root = query.build()

-- Attribute presence
query.check(root, "[$Variety]", { "ModelA" })
query.check(root, "[$Nonexistent]", {})

-- String attribute equality is case-sensitive
query.check(root, "[$Variety = Fuji]", { "ModelA" })
query.check(root, "[$Variety = fuji]", {})

-- Numeric attribute equality coerces the literal to a number; quotes only widen
-- the allowed character set, they do not force a string comparison
query.check(root, "[$Flavor = 4]", { "ModelA" })
query.check(root, '[$Flavor = "4"]', { "ModelA" })
query.check(root, '[$Flavor = "4.0"]', { "ModelA" })

-- Boolean attribute accepts only true/false
query.check(root, "[$OnFire = true]", { "PartD" })
query.check(root, "[$OnFire = 1]", {})
