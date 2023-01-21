local util = require("src/tests/net/request/util")
local pass = util.pass

pass("GET", "https://httpbin.org/absolute-redirect/3", "Redirect 3 times")
