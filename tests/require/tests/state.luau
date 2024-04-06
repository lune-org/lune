-- the idea of this test is that state_module stores some state in one of its local variable
-- we also set a global in the module to ensure the change we would have expected happens
local state_module = require("./state_module")

-- we confirm that without anything happening, the initial value is what we ecpect
assert(state_module.state == 10)
assert(_state_test_global == 10)

-- this second file also requires state_module and calls a function that changes both the local state and the global to 11
require("./state_second")

-- with correct module caching, we should see the change done in state_secone reflected here
assert(state_module.state == 11)
-- if this also fails then there is either a problem with globals, or state_second is not doing what we expect it to
assert(_state_test_global == 11)

