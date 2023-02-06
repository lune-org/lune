local util = require("./util")
local pass, fail = util.pass, util.fail

pass("GET", "https://httpbin.org/status/200", "Request status code - 200")
fail("GET", "https://httpbin.org/status/400", "Request status code - 400")
fail("GET", "https://httpbin.org/status/500", "Request status code - 500")
