local query = require("./tree")
local root = query.build()

-- `:not(...)` negates a selector list in the current scope, returning every
-- descendant that matches none of the inner selectors.
query.check(root, ":not(MeshPart)", { "ModelA", "RedTree", "FolderB", "ModelC", "PartC", "PartD" })
query.check(root, ":not(Part, Model)", { "MeshA", "FolderB" })

-- A compound combined with :not
query.check(root, "BasePart:not(MeshPart)", { "RedTree", "PartC", "PartD" })

-- `:has(...)` matches instances whose subtree contains a match, evaluated
-- relative to the instance.
query.check(root, "Model:has(.Fruit)", { "ModelA" })
query.check(root, ":has(.SwordPart)", { "ModelA" })

-- A leading child combinator inside :has anchors at the subject's children;
-- MeshA (the only SwordPart) has no children, so nothing matches
query.check(root, "MeshPart:has(> .SwordPart)", {})

-- The descendant form does match where the child form does not
query.check(root, "Model:has(> .SwordPart)", { "ModelA" })
