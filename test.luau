local process = require("@lune/process")

local a = process.spawn("yes", {})

print(a)
print(buffer.tostring(a.stdout:read()))
print(a.status())