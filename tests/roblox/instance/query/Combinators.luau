local query = require("./tree")
local root = query.build()

-- A bare selector (implicit `>>`) matches descendants at any depth
query.check(root, "Part", { "RedTree", "PartC", "PartD" })
query.check(root, ">> Part", { "RedTree", "PartC", "PartD" })

-- A leading child combinator matches direct children of the receiver only
query.check(root, "> Part", { "PartD" })

-- Child combinator between compounds: a tagged SwordPart directly under a Model
query.check(root, "Model > .SwordPart", { "MeshA" })

-- Descendant combinator between compounds: Parts anywhere under a Model
query.check(root, "Model >> Part", { "RedTree", "PartC" })

-- Whitespace around combinators is insignificant
query.check(root, "Model>>Part", { "RedTree", "PartC" })
