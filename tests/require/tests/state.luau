-- the idea of this test is that state_module stores some state in one of its local
-- variable
local state_module = require("./state_module")

-- we confirm that without anything happening, the initial value is what we expect
assert(state_module.state == 10)

-- this second file also requires state_module and calls a function that changes the local
-- state to 11
require("./state_second")

-- with correct module caching, we should see the change done in state_secone reflected
-- here
assert(state_module.state == 11)
