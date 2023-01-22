local MINIMUM_DELAY_TIME = 1 / 100

type ThreadOrFunction<A..., R...> = thread | (A...) -> R...
type AnyThreadOrFunction = ThreadOrFunction<...any, ...any>

type WaitingThreadKind = "Normal" | "Deferred" | "Delayed"
type WaitingThread = {
	idx: number,
	kind: WaitingThreadKind,
	thread: thread,
	args: { [number]: any, n: number },
}

local waitingThreadCounter = 0
local waitingThreads: { WaitingThread } = {}

local function scheduleWaitingThreads()
	-- Grab currently waiting threads and clear the queue but keep capacity
	local threadsToResume: { WaitingThread } = table.clone(waitingThreads)
	table.clear(waitingThreads)
	table.sort(threadsToResume, function(t0, t1)
		local k0: WaitingThreadKind = t0.kind
		local k1: WaitingThreadKind = t1.kind
		if k0 == k1 then
			return t0.idx < t1.idx
		end
		if k0 == "Normal" then
			return true
		elseif k1 == "Normal" then
			return false
		elseif k0 == "Deferred" then
			return true
		else
			return false
		end
	end)
	-- Resume threads in order, giving args & waiting if necessary
	for _, waitingThread in threadsToResume do
		coroutine.resume(
			waitingThread.thread,
			table.unpack(waitingThread.args, 1, waitingThread.args.n)
		)
	end
end

local function insertWaitingThread(kind: WaitingThreadKind, tof: AnyThreadOrFunction, ...: any)
	if typeof(tof) ~= "thread" and typeof(tof) ~= "function" then
		if tof == nil then
			error("Expected thread or function, got nil", 3)
		end
		error(
			string.format("Expected thread or function, got %s %s", typeof(tof), tostring(tof)),
			3
		)
	end
	local thread = if type(tof) == "function" then coroutine.create(tof) else tof
	waitingThreadCounter += 1
	local waitingThread: WaitingThread = {
		idx = waitingThreadCounter,
		kind = kind,
		thread = thread,
		args = table.pack(...),
	}
	table.insert(waitingThreads, waitingThread)
	return waitingThread
end

local function cancel(thread: unknown)
	if typeof(thread) ~= "thread" then
		if thread == nil then
			error("Expected thread, got nil", 2)
		end
		error(string.format("Expected thread, got %s %s", typeof(thread), tostring(thread)), 2)
	else
		coroutine.close(thread)
	end
end

local function defer(tof: AnyThreadOrFunction, ...: any): thread
	local waiting = insertWaitingThread("Deferred", tof, ...)
	local original = waiting.thread
	waiting.thread = coroutine.create(function(...)
		task.wait(1 / 1_000_000)
		coroutine.resume(original, ...)
	end)
	scheduleWaitingThreads()
	return waiting.thread
end

local function delay(delay: number?, tof: AnyThreadOrFunction, ...: any): thread
	local waiting = insertWaitingThread("Delayed", tof, ...)
	local original = waiting.thread
	waiting.thread = coroutine.create(function(...)
		task.wait(math.max(MINIMUM_DELAY_TIME, delay or 0))
		coroutine.resume(original, ...)
	end)
	scheduleWaitingThreads()
	return waiting.thread
end

local function spawn(tof: AnyThreadOrFunction, ...: any): thread
	local waiting = insertWaitingThread("Normal", tof, ...)
	scheduleWaitingThreads()
	return waiting.thread
end

return {
	cancel = cancel,
	defer = defer,
	delay = delay,
	spawn = spawn,
}
