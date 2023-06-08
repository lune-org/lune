local process = require("@lune/process")
local stdio = require("@lune/stdio")

stdio.ewrite("Hello, stderr!")

process.exit(0)
