local process = require("@lune/process")

local IS_WINDOWS = process.os == "windows"

-- Inheriting stdio & environment variables should work

local echoMessage = "Hello from child process!"
local echoResult = process.exec("echo", {
	if IS_WINDOWS then '"$Env:TEST_VAR"' else '"$TEST_VAR"',
}, {
	env = { TEST_VAR = echoMessage },
	shell = if IS_WINDOWS then "powershell" else "bash",
	stdio = "inherit",
})

-- Windows uses \r\n (CRLF) and unix uses \n (LF)

local echoTrail = if IS_WINDOWS then "\r\n" else "\n"
assert(
	echoResult.stdout == (echoMessage .. echoTrail),
	"Inheriting stdio did not return proper output"
)
