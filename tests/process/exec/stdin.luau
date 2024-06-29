local process = require("@lune/process")

local IS_WINDOWS = process.os == "windows"

-- Windows uses \r\n (CRLF) and unix uses \n (LF)

local echoTrail = if IS_WINDOWS then "\r\n" else "\n"
local echoMessage = "Hello from child process!"

-- When passing stdin to powershell on windows we must "accept" using the double newline

local result = if IS_WINDOWS
	then process.exec("powershell", { "echo" }, { stdin = echoMessage .. "\n\n" })
	else process.exec("xargs", { "echo" }, { stdin = echoMessage })

local resultStdout = if IS_WINDOWS
	then string.sub(result.stdout, #result.stdout - #echoMessage - 1)
	else result.stdout
assert(resultStdout == echoMessage .. echoTrail, "Stdin passing did not return proper output")
