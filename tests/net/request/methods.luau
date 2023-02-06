local util = require("./util")
local pass = util.pass

-- stylua: ignore start
pass("GET",    "https://httpbin.org/get",    "Request method - GET")
pass("POST",   "https://httpbin.org/post",   "Request method - POST")
pass("PATCH",  "https://httpbin.org/patch",  "Request method - PATCH")
pass("PUT",    "https://httpbin.org/put",    "Request method - PUT")
pass("DELETE", "https://httpbin.org/delete", "Request method - DELETE")
