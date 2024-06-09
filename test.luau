local process = require("@lune/process")

local a = process.spawn("echo", {"hello"})

print(a)
print(buffer.tostring(a.stdout:readToEnd()))
print(a.status())