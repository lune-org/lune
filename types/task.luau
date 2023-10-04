--[=[
	@class Task

	Built-in task scheduler & thread spawning

	### Example usage

	```lua
	local task = require("@lune/task")

	-- Waiting for a certain amount of time
	task.wait(1)
	print("Waited for one second")

	-- Running a task after a given amount of time
	task.delay(2, function()
		print("Ran after two seconds")
	end)

	-- Spawning a new task that runs concurrently
	task.spawn(function()
		print("Running instantly")
		task.wait(1)
		print("One second passed inside the task")
	end)

	print("Running after task.spawn yields")
	```
]=]
local task = {}

--[=[
	@within Task

	Stops a currently scheduled thread from resuming.

	@param thread The thread to cancel
]=]
function task.cancel(thread: thread) end

--[=[
	@within Task

	Defers a thread or function to run at the end of the current task queue.

	@param functionOrThread The function or thread to defer
	@return The thread that will be deferred
]=]
function task.defer<T...>(functionOrThread: thread | (T...) -> ...any, ...: T...): thread
	return nil :: any
end

--[=[
	@within Task

	Delays a thread or function to run after `duration` seconds.

	@param functionOrThread The function or thread to delay
	@return The thread that will be delayed
]=]
function task.delay<T...>(
	duration: number,
	functionOrThread: thread | (T...) -> ...any,
	...: T...
): thread
	return nil :: any
end

--[=[
	@within Task

	Instantly runs a thread or function.

	If the spawned task yields, the thread that spawned the task
	will resume, letting the spawned task run in the background.

	@param functionOrThread The function or thread to spawn
	@return The thread that was spawned
]=]
function task.spawn<T...>(functionOrThread: thread | (T...) -> ...any, ...: T...): thread
	return nil :: any
end

--[=[
	@within Task

	Waits for *at least* the given amount of time.

	The minimum wait time possible when using `task.wait` is limited by the underlying OS sleep implementation.
	For most systems this means `task.wait` is accurate down to about 5 milliseconds or less.

	@param duration The amount of time to wait
	@return The exact amount of time waited
]=]
function task.wait(duration: number?): number
	return nil :: any
end

return task
