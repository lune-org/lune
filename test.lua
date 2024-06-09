local process = require("@lune/process")
local stdio = require("@lune/stdio")

local child = process.spawn("luau-lsp", { "lsp" })

while true do
    child.stdin:write("hello world")
    local buf = child.stdout:read()

    if buffer.len(buf) == 0 then
        break
    end

    stdio.write(buffer.tostring(buf) .. "\n")
    -- stdio.write(buffer.tostring(child.stderr:read() .. child.stderr:read() .. child.stderr:read() .. child.stderr:read()))
end