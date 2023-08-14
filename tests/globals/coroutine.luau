local task = require("@lune/task")

-- Coroutines should return true, ret values OR false, error

local function pass()
	coroutine.yield(1, 2, 3)
	coroutine.yield(4, 5, 6)
end

local function fail()
	error("Error message")
end

local thread1 = coroutine.create(pass)
local t10, t11, t12, t13 = coroutine.resume(thread1)
assert(t10 == true, "Coroutine resume should return true as first value unless errored")
assert(t11 == 1, "Coroutine resume should return values yielded to it (1)")
assert(t12 == 2, "Coroutine resume should return values yielded to it (2)")
assert(t13 == 3, "Coroutine resume should return values yielded to it (3)")

local thread2 = coroutine.create(fail)
local t20, t21 = coroutine.resume(thread2)
assert(t20 == false, "Coroutine resume should return false as first value when errored")
assert(#tostring(t21) > 0, "Coroutine resume should return error as second if it errors")

-- Coroutine suspended status should be correct

assert(
	coroutine.status(thread1) == "suspended",
	"Coroutine status should return suspended properly"
)
assert(coroutine.status(thread2) == "dead", "Coroutine status should return dead properly")

-- Coroutines should return values yielded after the first

local t30, t31, t32, t33 = coroutine.resume(thread1)
assert(t30 == true, "Coroutine resume should return true as first value unless errored")
assert(t31 == 4, "Coroutine resume should return values yielded to it (4)")
assert(t32 == 5, "Coroutine resume should return values yielded to it (5)")
assert(t33 == 6, "Coroutine resume should return values yielded to it (6)")

local t40, t41 = coroutine.resume(thread1)
assert(t40 == true, "Coroutine resume should return true as first value unless errored")
assert(t41 == nil, "Coroutine resume should return values yielded to it (7)")

-- Coroutine dead status should be correct after first yielding

assert(coroutine.status(thread1) == "dead", "Coroutine status should return dead properly")

-- Resume should error for dead coroutines

local success1 = coroutine.resume(thread1)
local success2 = coroutine.resume(thread2)

assert(success1 == false, "Coroutine resume on dead coroutines should return false")
assert(success2 == false, "Coroutine resume on dead coroutines should return false")

-- Wait should work inside native lua coroutines

local flag: boolean = false
coroutine.resume(coroutine.create(function()
	task.wait(0.1)
	flag = true
end))
assert(not flag, "Wait failed while inside coroutine (1)")
task.wait(0.2)
assert(flag, "Wait failed while inside coroutine (2)")

local flag2: boolean = false
coroutine.wrap(function()
	task.wait(0.1)
	flag2 = true
end)()
assert(not flag2, "Wait failed while inside wrap (1)")
task.wait(0.2)
assert(flag2, "Wait failed while inside wrap (2)")

-- Coroutines should be passed arguments on initial resume

local co = coroutine.create(function(a, b, c)
	assert(a == 1)
	assert(b == "Hello, world!")
	assert(c == true)
end)

coroutine.resume(co, 1, "Hello, world!", true)
