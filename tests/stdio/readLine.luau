local stdio = require("@lune/stdio")

local function linePrefix(prefix:string)
    print("-------")
    stdio.write(prefix)
    local t = stdio.readLine()
    print("-------")
    return t
end

local function toEndPrefix(prefix:string)
    print("-------")
    stdio.write(prefix)
    local t = stdio.readToEnd()
    print("-------")
    return t
end

print(linePrefix("READLINE> "))
print(linePrefix("READLINE> "))
print(toEndPrefix("READTOEND> "))
