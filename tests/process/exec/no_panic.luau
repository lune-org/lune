local process = require("@lune/process")

-- Executing a non existent command as a child process
-- should not panic, but should error

local success = pcall(process.exec, "someProgramThatDoesNotExist")
assert(not success, "Spawned a non-existent program")
