local stdio = require("@lune/stdio")
local process = require("@lune/process")

stdio.ewrite("Hello, stderr!")

process.exit(0)
