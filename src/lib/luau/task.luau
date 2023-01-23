type ThreadOrFunction = thread | (...any) -> ...any

-- NOTE: The async wait function gets passed in here by task.rs,
-- the same function will then be used for the global task library
local wait: (seconds: number?) -> number = ...

local task = {}

function task.cancel(thread: unknown)
	if type(thread) ~= "thread" then
		error(string.format("Argument #1 must be a thread, got %s", typeof(thread)))
	else
		coroutine.close(thread)
	end
end

function task.defer(tof: ThreadOrFunction, ...: any): thread
	if type(tof) == "thread" then
		local thread = coroutine.create(function(...)
			wait()
			coroutine.resume(tof, ...)
		end)
		coroutine.resume(thread, ...)
		return thread
	elseif type(tof) == "function" then
		local thread = coroutine.create(tof)
		task.defer(thread, ...)
		return thread
	else
		error(string.format("Argument #1 must be a thread or function, got %s", typeof(tof)))
	end
end

function task.delay(duration: number?, tof: ThreadOrFunction, ...: any): thread
	if duration ~= nil and type(duration) ~= "number" then
		error(string.format("Argument #1 must be a number or nil, got %s", typeof(duration)))
	end
	if type(tof) == "thread" then
		local thread = coroutine.create(function(...)
			wait(duration)
			coroutine.resume(tof, ...)
		end)
		coroutine.resume(thread, ...)
		return thread
	elseif type(tof) == "function" then
		local thread = coroutine.create(tof)
		task.delay(duration, thread, ...)
		return thread
	else
		error(string.format("Argument #2 must be a thread or function, got %s", typeof(tof)))
	end
end

function task.spawn(tof: ThreadOrFunction, ...: any): thread
	if type(tof) == "thread" then
		coroutine.resume(tof, ...)
		return tof
	elseif type(tof) == "function" then
		local thread = coroutine.create(tof)
		coroutine.resume(thread, ...)
		return thread
	else
		error(string.format("Argument #1 must be a thread or function, got %s", typeof(tof)))
	end
end

return task
