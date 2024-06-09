local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")
local child = process.spawn("echo", { "lsp" })
task.wait(1)


stdio.write(buffer.tostring(child.stdout:readToEnd()))
stdio.write(buffer.tostring(child.stdout:readToEnd()))
stdio.write(buffer.tostring(child.stdout:readToEnd()))

-- while true do
--     child.stdin:write("hello world")
--     local buf = child.stdout:read()

--     if buffer.len(buf) == 0 then
--         break
--     end

--     stdio.write(buffer.tostring(buf) .. "\n")
--     -- stdio.write(buffer.tostring(child.stderr:read() .. child.stderr:read() .. child.stderr:read() .. child.stderr:read()))
-- end