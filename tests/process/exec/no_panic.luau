local process = require("@lune/process")

-- Spawning a child process for a non-existent
-- program should not panic, but should error

local success = pcall(process.exec, "someProgramThatDoesNotExist")
assert(not success, "Spawned a non-existent program")
