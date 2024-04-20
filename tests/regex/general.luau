--!nocheck

local regex = require("@lune/regex")

local re = regex.new("[0-9]+")

assert(re:isMatch("look, a number: 1241425") == true)

local mtch = re:find("1337 wow")
assert(mtch ~= nil)
assert(mtch.start == 1)
assert(mtch.finish == 4)
assert(mtch.len == 4)
assert(mtch.text == "1337")
assert(#mtch == mtch.len)

re = regex.new([[([0-9]+) (\d+) \D+ \d+ (\d)]])
local captures = re:captures("1337 125600 wow! 1984 0")
assert(captures ~= nil)
assert(#captures == 3)
assert(captures:get(0).text == "1337 125600 wow! 1984 0")
assert(captures:get(1).text == "1337")
assert(captures:get(2).text == "125600")
assert(captures:get(3).text == "0")
assert(captures:get(4) == nil)

assert(captures:format("$0") == "1337 125600 wow! 1984 0")
assert(captures:format("$3 $2 $1") == "0 125600 1337")

re = regex.new("(?P<first>[^ ]+)[ ]+(?P<last>[^ ]+)(?P<space>[ ]*)")
captures = re:captures("w1 w2 w3 w4")
assert(captures:format("$last $first$space") == "w2 w1 ")

local split = regex.new([[,]]):split("1,2,3,4")
assert(#split == 4)
assert(split[1] == "1")
assert(split[2] == "2")
assert(split[3] == "3")
assert(split[4] == "4")
